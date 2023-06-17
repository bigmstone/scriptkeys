use std::{error::Error, fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    constants::{CONFIG_FILE_NAMES, CONFIG_FILE_PATHS},
    device::{Action, Devices},
    errors::ConfigPathNotFound,
    helper::parse_path,
};

#[derive(Deserialize)]
pub struct Mapping {
    pub key: u32,
    pub action: Action,
    pub script: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub device: Devices,
    pub mappings: Vec<Mapping>,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let file_names: Vec<PathBuf> = CONFIG_FILE_NAMES.iter().map(|s| parse_path(s)).collect();
        let paths: Vec<PathBuf> = CONFIG_FILE_PATHS.iter().map(|s| parse_path(s)).collect();

        let mut full_path = None;

        for path in paths.iter() {
            for file_name in file_names.iter() {
                let full_path_option = path.join(file_name);
                if full_path_option.exists() {
                    full_path = Some(full_path_option);
                }
            }
        }

        let path = match full_path {
            Some(full_path) => full_path,
            None => return Err(Box::new(ConfigPathNotFound)),
        };

        let config_text = fs::read_to_string(path)?;

        Ok(toml::from_str(&config_text)?)
    }
}
