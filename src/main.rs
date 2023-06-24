use std::{
    error::Error,
    path::{Path, PathBuf},
};

use {
    anyhow::Result,
    enigo::{Enigo, KeyboardControllable},
    log::info,
    log::LevelFilter,
    log4rs::{
        append::{console::ConsoleAppender, file::FileAppender},
        config::{Appender, Config, Logger, Root},
        encode::pattern::PatternEncoder,
        Handle,
    },
    tokio::{sync::mpsc, task},
};

use scriptkeys::{
    config::ConfigWatcher,
    constants::{LOG_FILE_NAMES, LOG_FILE_PATHS},
    device::{derive_device, Event},
    script::{config_update_handler, script_loop, Script},
    EnigoCommand,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _handle = setup_logging();

    info!("Starting scriptkeys");

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

fn setup_logging() -> Result<Handle> {
    let stdout = ConsoleAppender::builder().build();
    let mut config_builder =
        Config::builder().appender(Appender::builder().build("stdout", Box::new(stdout)));
    let mut root_builder = Root::builder().appender("stdout");

    if let Some(log_path) = find_log_location() {
        let file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build(log_path)?;
        config_builder =
            config_builder.appender(Appender::builder().build("file", Box::new(file_appender)));
        root_builder = root_builder.appender("file");
    } else {
        eprintln!("Couldn't find location for logging to file. STDOUT will only be supported.");
    }

    let config = config_builder
        .logger(Logger::builder().build("scriptkeys", LevelFilter::Info))
        .build(root_builder.build(LevelFilter::Warn))?;

    Ok(log4rs::init_config(config)?)
}

fn find_log_location() -> Option<PathBuf> {
    for path in LOG_FILE_PATHS {
        let path = Path::new(path);
        for name in LOG_FILE_NAMES {
            let name = Path::new(name);

            if path.is_dir() {
                return Some(path.join(name));
            }
        }
    }

    None
}
