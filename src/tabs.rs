use crate::edits::Edit;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone)]
pub struct Tab {
    pub filename: String,
    pub buf: Vec<String>,
    pub cursor_xy: (usize, usize),
    pub displayed_name: String,
    pub changed: bool,
    pub scroll_offset: usize,
    pub edit_hist: Vec<Edit>,
}

impl Tab {
    // Creates a new tab with name.
    // Sets name "New tab" if other not passed
    pub fn new(displayed_name: Option<String>) -> Tab {
        let fname: String = String::new();
        let buf: Vec<String> = vec![String::new()];
        let cursor_pos: (usize, usize) = (0, 0);
        let displayed_n = displayed_name.unwrap_or("New tab".to_string());
        let changes: bool = false;
        let sc_offset: usize = 0;
        let ed_h: Vec<Edit> = Vec::new();

        Tab {
            filename: fname,
            buf: buf,
            cursor_xy: cursor_pos,
            displayed_name: displayed_n,
            changed: changes,
            scroll_offset: sc_offset,
            edit_hist: ed_h,
        }
    }

    // Reads file into tab
    pub fn readf(&mut self, filename: String) -> Result<(), std::io::Error> {
        self.filename = filename.clone();
        let file: File = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        let buf_reader = BufReader::new(file);
        for line in buf_reader.lines() {
            self.buf.push(line.expect("Can't parse line!"));
        }

        Ok(())
    }

    pub fn str_into_buf(&mut self, content: String) {
        let lines: Vec<String> = content.split('\n').map(|s| s.to_string()).collect();
        self.buf = lines;
    }
}
