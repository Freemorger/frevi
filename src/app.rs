use std::{
    clone,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    process::Command,
};

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};

use crate::tabs::Tab;

#[derive(Debug, Clone)]
pub struct App {
    pub insert_mode: bool,
    pub input_buf: Vec<String>,
    pub running: bool,
    pub left_area_open: bool,
    pub cursor_pos_xy: (u16, u16),
    pub saved_cursor_pos_xy: (u16, u16),
    pub command_buf: String,
    pub commands: HashMap<String, Operation>,
    pub status_message: bool,
    pub cur_filename: String,
    pub tabs: Vec<Tab>,
    pub cur_tab: usize,
    pub version: String,
}
type Operation = fn(&mut App, Vec<String>);

impl App {
    pub fn new() -> App {
        let ins_mod: bool = false;
        let in_buf: Vec<String> = vec![String::new()];
        let run: bool = true;
        let left_open: bool = false;
        let cpos_xy: (u16, u16) = (0, 0);
        let com_buf: String = String::new();
        let coms: HashMap<String, Operation> = HashMap::new();
        let sav_cpos_xy: (u16, u16) = (0, 0);
        let stat_msg: bool = false;
        let cur_filename: String = String::new();
        let tabsv: Vec<Tab> = vec![Tab::new(None)];
        let curtab: usize = 0;
        let vers: &str = env!("CARGO_PKG_VERSION");

        let mut app = App {
            insert_mode: ins_mod,
            input_buf: in_buf,
            running: run,
            left_area_open: left_open,
            cursor_pos_xy: cpos_xy,
            command_buf: com_buf,
            commands: coms,
            saved_cursor_pos_xy: sav_cpos_xy,
            status_message: stat_msg,
            cur_filename: cur_filename,
            tabs: tabsv,
            cur_tab: curtab,
            version: vers.to_string(),
        };
        app.gen_hashmap_com();
        app
    }

