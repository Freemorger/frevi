use core::panic;
use std::{
    char,
    collections::HashMap,
    fmt::format,
    sync::mpsc::{self, Receiver, Sender},
};

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEvent, MouseEventKind};
use crossterm::terminal::{ScrollDown, ScrollUp};
use mlua::Function;

use crate::{commands, logger::LogLevel, plugin::PluginMessage};
use crate::{
    dotfiles::FreviConfig,
    plugin::{LuaLoader, PlugLoaders, PluginLoader},
};
use crate::{edits::Edit, plugin::PlugCom};
use crate::{logger::Logger, tabs::Tab};

type RustHandler = fn(&mut App, Vec<String>);
#[derive(Debug, Clone)]
pub enum CommandHandler {
    Rust(RustHandler),
    Lua(Function, usize), // usize for plugin id.
}

#[derive(Debug)]
pub struct App {
    pub insert_mode: bool,
    pub running: bool,
    pub left_area_open: bool,
    pub left_area_used: bool,
    pub left_area: Tab,
    pub cursor_pos_xy: (u16, u16),
    pub command_buf: String,
    pub commands: HashMap<String, CommandHandler>,
    pub aliases: HashMap<String, Vec<String>>,
    pub command_hist: Vec<Vec<String>>,
    pub status_message: bool,
    pub tabs: Vec<Tab>,
    pub cur_tab: usize,
    pub version: String,
    pub hist_ctr: usize,
    pub plugin_subsys: PlugLoaders,
    pub plugin_tx: Sender<PluginMessage>,
    pub plugin_rx: Receiver<PluginMessage>,
    pub config: FreviConfig,
    pub logger: Logger,
}

impl App {
    // Creates App object with default plugin system (lualoader)
    pub fn new() -> App {
        let ins_mod: bool = false;
        let run: bool = true;
        let left_open: bool = false;
        let cpos_xy: (u16, u16) = (0, 0);
        let com_buf: String = String::new();
        let coms: HashMap<String, CommandHandler> = HashMap::new();
        let stat_msg: bool = false;
        let tabsv: Vec<Tab> = vec![Tab::new(None)];
        let curtab: usize = 0;

        let mut build_type: String = String::new();
        if (cfg!(debug_assertions)) {
            build_type = "debug".to_string();
        } else {
            build_type = "release".to_string();
        }
        let vers: &str = env!("CARGO_PKG_VERSION");
        let full_vers = format!("{} {}", vers, build_type);

        let com_aliases: HashMap<String, Vec<String>> = HashMap::new();
        let com_hist: Vec<Vec<String>> = Vec::new();
        let hist_c: usize = 0;

        let mut frevi_cfg = FreviConfig::new();
        let frevi_cfg_load_res = frevi_cfg.read_cfg();
        let log_path = frevi_cfg.cfg_path.clone().join("latest.log");
        let str_log_path = log_path.to_string_lossy().to_string();

        let mut logger: Logger = match Logger::new(str_log_path.clone()) {
            Ok(l) => l,
            Err(e) => {
                panic!("{}", &e);
            }
        };

        let (tx, rx) = mpsc::channel();

        let left_area: Tab = Tab::new(None);
        let left_ar_us: bool = false;
        let mut lua_load = LuaLoader::new();
        if let Err(errors) = lua_load.load_plugs_lines(frevi_cfg.autoplugs.clone(), tx.clone()) {
            for item in errors {
                logger.log_msg(LogLevel::PluginFault, item.clone());
            }
        }
        let pl_sys: PlugLoaders = PlugLoaders::LuaL(lua_load);

        let mut app = App {
            insert_mode: ins_mod,
            running: run,
            left_area_open: left_open,
            left_area: left_area,
            cursor_pos_xy: cpos_xy,
            command_buf: com_buf,
            commands: coms,
            status_message: stat_msg,
            tabs: tabsv,
            cur_tab: curtab,
            version: full_vers,
            command_hist: com_hist,
            aliases: com_aliases,
            hist_ctr: hist_c,
            left_area_used: left_ar_us,
            plugin_subsys: pl_sys,
            plugin_tx: tx,
            plugin_rx: rx,
            config: frevi_cfg,
            logger: logger,
        };
        app.gen_hashmap_com();
        app.throw_status_message(str_log_path);
        app
    }

