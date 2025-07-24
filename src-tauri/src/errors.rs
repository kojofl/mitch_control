use std::{error::Error as E, fmt::Display};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Error {
    Connect,
    Disconnect,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self))
    }
}

impl E for Error {}
