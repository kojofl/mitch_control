use super::mitch::MyInfo;
use lsl::StreamInfo;

#[derive(Debug, Clone, Copy)]
pub enum Recording {
    Accelerometry,
    Pressure,
}

impl Recording {
    pub fn info(&self, name: &str) -> MyInfo {
        match self {
            Recording::Accelerometry => {
                let mut info = StreamInfo::new(
                    name,
                    "Accelerometry",
                    3,
                    50.0,
                    lsl::ChannelFormat::Double64,
                    name,
                )
                .unwrap();
                let mut chnls = info.desc().append_child("channels");
                chnls
                    .append_child("channel")
                    .append_child_value("label", "Pitch");
                chnls
                    .append_child("channel")
                    .append_child_value("label", "Roll");
                chnls
                    .append_child("channel")
                    .append_child_value("label", "Yaw");
                MyInfo::from_info(info)
            }
            Recording::Pressure => {
                let info =
                    StreamInfo::new(name, "Pressure", 16, 50.0, lsl::ChannelFormat::Int16, name)
                        .unwrap();
                MyInfo::from_info(info)
            }
        }
    }
}
