use std::error::Error;

use {
    enigo::{Enigo, KeyboardControllable},
    tokio::{sync::mpsc, task},
};

use scriptkeys::{
    config::Config,
    device::{xkeys::xk68js::XK68JS, Device, Devices, Event},
    script::Script,
    EnigoCommand,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new()?;

    let device = match config.device {
        Devices::XK68JS => XK68JS::default(),
    };

    let (tx, rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel(32);
    let (enigo_tx, mut enigo_rx): (mpsc::Sender<EnigoCommand>, mpsc::Receiver<EnigoCommand>) =
        mpsc::channel(32);

    let script = Script::new(&config, enigo_tx)?;

    task::spawn_blocking(move || {
        device.read_loop(tx);
    });

    task::spawn(async move {
        script.script_loop(rx).await;
    });

    let mut enigo = Enigo::new();
    while let Some(command) = enigo_rx.recv().await {
        match command {
            EnigoCommand::KeyClick(key) => enigo.key_click(key),
            EnigoCommand::KeyDown(key) => enigo.key_down(key),
            EnigoCommand::KeyUp(key) => enigo.key_up(key),
        }
    }

    Ok(())
}