    pub fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Insert | KeyCode::Esc => {
                    self.insert_mode = !self.insert_mode;
                }
                KeyCode::Char(ch) => {
                    if (!self.insert_mode) {
                        if (self.status_message) {
                            self.command_buf = "".to_string();
                            self.command_buf.push(ch);
                            self.cursor_pos_xy.0 = 1;
                            self.status_message = !self.status_message;
                            return;
                        }
                        if (self.cursor_pos_xy.0 as usize >= self.command_buf.len()) {
                            match self.command_buf.len() {
                                0 => {
                                    self.cursor_pos_xy.0 = 0;
                                }
                                val => {
                                    self.cursor_pos_xy.0 = (val) as u16;
                                }
                            }
                        }
                        self.command_buf.insert(self.cursor_pos_xy.0 as usize, ch);
                        self.cursor_pos_xy.0 += 1;
                        return;
                    }

                    self.insert_ch_tab_buf(ch);
                }
                KeyCode::Backspace => {
                    let tgt_line: &mut String;
                    let prev_line: &mut String;
                    if (!self.insert_mode) {
                        tgt_line = &mut self.command_buf;
                        if (self.status_message) {
                            self.command_buf = "".to_string();
                            self.status_message = !self.status_message;
                            return;
                        }
                    } else {
                        self.tab_backspace();
                        return;
                    }

                    let curpos_x_bordered = if (self.cursor_pos_xy.0 == 0) {
                        self.cursor_pos_xy.0
                    } else {
                        self.cursor_pos_xy.0 - 1
                    };
                    if (curpos_x_bordered < tgt_line.len() as u16) {
                        tgt_line.remove((curpos_x_bordered) as usize);
                        self.cursor_pos_xy.0 = curpos_x_bordered;
                    }
                }
                KeyCode::Left => {
                    if (self.insert_mode) {
                        self.move_cursor_hor(-1);
                        return;
                    }
                    if (self.cursor_pos_xy.0 == 0) {
                        return;
                    }
                    self.cursor_pos_xy.0 -= 1;
                }
                KeyCode::Right => {
                    let tgt_str: &mut String;
                    if (!self.insert_mode) {
                        tgt_str = &mut self.command_buf;
                    } else {
                        self.move_cursor_hor(1);
                        return;
                    }

                    if (self.cursor_pos_xy.0 as usize >= tgt_str.len()) {
                        return;
                    }
                    self.cursor_pos_xy.0 += 1;
                }
                KeyCode::Up => {
                    self.move_cursor_vert(-1);
                }
                KeyCode::Down => {
                    self.move_cursor_vert(1);
                }
                KeyCode::Enter => {
                    if (!self.insert_mode) {
                        self.parse_command();
                        return;
                    }

                    self.tab_newline();
                }
                KeyCode::PageUp => {
                    self.tab_update_scroll_delta(-1);
                }
                KeyCode::PageDown => {
                    self.tab_update_scroll_delta(1);
                }
                KeyCode::Home => {
                    self.tab_update_scroll(0);
                }
                KeyCode::End => {
                    self.tab_update_scroll(usize::MAX); // this will work due to clamp inside function.
                }
                KeyCode::F(num) => {
                    self.cur_tab = num
                        .saturating_sub(1)
                        .min(self.tabs.len().saturating_sub(1) as u8)
                        as usize;
                }
                _ => {}
            },
            // Event::Mouse(m_ev) => match m_ev.kind {
            //     MouseEventKind::ScrollUp => { // doesn't work in ratatui for some reason...
            //         if (self.scroll_offset == 0) {
            //             return;
            //         }
            //         self.scroll_offset -= 1;
            //     }
            //     MouseEventKind::ScrollDown => {
            //         self.scroll_offset += 1;
            //     }
            //     _ => {}
            // },
            _ => {}
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

        let new_x =
            ((cur_tab.cursor_xy.0 as isize) + delta).clamp(0, (tgt_line.len() - 1) as isize);
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
        let cur_tab: &mut Tab = &mut self.tabs[self.cur_tab];
        let line_y = cur_tab.cursor_xy.1.clamp(0, cur_tab.buf.len());
        let x = cur_tab.cursor_xy.0.clamp(0, cur_tab.buf[line_y].len());

        cur_tab.buf[line_y].insert(x, ch);
        cur_tab.cursor_xy.0 += 1;
        cur_tab.changed = true;
    }

    fn tab_newline(&mut self) {
        let cur_tab: &mut Tab = &mut self.tabs[self.cur_tab];
        let line_y = cur_tab.cursor_xy.1.clamp(0, cur_tab.buf.len());
        let x = cur_tab.cursor_xy.0.clamp(0, cur_tab.buf[line_y].len());
        let tgt_str = &mut cur_tab.buf[line_y];

        let to_move: String = tgt_str[(x as usize)..].to_string();
        tgt_str.truncate(x as usize);
        cur_tab.buf.insert(line_y + 1, to_move);
        cur_tab.cursor_xy.0 = 0;
        cur_tab.cursor_xy.1 = line_y + 1;
        cur_tab.changed = true;
        return;
    }

    fn tab_backspace(&mut self) {
        let cur_tab: &mut Tab = &mut self.tabs[self.cur_tab];
        let line_y = cur_tab.cursor_xy.1.clamp(0, cur_tab.buf.len());
        let x = cur_tab.cursor_xy.0.clamp(0, cur_tab.buf[line_y].len());
        let mut tgt_line = &mut cur_tab.buf[line_y];

        let (before, after) = cur_tab.buf.split_at_mut(cur_tab.cursor_xy.1 as usize);
        tgt_line = after.get_mut(0).expect("can't get cur line!");

        if (cur_tab.cursor_xy.0 == 0) {
            if (cur_tab.cursor_xy.1 == 0) {
                return;
            }
            let prev_line = before.last_mut().expect("can't get prev line");
            cur_tab.cursor_xy.0 = prev_line.len() - 1;
            prev_line.push_str(tgt_line);
            cur_tab.buf.remove(cur_tab.cursor_xy.1 as usize);
            cur_tab.cursor_xy.1 -= 1;
            cur_tab.changed = true;
            return;
        }

        match x {
            0 => {}
            _ => {
                cur_tab.buf[line_y].remove(x.saturating_sub(1));
                cur_tab.changed = true;
                cur_tab.cursor_xy.0 = cur_tab.cursor_xy.0.saturating_sub(1);
            }
        }
    }

    fn parse_command(&mut self) {
        let lexems: Vec<String> = self
            .command_buf
            .split_whitespace()
            .map(String::from)
            .collect();
        if (lexems.is_empty()) {
            self.throw_status_message("ERR: Command buffer is empty".to_string());
            return;
        }

        let command = &lexems.clone()[0];
        let args = lexems.into_iter().skip(1).collect();

        match self.commands.get(command) {
            Some(f) => f(self, args),
            None => self.throw_status_message("ERR: No such command".to_string()),
        };
    }

    pub fn throw_status_message(&mut self, error: String) {
        self.command_buf = error;
        self.status_message = true;
    }

    fn gen_hashmap_com(&mut self) {
        self.commands.insert("!hi".to_string(), App::com_hi);
        self.commands.insert("!w".to_string(), App::com_w);
        self.commands.insert("!r".to_string(), App::com_r);
        self.commands.insert("!ri".to_string(), App::com_ri);
        self.commands.insert("!q".to_string(), App::com_q);
        self.commands.insert("!exec".to_string(), App::com_exec);
        self.commands.insert("!execn".to_string(), App::com_execn);
        self.commands.insert("!exec_f".to_string(), App::com_exec_f);
        self.commands
            .insert("!execn_f".to_string(), App::com_execn_f);
        self.commands.insert("!tab".to_string(), App::com_tab);
        self.commands
            .insert("!version".to_string(), App::com_version);
        self.commands.insert("!qi".to_string(), App::com_qi);
        self.commands.insert("!rn".to_string(), App::com_rn);
    }

    fn com_hi(&mut self, args: Vec<String>) {
        self.throw_status_message("Hello!".to_string());
        return;
    }

    fn com_w(&mut self, args: Vec<String>) {
        let curtab = &mut self.tabs[self.cur_tab];
        let mut file_out_name: String = String::new();
        if (!args.is_empty()) {
            file_out_name = args[0].clone();
        } else if (!curtab.filename.is_empty()) {
            file_out_name = curtab.filename.clone();
        } else {
            self.throw_status_message("Usage: !w filename".to_string());
            return;
        }

        let mut file_out: File = match File::create(file_out_name.clone()) {
            Ok(f) => f,
            Err(e) => {
                self.throw_status_message(e.to_string());
                return;
            }
        };

        curtab.filename = file_out_name;
        curtab.changed = false;
        let mut contents: String = curtab.buf.join("\n");
        contents.push('\n');
        match file_out.write_all(contents.as_bytes()) {
            Ok(_) => self.throw_status_message("Success".to_string()),
            Err(e) => {
                curtab.changed = true;
                self.throw_status_message(e.to_string());
            }
        };
    }

    fn com_r(&mut self, args: Vec<String>) {
        let curtab = &mut self.tabs[self.cur_tab];
        if (curtab.changed) {
            self.throw_status_message("W: Current buffer isn't saved. !ri to ignore".to_string());
            return;
        }
        if (args.is_empty()) {
            self.throw_status_message("Usage: !r filename".to_string());
            return;
        }

        let file_in: File = match File::open(args[0].clone()) {
            Ok(f) => f,
            Err(e) => {
                self.throw_status_message(e.to_string());
                return;
            }
        };

        curtab.buf.clear();
        let reader = BufReader::new(file_in);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let res = l.clone().replace('\n', "");
                    curtab.buf.push(res);
                }
                Err(e) => {
                    curtab.buf.clear();
                    self.throw_status_message(e.to_string());
                    return;
                }
            }
        }
        curtab.changed = false;
        curtab.cursor_xy = (0, 0);
        curtab.filename = args[0].clone();
        self.throw_status_message("Success".to_string());
        return;
    }

    fn com_ri(&mut self, args: Vec<String>) {
        let curtab = &mut self.tabs[self.cur_tab];
        if (args.is_empty()) {
            self.throw_status_message("Usage: !r filename".to_string());
            return;
        }

        let file_in: File = match File::open(args[0].clone()) {
            Ok(f) => f,
            Err(e) => {
                self.throw_status_message(e.to_string());
                return;
            }
        };

        curtab.buf.clear();
        let reader = BufReader::new(file_in);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let res = l.clone().replace('\n', "");
                    curtab.buf.push(res);
                }
                Err(e) => {
                    curtab.buf.clear();
                    self.throw_status_message(e.to_string());
                    return;
                }
            }
        }
        curtab.changed = false;
        curtab.cursor_xy = (0, 0);
        curtab.filename = args[0].clone();
        self.throw_status_message("Success".to_string());
        return;
    }

    fn com_rn(&mut self, args: Vec<String>) {
        if (args.is_empty()) {
            self.throw_status_message("Usage: !rn filename".to_string());
            return;
        }
        let filename = args[0].clone();
        let mut newtab = Tab::new(Some(filename.clone()));

        match newtab.readf(filename.clone()) {
            Ok(()) => {}
            Err(e) => {
                self.throw_status_message(e.to_string());
                return;
            }
        }
        newtab.filename = filename;
        self.tabs.push(newtab);
        self.cur_tab = self.tabs.len().saturating_sub(1);
        self.throw_status_message("Success".to_string());
        return;
    }

    fn com_q(&mut self, args: Vec<String>) {
        let curtab = &self.tabs[self.cur_tab];
        if (curtab.changed) {
            self.throw_status_message(
                "W: Current buffer has unsaved changes; !qi to ignore".to_string(),
            );
            return;
        }
        self.running = false;
    }

    fn com_qi(&mut self, args: Vec<String>) {
        self.running = false;
    }

    fn com_exec(&mut self, args: Vec<String>) {
        if (args.is_empty()) {
            self.throw_status_message("Usage: !exec command".to_string());
            return;
        }
        let com = if cfg!(target_os = "windows") {
            let argline: &str = &args.join(" ");
            Command::new("cmd")
                .args(&["/C", &argline])
                .output()
                .expect("Error creating cmd")
        } else {
            let argline: &str = &args.join(" ");
            Command::new("sh")
                .args(&["-c", argline])
                .output()
                .expect("Error running sh")
        };

        let mut output_s: String = String::new();
        if (com.stdout.is_empty()) {
            output_s = String::from_utf8_lossy(&com.stderr).to_string();
        } else {
            output_s = String::from_utf8_lossy(&com.stdout).to_string();
        }
        self.throw_status_message(output_s);
    }

    fn com_exec_f(&mut self, args: Vec<String>) {
        // executes shell/cmd script from file (current or specified)
        if (args.is_empty()) {
            self.throw_status_message("Usage: !exec_f filename".to_string());
            return;
        }

        let com = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(args)
                .output()
                .expect("Error creating cmd")
        } else {
            Command::new("sh")
                .args(args)
                .output()
                .expect("Error running sh")
        };

        let mut output_s: String = String::new();
        if (com.stdout.is_empty()) {
            output_s = String::from_utf8_lossy(&com.stderr).to_string();
        } else {
            output_s = String::from_utf8_lossy(&com.stdout).to_string();
        }
        self.throw_status_message(output_s);
    }

    fn com_execn(&mut self, args: Vec<String>) {
        if (args.is_empty()) {
            self.throw_status_message("Usage: !execn command".to_string());
            return;
        }
        let com = if cfg!(target_os = "windows") {
            let argline: &str = &args.join(" ");
            Command::new("cmd")
                .args(&["/C", &argline])
                .output()
                .expect("Error creating cmd")
        } else {
            let argline: &str = &args.join(" ");
            Command::new("sh")
                .args(&["-c", argline])
                .output()
                .expect("Error running sh")
        };

        let mut output_s: String = String::new();
        if (com.stdout.is_empty()) {
            output_s = String::from_utf8_lossy(&com.stderr).to_string();
        } else {
            output_s = String::from_utf8_lossy(&com.stdout).to_string();
        }
        let mut output_tab = Tab::new(Some("Output".to_string()));
        let lines: Vec<String> = output_s.lines().map(|line| line.to_string()).collect();
        output_tab.buf = lines;
        self.tabs.push(output_tab);
        self.cur_tab = self.tabs.len().saturating_sub(1);

        self.throw_status_message("Success".to_string());
        return;
    }

    fn com_execn_f(&mut self, args: Vec<String>) {
        if (args.is_empty()) {
            self.throw_status_message("Usage: !execn command".to_string());
            return;
        }
        let com = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(args)
                .output()
                .expect("Error creating cmd")
        } else {
            Command::new("sh")
                .args(args)
                .output()
                .expect("Error running sh")
        };

        let mut output_s: String = String::new();
        if (com.stdout.is_empty()) {
            output_s = String::from_utf8_lossy(&com.stderr).to_string();
        } else {
            output_s = String::from_utf8_lossy(&com.stdout).to_string();
        }
        let mut output_tab = Tab::new(Some("Output".to_string()));
        let lines: Vec<String> = output_s.lines().map(|line| line.to_string()).collect();
        output_tab.buf = lines;
        self.tabs.push(output_tab);
        self.cur_tab = self.tabs.len().saturating_sub(1);

        self.throw_status_message("Success".to_string());
        return;
    }

    fn com_tab(&mut self, args: Vec<String>) {
        if (args.is_empty()) {
            self.throw_status_message(
                "Usage: !tab new, !tab goto num, !tab rm num, !tab next, !tab prev, !tab rename num name".to_string(),
            );
            return;
        }
        if (args[0] == "new") {
            self.tabs.push(Tab::new(None));
            self.throw_status_message("Success".to_string());
            return;
        }
        if (args[0] == "goto") {
            let ind: usize = match args[1].parse() {
                Ok(n) => n,
                Err(e) => {
                    self.throw_status_message(e.to_string());
                    return;
                }
            };
            if (ind > self.tabs.len()) {
                self.throw_status_message("Tab with specified indice not opened".to_string());
                return;
            }
            self.cur_tab = ind.saturating_sub(1);
            self.throw_status_message("Success".to_string());
            return;
        }
        if (args[0] == "rm") {
            let mut ind: usize = match args[1].parse() {
                Ok(n) => n,
                Err(e) => {
                    self.throw_status_message(e.to_string());
                    return;
                }
            };
            ind = ind.saturating_sub(1);
            if (ind >= self.tabs.len()) {
                self.throw_status_message("Tab with specified indice not opened".to_string());
                return;
            }
            self.tabs.remove(ind);
            if (self.tabs.len() == 0) {
                let newtab = Tab::new(None);
                self.tabs.push(newtab);
                self.cur_tab = 0;
            } else if (self.cur_tab >= self.tabs.len()) {
                self.cur_tab = self.cur_tab.saturating_sub(1);
            }
            self.throw_status_message("Success".to_string());
            return;
        }
        if (args[0] == "next") {
            if (self.cur_tab + 1 >= self.tabs.len()) {
                self.throw_status_message("Current tab is already last!".to_string());
                return;
            }
            self.cur_tab += 1;
            self.throw_status_message("Success".to_string());
            return;
        }
        if (args[0] == "prev") {
            if (self.cur_tab == 0) {
                self.throw_status_message("Current tab is first!".to_string());
                return;
            }
            self.cur_tab -= 1;
            self.throw_status_message("Success".to_string());
            return;
        }
        if (args[0] == "rename") {
            let ind: usize = match args[1].parse() {
                Ok(n) => n,
                Err(e) => {
                    self.throw_status_message(e.to_string());
                    return;
                }
            };
            if (ind > self.tabs.len()) {
                self.throw_status_message("Tab with specified indice not opened".to_string());
                return;
            }
            let new_name: String = args[2..].join(" ");
            self.tabs[ind.saturating_sub(1)].displayed_name = new_name;
            self.throw_status_message("Success".to_string());
            return;
        }
        self.throw_status_message(
            "Usage: !tab new, !tab goto num, !tab rm num, !tab next, !tab prev, !tab rename num name".to_string(),
        );
        return;
    }

    fn com_version(&mut self, args: Vec<String>) {
        self.throw_status_message(self.version.clone());
        return;
    }
}
