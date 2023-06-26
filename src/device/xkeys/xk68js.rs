// Below is the mapping of bytes...the less relevant bytes I didn't inspect closely
//
// 00000000 Unused
// 00000000 Unused
// 00000001 Column 1
// 00000001 Column 2
// 00000001 Column 3
// 00000001 Column 4
// 00000001 Column 5
// 10000001 Column 6
// 00000001 Column 7
// 00000001 Column 8
// 00000001 Column 9
// 00000001 Column 10
// 00000000 Different representation of rotor
// 01000000 Different representation of rotor
// 00000000 Different representation of rotor
// 00000000 Different representation of rotor
// 00000000 Continuous Inactive 00000001 Continuous Right 11111111 Continuous Left
// 11111001 Rotor Left 00000111 Rotor Right
// 00000001
// 01101001 Appears to count up
// 00101000 Appears to count up
// 10000011 Appears to count up
// 00000000 Unused
// ...x41 Unused Bytes

use std::collections::HashMap;

use {
    anyhow::{Error, Result},
    hidapi::{HidApi, HidDevice},
    log::trace,
    serde::Deserialize,
    tokio::sync::mpsc::Sender,
};

use crate::{
    device::{Action, Device, Event},
    errors::DeviceNotFound,
};

#[derive(Debug)]
pub struct State {}

#[derive(Deserialize, Debug)]
pub enum InterfaceType {
    Button(bool),
    Wheel(i32),
}

impl Default for InterfaceType {
    fn default() -> Self {
        Self::Button(false)
    }
}

#[derive(Deserialize)]
pub struct XK68JS {
    pub state: HashMap<u32, InterfaceType>,
}

impl Default for XK68JS {
    fn default() -> Self {
        let mut state = HashMap::new();

        let columns = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        for i in columns {
            for j in 0..8 {
                let id = ((i - 2) * 8) + j;
                state.insert(id, InterfaceType::Button(false));
            }
        }
        Self { state }
    }
}

impl XK68JS {
    pub fn process_buffer(&mut self, data: &[u8]) -> Vec<Event> {
        let mut change_buffer = vec![];

        let columns = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

        for i in columns {
            for j in 0..8 {
                let id = ((i - 2) * 8) + j;
                let new_value = Self::bit_set(data[i as usize], j as i8);
                if let Some(interface) = self.state.get_mut(&id) {
                    if let InterfaceType::Button(value) = interface {
                        if value != &new_value {
                            change_buffer.push(Event {
                                key: id,
                                action: match new_value {
                                    true => Action::Press,
                                    false => Action::Release,
                                },
                            });
                        }
                        *interface = InterfaceType::Button(new_value);
                    }
                } else {
                    self.state.insert(id, InterfaceType::Button(new_value));
                    change_buffer.push(Event {
                        key: id,
                        action: match new_value {
                            true => Action::Press,
                            false => Action::Release,
                        },
                    });
                }
            }
        }

        change_buffer
    }

    fn bit_set(byte: u8, bit: i8) -> bool {
        (byte & (1 << bit)) != 0
    }

    fn get_device(&self) -> Result<HidDevice, Error> {
        let api = HidApi::new()?;
        let mut xkeys_device = None;

        for device in api.device_list() {
            if device.vendor_id() == 0x05f3
                && device.product_id() == 0x045a
                && device.interface_number() == 0
            {
                xkeys_device = Some(device.open_device(&api)?);
                break;
            }
        }

        let device = match xkeys_device {
            Some(device) => device,
            None => return Err(Error::new(DeviceNotFound)),
        };

        Ok(device)
    }
}

impl Device for XK68JS {
    fn read_loop(&mut self, tx: Sender<Event>) {
        let device = self.get_device().unwrap();

        loop {
            let mut buf: Vec<u8> = vec![0; 64];
            device.read(&mut buf).unwrap();

            let events = self.process_buffer(&buf);

            let txc = tx.clone();
            tokio::spawn(async move {
                for event in events {
                    trace!("Sending event: {:?}", event);
                    txc.send(event).await.unwrap();
                }
            });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_deseralize() {
        let data = vec![
            0b00000000, 0b00000000, 0b00000011, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b10000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000001, 0b01101110, 0b00011111,
            0b10100000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000,
        ];

        let mut device = XK68JS::default();

        device.process_buffer(&data);

        for (i, expected_value) in [(0, true), (1, true), (2, false), (7, false)] {
            if let InterfaceType::Button(value) = device.state[&(i as u32)] {
                assert_eq!(value, expected_value);
            }
        }
    }
}

