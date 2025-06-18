use std::{
    fs::{self, File},
    sync::mpsc::{self, Receiver, Sender},
};

use mlua::{Function, Lua, Value};

pub trait PluginLoader {}

#[derive(Debug, Clone)]
pub enum PlugLoaders {
    LuaL(LuaLoader),
}

#[derive(Debug, Clone)]
pub enum LoaderSysState {
    Running,
    Disabled,
    Panicked,
}

#[derive(Debug, Clone)]
pub struct LuaLoader {
    pub plugins: Vec<LuaPlugin>,
    pub state: LoaderSysState,
}

impl LuaLoader {
    pub fn new() -> LuaLoader {
        let plugs: Vec<LuaPlugin> = Vec::new();
        let state = LoaderSysState::Running;

        LuaLoader {
            plugins: plugs,
            state: state,
        }
    }
    pub fn load_plug(&mut self, path: String, tx: Sender<PluginMessage>) -> Result<(), String> {
        let mut plug = LuaPlugin::new();
        let plug_buf: String = match fs::read_to_string(path) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = "FS ERR: ".to_string() + &e.to_string();
                return Err(err_msg);
            }
        };
        match plug.lua.load(&plug_buf).exec() {
            Ok(_) => {}
            Err(e) => {
                let err_msg = "LUA ERR: ".to_string() + &e.to_string();
                return Err(err_msg);
            }
        }
        let globals = plug.lua.globals();
        plug.name = globals
            .get("PLUGIN_NAME")
            .unwrap_or("Unnamed plugin".to_string());
        plug.author = globals
            .get("PLUGIN_AUTHOR")
            .unwrap_or("Unknown author".to_string());
        plug.desc = globals
            .get("PLUGIN_DESC")
            .unwrap_or("No description providen".to_string());
        plug.version = globals
            .get("PLUGIN_VERSION")
            .unwrap_or("v1.0.0".to_string());

        plug.load_defaults(tx.clone());

        let init_func: Value = globals.get("onInit").unwrap();
        match init_func {
            Value::Function(lf) => {
                let init_func_res = lf.call::<()>(());
                if let Err(e) = init_func_res {
                    tx.send(PluginMessage::Error(e.to_string()));
                }
            }
            _ => {}
        }

        self.plugins.push(plug);

        Ok(())
    }
    pub fn unload_plugin_ref(&mut self, plug: &LuaPlugin) -> Result<(), String> {
        for (i, p) in self.plugins.iter().enumerate() {
            if std::ptr::eq(p, plug) {
                self.plugins.remove(i);
                return Ok(());
            }
        }
        return Err("Specified plugin could not be found".to_string());
    }
    pub fn unload_plugin_ind(&mut self, ind: usize) {
        if (ind >= self.plugins.len()) {
            return;
        }
        self.plugins.remove(ind);
    }
    pub fn find_plug_by_name_ref(&self, name: String) -> Option<&LuaPlugin> {
        for p in &self.plugins {
            if p.name == name {
                return Some(p);
            }
        }
        None
    }
    pub fn find_plug_ind_by_name(&mut self, name: String) -> Option<usize> {
        for (i, p) in self.plugins.iter().enumerate() {
            if p.name == name {
                return Some(i);
            }
        }
        None
    }
}

impl PluginLoader for LuaLoader {}

#[derive(Debug, Clone)]
pub enum PlugCom {
    StatusMsg(String),
}

#[derive(Debug, Clone)]
pub enum PluginMessage {
    Command(PlugCom),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct LuaPlugin {
    lua: Lua,
    pub name: String,
    pub author: String,
    pub version: String,
    pub desc: String,
}

impl LuaPlugin {
    fn new() -> LuaPlugin {
        LuaPlugin {
            lua: Lua::new(),
            name: String::new(),
            author: String::new(),
            version: String::new(),
            desc: String::new(),
        }
    }
    fn load_defaults(&mut self, tx: Sender<PluginMessage>) {
        let globals = self.lua.globals();

        let tx_status = tx.clone();
        let print_stat_func = match self.lua.create_function(move |_, msg: String| {
            tx_status
                .send(PluginMessage::Command(PlugCom::StatusMsg(msg)))
                .map_err(|e| mlua::Error::RuntimeError(format!("Failed to send message: {}", e)))?;
            Ok(())
        }) {
            Ok(lf) => lf,
            Err(e) => {
                tx.send(PluginMessage::Error(e.to_string()));
                return;
            }
        };
        globals.set("frevi_status_msg", print_stat_func);
    }
}
