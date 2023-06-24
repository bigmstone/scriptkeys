pub mod xkeys;

use std::{thread, time::Duration};

use {anyhow::Result, serde::Deserialize, tokio::sync::mpsc::Sender};

#[derive(Deserialize, Debug, PartialEq)]
pub enum Action {
    Press,
    Release,
}

#[derive(Debug)]
pub struct Event {
    pub key: u32,
    pub action: Action,
}

#[derive(Deserialize, PartialEq)]
pub enum Devices {
    XK68JS,
    Dummy,
}

pub fn derive_device(device: &Devices) -> Result<Box<dyn Device + Send>> {
    match device {
        Devices::XK68JS => Ok(Box::<xkeys::xk68js::XK68JS>::default()),
        Devices::Dummy => Ok(Box::new(Dummy {
            duration: Duration::from_secs(10),
        })),
    }
}

pub trait Device {
    fn read_loop(&mut self, tx: Sender<Event>);
}

pub struct Dummy {
    pub duration: Duration,
}

impl Device for Dummy {
    fn read_loop(&mut self, tx: Sender<Event>) {
        loop {
            let txc = tx.clone();
            tokio::spawn(async move {
                let event = Event {
                    key: 0,
                    action: Action::Press,
                };

                txc.send(event).await.unwrap();
            });
            thread::sleep(self.duration);
        }
    }
}
