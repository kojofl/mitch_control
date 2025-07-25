use std::{error::Error, sync::Arc};

use super::recording::Recording;
use btleplug::{
    api::{Peripheral as _, WriteType},
    platform::Peripheral,
};
use futures::{executor::block_on, StreamExt as _};
use lsl::{Pushable as _, StreamInfo, StreamOutlet};
use serde::Serialize;
use tauri::async_runtime::{spawn, JoinHandle};
use uuid::{uuid, Uuid};

pub const COMMAND_CHAR: Uuid = uuid!("d5913036-2d8a-41ee-85b9-4e361aa5c8a7");
pub const DATA_CHAR: Uuid = uuid!("09bf2c52-d1d9-c0b7-4145-475964544307");

#[derive(Clone, Serialize, Debug)]
pub struct Mitch {
    pub name: String,
    connected: bool,
    state: Option<MitchState>,
    #[serde(skip)]
    per: Peripheral,
    #[serde(skip)]
    handle: Option<Arc<JoinHandle<()>>>,
}

#[derive(Clone, Serialize, Debug)]
pub struct SerializableMitch {
    pub name: String,
    connected: bool,
    state: Option<MitchState>,
}

impl From<&Mitch> for SerializableMitch {
    fn from(value: &Mitch) -> Self {
        Self {
            name: value.name.clone(),
            connected: value.connected,
            state: value.state,
        }
    }
}

impl Drop for Mitch {
    fn drop(&mut self) {
        if self.connected {
            let _ = block_on(self.disconnect());
        }
    }
}

pub struct MyInfo(StreamInfo);
unsafe impl Send for MyInfo {}
impl MyInfo {
    pub fn from_info(info: StreamInfo) -> Self {
        Self(info)
    }
}

struct MyOutlet(StreamOutlet);
unsafe impl Send for MyOutlet {}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MitchState {
    SysStartup = 0x01,
    SysIdle = 0x02,
    SysStandby = 0x03,
    SysLog = 0x04,
    SysReadout = 0x05,
    SysTx = 0xF8,
    SysError = 0xFF,
    BootStartup = 0xf0,
    BootIdle = 0xf1,
    BootDownload = 0xf2,
}

impl TryFrom<u8> for MitchState {
    type Error = &'static str;

    fn try_from(value: u8) -> std::result::Result<MitchState, &'static str> {
        if (1_u8..=5_u8).contains(&value)
            || value == 0xf8
            || value == 0xff
            || value == 0xf0
            || value == 0xf1
            || value == 0xf2
        {
            return Ok(unsafe { *(&value as *const _ as *const MitchState) });
        }
        Err("Unknown state")
    }
}

enum Commands {
    GetState,
    StartAccelerometryStream,
    StartPressureStream,
    StopStream,
}

impl From<Recording> for Commands {
    fn from(value: Recording) -> Self {
        match value {
            Recording::Accelerometry => Self::StartAccelerometryStream,
            Recording::Pressure => Self::StartPressureStream,
        }
    }
}

impl AsRef<[u8]> for Commands {
    fn as_ref(&self) -> &[u8] {
        match self {
            Commands::GetState => &[130, 0],
            Commands::StartAccelerometryStream => &[0x02, 0x03, 0xF8, 0x04, 0x04],
            Commands::StartPressureStream => &[0x02, 0x03, 0xF8, 0x01, 0x04],
            Commands::StopStream => &[0x02, 0x01, 0x02],
        }
    }
}

type MitchResult<T> = std::result::Result<T, Box<dyn Error>>;

impl Mitch {
    pub fn new(name: &str, per: Peripheral) -> Self {
        Self {
            name: name.to_owned(),
            connected: false,
            state: None,
            per,
            handle: None,
        }
    }

    pub(crate) async fn connect(&mut self) -> MitchResult<()> {
        self.per.connect().await?;
        self.per.discover_services().await?;
        self.connected = true;
        self.update_state().await?;
        Ok(())
    }

    pub(crate) async fn disconnect(&mut self) -> MitchResult<()> {
        self.per.disconnect().await?;
        self.connected = false;
        Ok(())
    }

    pub(crate) async fn update_state(&mut self) -> MitchResult<()> {
        let c = self.per.characteristics();
        let cmd_char = c.iter().find(|c| c.uuid == COMMAND_CHAR).unwrap();
        match self
            .per
            .write(
                cmd_char,
                Commands::GetState.as_ref(),
                btleplug::api::WriteType::WithResponse,
            )
            .await
        {
            Err(_) => {
                self.state = None;
                self.connected = false;
            }
            _ => {}
        }
        self.state = Some(MitchState::try_from(self.per.read(cmd_char).await?[4])?);
        Ok(())
    }

    pub(crate) async fn start_recording(&mut self, rec_type: Recording) -> MitchResult<()> {
        let c = self.per.characteristics();
        let data_char = c.iter().find(|c| c.uuid == DATA_CHAR).unwrap();
        self.per.subscribe(data_char).await?;
        let cmd_char = c.iter().find(|c| c.uuid == COMMAND_CHAR).unwrap();
        self.per
            .write(
                cmd_char,
                Commands::from(rec_type).as_ref(),
                WriteType::WithResponse,
            )
            .await?;
        self.per.read(cmd_char).await?;
        let mut s = self.per.notifications().await?;
        let stream_name = self.name.clone();
        let handle = spawn(async move {
            let info = rec_type.info(stream_name.as_str());
            let outlet = MyOutlet(StreamOutlet::new(&info.0, 1, 360).unwrap());
            while let Some(b) = s.next().await {
                if b.uuid != DATA_CHAR {
                    continue;
                }
                match rec_type {
                    Recording::Accelerometry => {
                        outlet
                            .0
                            .push_sample(&[
                                i16::from_le_bytes([b.value[4], b.value[5]]),
                                i16::from_le_bytes([b.value[6], b.value[7]]),
                                i16::from_le_bytes([b.value[8], b.value[9]]),
                            ])
                            .unwrap();
                    }
                    Recording::Pressure => outlet
                        .0
                        .push_sample(&b.value[4..].iter().map(|b| *b as i16).collect::<Vec<i16>>())
                        .unwrap(),
                }
            }
        });
        self.handle = Some(Arc::new(handle));
        self.update_state().await?;
        Ok(())
    }

    pub(crate) async fn stop_recording(&mut self) -> MitchResult<()> {
        let characteristics = self.per.characteristics();
        let cmd_char = characteristics
            .iter()
            .find(|c| c.uuid == COMMAND_CHAR)
            .unwrap();
        self.per
            .write(
                cmd_char,
                Commands::StopStream.as_ref(),
                WriteType::WithResponse,
            )
            .await?;
        self.per.read(cmd_char).await?;
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        self.update_state().await?;
        Ok(())
    }
}
