use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    process::Command,
};

use crossterm::event::{Event, KeyCode, MouseEventKind};

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
    pub scroll_offset: usize,
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
        let scroll: usize = 0;

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
            scroll_offset: scroll,
        };
        app.gen_hashmap_com();
        app
    }

    pub fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Insert | KeyCode::Esc => {
                    std::mem::swap(&mut self.cursor_pos_xy, &mut self.saved_cursor_pos_xy);
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

                    if (self.input_buf.len() < self.cursor_pos_xy.1 as usize) {
                        return;
                    }
                    let tgt_str: &mut String = self
                        .input_buf
                        .get_mut(self.cursor_pos_xy.1 as usize)
                        .expect("can't get buf string");
                    if (self.cursor_pos_xy.0 as usize <= tgt_str.len()) && self.insert_mode {
                        tgt_str.insert(self.cursor_pos_xy.0 as usize, ch);
                        self.cursor_pos_xy.0 += 1;
                    }
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
                        if (self.cursor_pos_xy.0 == 0) && (self.cursor_pos_xy.1 == 0) {
                            return;
                        }
                        let (before, after) =
                            self.input_buf.split_at_mut(self.cursor_pos_xy.1 as usize);
                        tgt_line = after.get_mut(0).expect("can't get cur line!");

                        if (self.cursor_pos_xy.0 == 0) {
                            prev_line = before.last_mut().expect("can't get prev line");
                            self.cursor_pos_xy.0 = (prev_line.len() - 1) as u16;
                            prev_line.push_str(tgt_line);
                            self.input_buf.remove(self.cursor_pos_xy.1 as usize);
                            self.cursor_pos_xy.1 -= 1;
                            return;
                        }
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
                        tgt_str = self
                            .input_buf
                            .get_mut(self.cursor_pos_xy.1 as usize)
                            .expect("can't get buf string");
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

                    let tgt_str: &mut String = self
                        .input_buf
                        .get_mut(self.cursor_pos_xy.1 as usize)
                        .expect("can't get buf string");

                    if (self.cursor_pos_xy.0 as usize <= tgt_str.len()) {
                        let to_move: String =
                            tgt_str[(self.cursor_pos_xy.0 as usize)..].to_string();
                        tgt_str.truncate(self.cursor_pos_xy.0 as usize);
                        self.input_buf
                            .insert((self.cursor_pos_xy.1 + 1) as usize, to_move.to_string());

                        self.cursor_pos_xy.0 = 0;
                        self.cursor_pos_xy.1 += 1;
                    }
                }
                KeyCode::PageUp => {
                    if (self.scroll_offset == 0) {
                        return;
                    }
                    self.scroll_offset -= 1;
                }
                KeyCode::PageDown => {
                    self.scroll_offset += 1;
                }
                KeyCode::Home => {
                    self.scroll_offset = 0;
                }
                KeyCode::End => {
                    self.scroll_offset = self.input_buf.len() - 1;
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
        let new_y = (self.cursor_pos_xy.1 as isize) + delta;
        if (new_y < 0) || (new_y as usize >= self.input_buf.len()) {
            return;
        }
        self.cursor_pos_xy.1 = new_y as u16;
        let mut idx: usize = self.cursor_pos_xy.0 as usize;
        let tgt_line = self.get_tgt_line(new_y as usize);

        if (idx >= tgt_line.len()) {
            if (tgt_line.len() == 0) {
                idx = 0;
            } else {
                idx = tgt_line.len() - 1;
            }
        }
        self.cursor_pos_xy.0 = idx as u16;
    }
    fn get_tgt_line(&mut self, ind: usize) -> &mut String {
        self.input_buf.get_mut(ind).expect("Can't get target line!")
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
        self.commands.insert("!q".to_string(), App::com_q);
        self.commands.insert("!exec".to_string(), App::com_exec);
        self.commands.insert("!exec_f".to_string(), App::com_exec_f);
    }

    fn com_hi(&mut self, args: Vec<String>) {
        self.throw_status_message("Hello!".to_string());
        return;
    }

    fn com_w(&mut self, args: Vec<String>) {
        let mut file_out_name: String = String::new();
        if (!args.is_empty()) {
            file_out_name = args[0].clone();
        } else if (!self.cur_filename.is_empty()) {
            file_out_name = self.cur_filename.clone();
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

        let mut contents: String = self.input_buf.join("\n");
        contents.push('\n');
        match file_out.write_all(contents.as_bytes()) {
            Ok(_) => self.throw_status_message("Success".to_string()),
            Err(e) => self.throw_status_message(e.to_string()),
        };
        self.cur_filename = file_out_name;
    }

    fn com_r(&mut self, args: Vec<String>) {
        if (args.is_empty()) {
            self.throw_status_message("Usage: !r filename".to_string());
            return;
        }

        let mut file_in: File = match File::open(args[0].clone()) {
            Ok(f) => f,
            Err(e) => {
                self.throw_status_message(e.to_string());
                return;
            }
        };

        self.input_buf.clear();
        let reader = BufReader::new(file_in);
        let mut err_flag: bool = false;
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let res = l.clone().replace('\n', "");
                    self.input_buf.push(res);
                }
                Err(e) => {
                    err_flag = true;
                    self.throw_status_message(e.to_string());
                }
            }
        }
        if (!err_flag) {
            self.throw_status_message("Success".to_string());
        }
        self.saved_cursor_pos_xy = (0, 0);
        self.cursor_pos_xy = (0, 0);
        self.cur_filename = args[0].clone();
        return;
    }

    fn com_q(&mut self, args: Vec<String>) {
        self.running = false;
        // TODO: Check if unsaved changes
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
}
