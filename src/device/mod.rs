pub mod xkeys;

use std::error::Error;

use {hidapi::HidDevice, serde::Deserialize, tokio::sync::mpsc::Sender};

#[derive(Deserialize, Debug)]
pub enum Action {
    Press,
    Release,
}

#[derive(Debug)]
pub struct Event {
    pub key: u32,
    pub action: Action,
}

#[derive(Deserialize)]
pub enum Devices {
    XK68JS,
}

pub trait Device {
    fn get_device(&self) -> Result<HidDevice, Box<dyn Error>>;
    fn read_loop(self, tx: Sender<Event>);
}
