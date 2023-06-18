mod helper;

use std::{collections::HashMap, error::Error, fs, path::Path};

use {
    enigo::Key,
    rlua::{InitFlags, Lua, StdLib},
    tokio::sync::mpsc::{Receiver, Sender},
};

use crate::{
    config::Config,
    constants::SCRIPT_FILE_PATHS,
    device::{Action, Event},
    errors::{LoadScriptError, ScriptNotFound},
    helper::parse_path,
    EnigoCommand,
};

pub struct Script {
    lua: Lua,
    script_map: HashMap<u32, String>,
}

impl Script {
    pub fn new(config: &Config, enigo_tx: Sender<EnigoCommand>) -> Result<Self, Box<dyn Error>> {
        let lua = unsafe {
            Lua::unsafe_new_with_flags(
                StdLib::ALL_NO_DEBUG,
                InitFlags::DEFAULT - InitFlags::REMOVE_LOADLIB,
            )
        };

        let mut scripts = vec![];
        let mut script_map = HashMap::new();

        for mapping in &config.mappings {
            let mut script = None;

            for path_str in SCRIPT_FILE_PATHS {
                let path = parse_path(path_str);
                let name = Path::new(&mapping.script);
                let full_path = path.join(name);

                if full_path.exists() {
                    if let Ok(script_string) = fs::read_to_string(full_path) {
                        script = Some(script_string);
                    } else {
                        return Err(Box::new(LoadScriptError));
                    }
                }
            }

            if let Some(script) = script {
                scripts.push(script);
            } else {
                return Err(Box::new(ScriptNotFound));
            }

            let name = Path::new(&mapping.script).file_stem().unwrap();
            script_map.insert(mapping.key, String::from(name.to_str().unwrap()));
        }

        lua.context(|lua_ctx| -> Result<(), Box<dyn Error>> {
            let globals = lua_ctx.globals();

            define_keys(enigo_tx.clone(), &lua_ctx, &globals)?;
            define_raw_keys(enigo_tx, &lua_ctx, &globals)?;

            for script in scripts {
                lua_ctx.load(&script).exec()?;
            }

            Ok(())
        })?;

        Ok(Self { lua, script_map })
    }

    pub async fn script_loop(self, mut rx: Receiver<Event>) {
        loop {
            let event = rx.recv().await.unwrap();

            if let Some(table_name) = self.script_map.get(&event.key) {
                let method = match event.action {
                    Action::Press => format!("{}.Press", table_name),
                    Action::Release => format!("{}.Release", table_name),
                };

                match self.lua.context(|lua_ctx| -> Result<(), Box<dyn Error>> {
                    lua_ctx.load(&format!("{}()", method)).exec()?;
                    Ok(())
                }) {
                    Ok(()) => {}
                    Err(err) => eprintln!("Error running script:\n{}", err),
                }
            }
        }
    }
}

fn define_keys<'a>(
    enigo_tx: Sender<EnigoCommand>,
    lua_ctx: &rlua::Context<'a>,
    globals: &rlua::Table<'a>,
) -> Result<(), Box<dyn Error>> {
    let enigo_copy = enigo_tx.clone();
    let key_click = lua_ctx.create_function(move |_lua_ctx, val: String| {
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
    let key_press = lua_ctx.create_function(move |_lua_ctx, val: String| {
        let enigo_copy = enigo_copy.clone();
        tokio::spawn(async move {
            enigo_copy
                .send(EnigoCommand::KeyDown(helper::map_str_to_key(&val)))
                .await
        });
        Ok(())
    })?;
    globals.set("keyDown", key_press)?;

    let key_release = lua_ctx.create_function(move |_lua_ctx, val: String| {
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

fn define_raw_keys<'a>(
    enigo_tx: Sender<EnigoCommand>,
    lua_ctx: &rlua::Context<'a>,
    globals: &rlua::Table<'a>,
) -> Result<(), Box<dyn Error>> {
    let enigo_copy = enigo_tx.clone();
    let key_click = lua_ctx.create_function(move |_lua_ctx, val: u16| {
        let enigo_copy = enigo_copy.clone();
        tokio::spawn(async move { enigo_copy.send(EnigoCommand::KeyClick(Key::Raw(val))).await });
        Ok(())
    })?;
    globals.set("rawKeyClick", key_click)?;

    let enigo_copy = enigo_tx.clone();
    let key_press = lua_ctx.create_function(move |_lua_ctx, val: u16| {
        let enigo_copy = enigo_copy.clone();
        tokio::spawn(async move { enigo_copy.send(EnigoCommand::KeyDown(Key::Raw(val))).await });
        Ok(())
    })?;
    globals.set("rawKeyDown", key_press)?;

    let key_release = lua_ctx.create_function(move |_lua_ctx, val: u16| {
        let enigo_tx = enigo_tx.clone();
        tokio::spawn(async move { enigo_tx.send(EnigoCommand::KeyUp(Key::Raw(val))).await });
        Ok(())
    })?;
    globals.set("rawKeyUp", key_release)?;

    Ok(())
}
