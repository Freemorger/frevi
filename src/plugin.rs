use std::fs::{self, File};

use mlua::Lua;

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
    pub fn load_plug(&mut self, path: String) -> Result<(), String> {
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

        self.plugins.push(plug);

        Ok(())
    }
}

impl PluginLoader for LuaLoader {}

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
}
