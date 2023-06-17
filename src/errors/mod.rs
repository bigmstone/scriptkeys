use std::{
    error::Error,
    fmt::{Display, Error as FmtError, Formatter},
};

#[derive(Debug)]
pub struct ConfigPathNotFound;

impl Error for ConfigPathNotFound {}

impl Display for ConfigPathNotFound {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        formatter
            .write_str("No config file found in default locations. Refer to the documentation.")
    }
}

#[derive(Debug)]
pub struct DeviceNotFound;

impl Error for DeviceNotFound {}

impl Display for DeviceNotFound {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        formatter.write_str("No device found. Refer to the documentation for supported devices.")
    }
}

#[derive(Debug)]
pub struct ScriptNotFound;

impl Error for ScriptNotFound {}

impl Display for ScriptNotFound {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        formatter
            .write_str("Could not locate script in scripts directory. Refer to the documentation.")
    }
}

#[derive(Debug)]
pub struct LoadScriptError;

impl Error for LoadScriptError {}

impl Display for LoadScriptError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        formatter.write_str("Could not load script. Refer to the documentation.")
    }
}