    pub fn handle_input(&mut self, event: Event) {
        match event {
            Event::Mouse(m_ev) => match m_ev.kind {
                MouseEventKind::ScrollUp => {
                    let cur_tab_opt = self.tabs.get_mut(self.cur_tab);
                    let cur_tab = match cur_tab_opt {
                        Some(tab) => tab,
                        None => {
                            self.throw_status_message(format!(
                                "E: Can't get tab with indice {}",
                                self.cur_tab
                            ));
                            return;
                        }
                    };
                    cur_tab.scroll_offset = cur_tab.scroll_offset.saturating_sub(1);
                    return;
                }
                MouseEventKind::ScrollDown => {
                    let cur_tab_opt = self.tabs.get_mut(self.cur_tab);
                    let cur_tab = match cur_tab_opt {
                        Some(tab) => tab,
                        None => {
                            self.throw_status_message(format!(
                                "E: Can't get tab with indice {}",
                                self.cur_tab
                            ));
                            return;
                        }
                    };
                    cur_tab.scroll_offset = cur_tab.scroll_offset.saturating_add(1);
                    return;
                }
                _ => {}
            },
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Insert | KeyCode::Esc => {
                    self.insert_mode = !self.insert_mode;
                }
                KeyCode::Char(ch) => {
                    if !self.insert_mode {
                        if self.status_message {
                            self.command_buf = "".to_string();
                            self.command_buf.push(ch);
                            self.cursor_pos_xy.0 = 1;
                            self.status_message = !self.status_message;
                            return;
                        }
                        if self.cursor_pos_xy.0 as usize >= self.command_buf.len() {
                            match self.command_buf.len() {
                                0 => {
                                    self.cursor_pos_xy.0 = 0;
                                }
                                val => {
                                    self.cursor_pos_xy.0 = (val) as u16;
                                }
                            }
                        }
                        //self.command_buf.insert(self.cursor_pos_xy.0 as usize, ch);
                        let char_pos = self.cursor_pos_xy.0 as usize;
                        let byte_idx = self
                            .command_buf
                            .char_indices()
                            .nth(char_pos)
                            .map(|(idx, _)| idx)
                            .unwrap_or(self.command_buf.len()); // insert at end if position is out of bounds

                        self.command_buf.insert(byte_idx, ch);

                        self.cursor_pos_xy.0 += 1;
                        return;
                    }

                    self.insert_ch_tab_buf(ch);
                }
                KeyCode::Tab => {
                    if self.insert_mode {
                        self.insert_ch_tab_buf('\t');
                    }
                }
                KeyCode::Backspace => {
                    let tgt_line: &mut String;
                    let prev_line: &mut String;
                    if !self.insert_mode {
                        tgt_line = &mut self.command_buf;
                        if self.status_message {
                            self.command_buf = "".to_string();
                            self.status_message = !self.status_message;
                            return;
                        }
                    } else {
                        self.tab_backspace();
                        return;
                    }

                    let curpos_x_bordered = if self.cursor_pos_xy.0 == 0 {
                        self.cursor_pos_xy.0
                    } else {
                        self.cursor_pos_xy.0 - 1
                    };

                    // non ascii support stuff (should be extracted into function later)
                    let char_count = tgt_line.chars().count() as u16;
                    if curpos_x_bordered < char_count {
                        if let Some(idx) = tgt_line
                            .char_indices()
                            .nth(curpos_x_bordered as usize)
                            .map(|(i, _)| i)
                        {
                            tgt_line.remove(idx);
                            self.cursor_pos_xy.0 = curpos_x_bordered;
                        }
                    }
                }
                KeyCode::Left => {
                    if self.insert_mode {
                        self.move_cursor_hor(-1);
                        return;
                    }
                    if self.cursor_pos_xy.0 == 0 {
                        return;
                    }
                    self.cursor_pos_xy.0 -= 1;
                }
                KeyCode::Right => {
                    let tgt_str: &mut String;
                    if !self.insert_mode {
                        tgt_str = &mut self.command_buf;
                    } else {
                        self.move_cursor_hor(1);
                        return;
                    }

                    if self.cursor_pos_xy.0 as usize >= tgt_str.len() {
                        return;
                    }
                    self.cursor_pos_xy.0 += 1;
                }
                KeyCode::Up => {
                    if !self.insert_mode {
                        self.hist_ctr = self.hist_ctr.saturating_sub(1);
                        if !self.command_hist.is_empty() {
                            if let Some(s) = self.command_hist.get(self.hist_ctr.saturating_sub(1))
                            {
                                self.command_buf = s.join(" ");
                                self.status_message = false;
                            }
                        }
                        return;
                    }
                    self.move_cursor_vert(-1);
                }
                KeyCode::Down => {
                    if !self.insert_mode {
                        self.hist_ctr = self
                            .hist_ctr
                            .saturating_add(1)
                            .clamp(0, self.command_hist.len() + 1);
                        if self.hist_ctr >= self.command_hist.len() {
                            self.command_buf = "".to_string();
                            return;
                        }
                        if let Some(s) = self.command_hist.get(self.hist_ctr.saturating_add(1)) {
                            self.command_buf = s.join(" ");
                        }
                        return;
                    }
                    self.move_cursor_vert(1);
                }
                KeyCode::Enter => {
                    if !self.insert_mode {
                        self.parse_command();
                        return;
                    }

                    self.tab_newline();
                }
                KeyCode::PageUp => {
                    self.tab_update_scroll_delta(-10);
                }
                KeyCode::PageDown => {
                    self.tab_update_scroll_delta(10);
                }
                KeyCode::Home => {
                    self.tab_update_scroll(0);
                }
                KeyCode::End => {
                    self.tab_update_scroll(usize::MAX); // this will work due to clamp inside function.
                }
                KeyCode::F(num) => {
                    if num as usize > self.tabs.len() {
                        let newtab = Tab::new(None);
                        self.tabs.push(newtab);
                        self.cur_tab = self.tabs.len().saturating_sub(1);
                        return;
                    }
                    self.cur_tab = num
                        .saturating_sub(1)
                        .min(self.tabs.len().saturating_sub(1) as u8)
                        as usize;
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn recv_msg(&mut self) {
        while let Ok(msg) = self.plugin_rx.try_recv() {
            match msg {
                PluginMessage::Command(com) => match com {
                    PlugCom::StatusMsg(sm) => {
                        self.throw_status_message(sm);
                    }
                    _ => {}
                },
                PluginMessage::RegisterCommand(name, handlr, id) => {
                    self.commands.insert(name, CommandHandler::Lua(handlr, id));
                }
                _ => {}
            }
        }
    }

    fn move_cursor_vert(&mut self, delta: isize) {
        let cur_tab = &mut self.tabs[self.cur_tab];

        let new_y =
            ((cur_tab.cursor_xy.1 as isize) + delta).clamp(0, (cur_tab.buf.len() - 1) as isize);
        cur_tab.cursor_xy.1 = new_y as usize;
        let tgt_line: &String = &cur_tab.buf[new_y as usize];

        let new_x = cur_tab.cursor_xy.0.clamp(0, tgt_line.len());
        cur_tab.cursor_xy.0 = new_x;
    }

    fn move_cursor_hor(&mut self, delta: isize) {
        let cur_tab = &mut self.tabs[self.cur_tab];
        let line_y = cur_tab.cursor_xy.1;
        let tgt_line = &cur_tab.buf[line_y];

        let new_x = ((cur_tab.cursor_xy.0 as isize) + delta).clamp(0, (tgt_line.len()) as isize);
        cur_tab.cursor_xy.0 = new_x as usize;
    }

    fn tab_update_scroll(&mut self, new_scroll_offset: usize) {
        let cur_tab = &mut self.tabs[self.cur_tab];
        let max_scroll = cur_tab.buf.len().saturating_sub(1);
        cur_tab.scroll_offset = new_scroll_offset.clamp(0, max_scroll);
    }

    fn tab_update_scroll_delta(&mut self, delta: isize) {
        let cur_tab = &mut self.tabs[self.cur_tab];
        let max_scroll = cur_tab.buf.len().saturating_sub(1);
        let new_scroll = (cur_tab.scroll_offset as isize + delta).clamp(0, max_scroll as isize);
        cur_tab.scroll_offset = new_scroll as usize;
    }

    fn insert_ch_tab_buf(&mut self, ch: char) {
        let cur_tab: &mut Tab = match self.left_area_used {
            true => &mut self.left_area,
            false => &mut self.tabs[self.cur_tab],
        };
        let line_y = (cur_tab.scroll_offset + cur_tab.cursor_xy.1).clamp(0, cur_tab.buf.len());
        let x = cur_tab.cursor_xy.0.clamp(0, cur_tab.buf[line_y].len());

        let x_char = cur_tab
            .cursor_xy
            .0
            .clamp(0, cur_tab.buf[line_y].chars().count());
        let x_byte = cur_tab.buf[line_y]
            .char_indices()
            .nth(x_char)
            .map(|(i, _)| i)
            .unwrap_or(cur_tab.buf[line_y].len()); // for non-ascii
        cur_tab.buf[line_y].insert(x_byte, ch);
        cur_tab.cursor_xy.0 += 1;
        cur_tab.changed = true;
    }

    fn tab_newline(&mut self) {
        let cur_tab: &mut Tab = &mut self.tabs[self.cur_tab];
        let line_y = cur_tab.cursor_xy.1.clamp(0, cur_tab.buf.len());
        let line = &mut cur_tab.buf[line_y];

        // Clamp the cursor x to the number of chars, not bytes
        let x_chars = cur_tab.cursor_xy.0.clamp(0, line.chars().count());

        // byte offset
        let byte_index = line
            .char_indices()
            .nth(x_chars)
            .map(|(i, _)| i)
            .unwrap_or_else(|| line.len());

        let to_move: String = line[byte_index..].to_string();
        line.truncate(byte_index);
        cur_tab.buf.insert(line_y + 1, to_move);
        cur_tab.cursor_xy.0 = 0;
        cur_tab.cursor_xy.1 = line_y + 1;
        cur_tab.changed = true;
    }

    fn tab_backspace(&mut self) {
        let cur_tab: &mut Tab = &mut self.tabs[self.cur_tab];
        let line_y = (cur_tab.scroll_offset + cur_tab.cursor_xy.1).clamp(0, cur_tab.buf.len());
        let x_char = cur_tab
            .cursor_xy
            .0
            .clamp(0, cur_tab.buf[line_y].chars().count());
        let mut tgt_line = &mut cur_tab.buf[line_y];

        let (before, after) = cur_tab.buf.split_at_mut(cur_tab.cursor_xy.1 as usize);
        tgt_line = after.get_mut(0).expect("can't get cur line!");

        if cur_tab.cursor_xy.0 == 0 {
            if cur_tab.cursor_xy.1 == 0 {
                return;
            }
            let prev_line = before.last_mut().expect("can't get prev line");
            cur_tab.cursor_xy.0 = prev_line.chars().count();
            prev_line.push_str(tgt_line);
            cur_tab.buf.remove(cur_tab.cursor_xy.1 as usize);
            cur_tab.cursor_xy.1 -= 1;
            cur_tab.changed = true;
            return;
        }

        match x_char {
            0 => {}
            _ => {
                // getting byte index
                if let Some(idx) = cur_tab.buf[line_y]
                    .char_indices()
                    .nth(x_char - 1)
                    .map(|(i, _)| i)
                {
                    cur_tab.buf[line_y].remove(idx);
                    cur_tab.cursor_xy.0 -= 1;
                    cur_tab.changed = true;
                }
            }
        }
    }

    fn parse_command(&mut self) {
        let lexems: Vec<String> = self
            .command_buf
            .split_whitespace()
            .map(String::from)
            .collect();
        if lexems.is_empty() {
            self.throw_status_message("ERR: Command buffer is empty".to_string());
            return;
        }

        self.command_hist.push(lexems.clone());
        self.hist_ctr = self.command_hist.len();
        let command = &lexems.clone()[0];

        let mut res_com: &String = &command.clone();
        let mut res_args: Vec<String> = Vec::new();
        match self.aliases.get(command) {
            Some(cv) => {
                if !cv.is_empty() {
                    res_com = &cv[0];
                    res_args = cv[1..].to_vec();
                }
            }
            None => {}
        }
        let mut args: Vec<String> = lexems.into_iter().skip(1).collect();
        res_args.append(&mut args);
        let mut to_throw: Option<String> = None;

        match self.commands.get(&(res_com.clone())) {
            Some(handler) => match handler {
                CommandHandler::Rust(f) => f(self, res_args.clone()),
                CommandHandler::Lua(lf, id) => {
                    let PlugLoaders::LuaL(lualoader) = &self.plugin_subsys;
                    match lualoader.plugins.get(id.clone()) {
                        Some(plug) => {
                            let args_table =
                                plug.lua.create_table().map_err(|e| e.to_string()).unwrap();
                            for (i, arg) in res_args.iter().enumerate() {
                                args_table
                                    .set(i + 1, arg.clone())
                                    .map_err(|e| e.to_string())
                                    .unwrap();
                            }
                            let res = lf.call::<()>(&args_table);
                            match res {
                                Err(e) => {
                                    to_throw = Some(e.to_string());
                                }
                                _ => {}
                            }
                        }
                        None => {
                            self.throw_status_message(format!(
                                "Can't get plugin with id {}",
                                id.clone()
                            ));
                            return;
                        }
                    }

                    // debug
                    if let Some(m) = to_throw {
                        self.throw_status_message(m);
                    } else {
                        self.throw_status_message(res_com.clone());
                    }
                }
                _ => {}
            },
            None => {
                self.throw_status_message("ERR: No such command".to_string());
                return;
            }
        };
    }

    pub fn throw_status_message(&mut self, error: String) {
        self.command_buf = error;
        self.status_message = true;
    }

    fn gen_hashmap_com(&mut self) {
        self.commands
            .insert("!hi".to_string(), CommandHandler::Rust(commands::com_hi));
        self.commands
            .insert("!w".to_string(), CommandHandler::Rust(commands::com_w));
        self.commands
            .insert("!r".to_string(), CommandHandler::Rust(commands::com_r));
        self.commands
            .insert("!ri".to_string(), CommandHandler::Rust(commands::com_ri));
        self.commands
            .insert("!q".to_string(), CommandHandler::Rust(commands::com_q));
        self.commands.insert(
            "!exec".to_string(),
            CommandHandler::Rust(commands::com_exec),
        );
        self.commands.insert(
            "!execn".to_string(),
            CommandHandler::Rust(commands::com_execn),
        );
        self.commands.insert(
            "!exec_f".to_string(),
            CommandHandler::Rust(commands::com_exec_f),
        );
        self.commands.insert(
            "!execn_f".to_string(),
            CommandHandler::Rust(commands::com_execn_f),
        );
        self.commands
            .insert("!tab".to_string(), CommandHandler::Rust(commands::com_tab));
        self.commands.insert(
            "!version".to_string(),
            CommandHandler::Rust(commands::com_version),
        );
        self.commands
            .insert("!qi".to_string(), CommandHandler::Rust(commands::com_qi));
        self.commands
            .insert("!rn".to_string(), CommandHandler::Rust(commands::com_rn));
        self.commands.insert(
            "!alias".to_string(),
            CommandHandler::Rust(commands::com_alias),
        );
        self.commands.insert(
            "!plugin".to_string(),
            CommandHandler::Rust(commands::com_plugin),
        );
    }
}
