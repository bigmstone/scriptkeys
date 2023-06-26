use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use {
    anyhow::{Error, Result},
    log::{debug, error, info},
    notify::{Error as NotifyError, Event, RecommendedWatcher, RecursiveMode, Watcher},
    serde::Deserialize,
    tokio::sync::{
        broadcast::{channel as broadcast_channel, Sender},
        mpsc::{channel, Receiver},
        Mutex,
    },
};

use crate::{
    constants::{CONFIG_FILE_NAMES, CONFIG_FILE_PATHS},
    device::Devices,
    errors::ConfigPathNotFound,
    helper::parse_path,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConfigEvent {
    Device,
    Mapping,
    Script,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Mapping {
    pub key: u32,
    pub script: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub device: Devices,
    pub mappings: Vec<Mapping>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Arc<Mutex<Self>>> {
        let config_text = fs::read_to_string(path)?;
        let config = Arc::new(Mutex::new(toml::from_str(&config_text)?));

        Ok(config)
    }

    pub async fn update(&mut self) -> Result<Vec<ConfigEvent>> {
        let path = match find_config() {
            Some(full_path) => full_path,
            None => return Err(Error::new(ConfigPathNotFound)),
        };

        let mut config_events = vec![];

        let config: Config = toml::from_str(&fs::read_to_string(path)?)?;

        if config.device != self.device {
            config_events.push(ConfigEvent::Device);
        }

        if !config.mappings.eq(&self.mappings) {
            config_events.push(ConfigEvent::Mapping);
        }

        *self = config;

        Ok(config_events)
    }
}

pub struct ConfigWatcher {
    pub config: Arc<Mutex<Config>>,
    pub config_event: Sender<ConfigEvent>,
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub async fn new() -> Result<Self> {
        let path = match find_config() {
            Some(full_path) => full_path,
            None => return Err(Error::new(ConfigPathNotFound)),
        };

        let config = Config::new(&path)?;

        let (ce_tx, _ce_rx) = broadcast_channel::<ConfigEvent>(32);
        let (tx, rx) = channel::<Result<Event, NotifyError>>(32);

        let mut watcher = notify::recommended_watcher(
            move |res: Result<Event, NotifyError>| {
                if tx.blocking_send(res).is_err() {}
            },
        )?;

        tokio::spawn(config_watcher(config.clone(), rx, ce_tx.clone()));

        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        Ok(Self {
            config,
            config_event: ce_tx,
            _watcher: watcher,
        })
    }
}

fn find_config() -> Option<PathBuf> {
    let file_names: Vec<PathBuf> = CONFIG_FILE_NAMES.iter().map(|s| parse_path(s)).collect();
    let paths: Vec<PathBuf> = CONFIG_FILE_PATHS.iter().map(|s| parse_path(s)).collect();

    for path in paths.iter() {
        for file_name in file_names.iter() {
            let full_path_option = path.join(file_name);
            if full_path_option.exists() {
                return Some(full_path_option);
            }
        }
    }

    None
}

async fn config_watcher(
    config: Arc<Mutex<Config>>,
    mut rx: Receiver<Result<Event, NotifyError>>,
    tx: Sender<ConfigEvent>,
) {
    loop {
        if let Some(Ok(event)) = rx.recv().await {
            if event.kind
                == notify::EventKind::Modify(notify::event::ModifyKind::Data(
                    notify::event::DataChange::Content,
                ))
            {
                let mut conf = config.lock().await;
                if let Ok(events) = conf.update().await {
                    info!("Updating Config file");
                    for event in events {
                        info!("Config update event: {:?}", event);
                        if tx.send(event).is_err() {
                            error!("Cannot send Config update event to broadcast channel");
                        }
                    }
                } else {
                    debug!(
                        "Config file notify fired but not an update notification: {:?}",
                        event
                    );
                }
            }
        }
    }
}

