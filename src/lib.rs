pub mod config;
pub mod constants;
pub mod device;
pub mod errors;
pub mod helper;
pub mod script;

#[derive(Debug)]
pub enum EnigoCommand {
    KeyClick(enigo::Key),
    KeyDown(enigo::Key),
    KeyUp(enigo::Key),
}
