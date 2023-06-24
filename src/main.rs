use std::error::Error;

use {
    anyhow::Result,
    enigo::{Enigo, KeyboardControllable},
    tokio::{sync::mpsc, task},
};

use scriptkeys::{
    config::ConfigWatcher,
    device::{derive_device, Event},
    script::{config_update_handler, script_loop, Script},
    EnigoCommand,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel(32);
    let (enigo_tx, mut enigo_rx): (mpsc::Sender<EnigoCommand>, mpsc::Receiver<EnigoCommand>) =
        mpsc::channel(32);

    let config_watcher = ConfigWatcher::new().await?;

    {
        let conf = config_watcher.config.lock().await;
        let mut device = derive_device(&conf.device)?;

        task::spawn_blocking(move || {
            device.read_loop(tx);
        });
    }

    let script = Script::new(config_watcher.config.clone(), enigo_tx).await?;

    let script_clone = script.clone();
    task::spawn(async move {
        script_loop(script_clone, rx).await;
    });

    let script_clone = script.clone();
    task::spawn(async move {
        let config_event_reader = config_watcher.config_event.subscribe();
        config_update_handler(
            script_clone,
            config_watcher.config.clone(),
            config_event_reader,
        )
        .await;
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
