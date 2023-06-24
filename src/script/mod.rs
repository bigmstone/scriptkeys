mod helper;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use {
    anyhow::{Error, Result},
    enigo::Key,
    mlua::Lua,
    notify::{
        Error as NotifyError, Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher,
    },
    tokio::sync::{
        broadcast::Receiver as BroadcastReceiver,
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
};

use crate::{
    config::{Config, ConfigEvent, Mapping},
    constants::SCRIPT_FILE_PATHS,
    device::{Action, Event},
    errors::{LoadScriptError, ScriptNotFound},
    helper::parse_path,
    EnigoCommand,
};

pub struct Script {
    lua: Lua,
    script_map: HashMap<u32, String>,
    watcher: RecommendedWatcher,
}

impl Script {
    pub async fn new(
        config: Arc<Mutex<Config>>,
        enigo_tx: Sender<EnigoCommand>,
    ) -> Result<Arc<Mutex<Self>>> {
        let (tx, rx) = channel::<Result<NotifyEvent, NotifyError>>(32);

        let watcher = notify::recommended_watcher(
            move |res: Result<NotifyEvent, NotifyError>| {
                if tx.blocking_send(res).is_err() {}
            },
        )?;

        let lua = unsafe { Lua::unsafe_new() };

        let script = Arc::new(Mutex::new(Self {
            lua,
            script_map: HashMap::new(),
            watcher,
        }));
        {
            let mut script = script.lock().await;
            {
                let globals = script.lua.globals();

                define_keys(enigo_tx.clone(), &script.lua, &globals)?;
                define_raw_keys(enigo_tx, &script.lua, &globals)?;
            }

            script.load_mapping(&*config.lock().await)?;
        }

        tokio::spawn(script_watcher(script.clone(), rx));

        Ok(script)
    }

    pub fn load_mapping(&mut self, conf: &Config) -> Result<()> {
        for mapping in &conf.mappings {
            if let Some(full_path) = find_script(&mapping.script) {
                match self.load_script_mapping(&full_path, mapping) {
                    Ok(()) => break,
                    Err(err) => {
                        if err.downcast_ref::<ScriptNotFound>().is_some() {
                            continue;
                        } else {
                            return Err(err);
                        }
                    }
                };
            } else {
                return Err(Error::new(ScriptNotFound));
            }
        }

        Ok(())
    }

    pub fn load_script_mapping(&mut self, path: &Path, mapping: &Mapping) -> Result<()> {
        if path.exists() {
            if let Ok(script) = fs::read_to_string(path) {
                self.lua.load(&script).exec()?;
                let name = Path::new(&mapping.script).file_stem().unwrap();
                self.script_map
                    .insert(mapping.key, String::from(name.to_str().unwrap()));
                self.watcher.watch(path, RecursiveMode::NonRecursive)?;
                Ok(())
            } else {
                Err(Error::new(LoadScriptError))
            }
        } else {
            Err(Error::new(ScriptNotFound))
        }
    }

    pub fn load_script(&mut self, path: &Path) -> Result<()> {
        if path.exists() {
            if let Ok(script) = fs::read_to_string(path) {
                self.lua.load(&script).exec()?;

                Ok(())
            } else {
                Err(Error::new(LoadScriptError))
            }
        } else {
            Err(Error::new(ScriptNotFound))
        }
    }
}

pub async fn script_loop(script: Arc<Mutex<Script>>, mut rx: Receiver<Event>) {
    loop {
        let event = rx.recv().await.unwrap();

        let script = script.lock().await;
        if let Some(table_name) = script.script_map.get(&event.key) {
            let method = match event.action {
                Action::Press => format!("{}.Press", table_name),
                Action::Release => format!("{}.Release", table_name),
            };

            script.lua.load(&format!("{}()", method)).exec().unwrap();
        }
    }
}

pub async fn config_update_handler(
    script: Arc<Mutex<Script>>,
    config: Arc<Mutex<Config>>,
    mut rx: BroadcastReceiver<ConfigEvent>,
) {
    loop {
        let event = rx.recv().await;

        if let Ok(event) = event {
            if event == ConfigEvent::Mapping {
                let mut script = script.lock().await;
                let conf = config.lock().await;

                if script.load_mapping(&conf).is_err() {
                    // TODO: Better Error Handling
                }
            }
        }
    }
}

fn find_script(script: &str) -> Option<PathBuf> {
    for path_str in SCRIPT_FILE_PATHS {
        let path = parse_path(path_str);
        let name = Path::new(script);
        let full_path = path.join(name);
        if full_path.exists() {
            return Some(full_path);
        }
    }

    None
}

fn define_keys(enigo_tx: Sender<EnigoCommand>, lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let enigo_copy = enigo_tx.clone();
    let key_click = lua.create_function(move |_lua, val: String| {
        let enigo_copy = enigo_copy.clone();
        tokio::spawn(async move {
            enigo_copy
                .send(EnigoCommand::KeyClick(helper::map_str_to_key(&val)))
                .await
        });
        Ok(())
    })?;
    globals.set("keyClick", key_click)?;

    let enigo_copy = enigo_tx.clone();
    let key_press = lua.create_function(move |_lua, val: String| {
        let enigo_copy = enigo_copy.clone();
        tokio::spawn(async move {
            enigo_copy
                .send(EnigoCommand::KeyDown(helper::map_str_to_key(&val)))
                .await
        });
        Ok(())
    })?;
    globals.set("keyDown", key_press)?;

    let key_release = lua.create_function(move |_lua, val: String| {
        let enigo_tx = enigo_tx.clone();
        tokio::spawn(async move {
            enigo_tx
                .send(EnigoCommand::KeyDown(helper::map_str_to_key(&val)))
                .await
        });
        Ok(())
    })?;
    globals.set("keyUp", key_release)?;

    Ok(())
}

fn define_raw_keys(enigo_tx: Sender<EnigoCommand>, lua: &Lua, globals: &mlua::Table) -> Result<()> {
    let enigo_copy = enigo_tx.clone();
    let key_click = lua.create_function(move |_lua, val: u16| {
        let enigo_copy = enigo_copy.clone();
        tokio::spawn(async move { enigo_copy.send(EnigoCommand::KeyClick(Key::Raw(val))).await });
        Ok(())
    })?;
    globals.set("rawKeyClick", key_click)?;

    let enigo_copy = enigo_tx.clone();
    let key_press = lua.create_function(move |_lua, val: u16| {
        let enigo_copy = enigo_copy.clone();
        tokio::spawn(async move { enigo_copy.send(EnigoCommand::KeyDown(Key::Raw(val))).await });
        Ok(())
    })?;
    globals.set("rawKeyDown", key_press)?;

    let key_release = lua.create_function(move |_lua, val: u16| {
        let enigo_tx = enigo_tx.clone();
        tokio::spawn(async move { enigo_tx.send(EnigoCommand::KeyUp(Key::Raw(val))).await });
        Ok(())
    })?;
    globals.set("rawKeyUp", key_release)?;

    Ok(())
}

async fn script_watcher(
    script: Arc<Mutex<Script>>,
    mut rx: Receiver<Result<NotifyEvent, NotifyError>>,
) {
    loop {
        if let Some(Ok(event)) = rx.recv().await {
            if event.kind
                == notify::EventKind::Modify(notify::event::ModifyKind::Data(
                    notify::event::DataChange::Content,
                ))
            {
                let mut script = script.lock().await;

                if let Some(path) = event.paths.first() {
                    if script.load_script(path).is_err() {
                        // TODO: Better Error Handling
                    }
                }
            } else {
                // TODO: Better Error Handling
            }
        }
    }
}
