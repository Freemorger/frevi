use std::char;

use crate::tabs::Tab;

#[derive(Debug, Clone)]
pub struct Edit {
    pub start_line: usize,
    pub start_x: usize,
    pub end_line: usize,
    pub end_x: usize,
    pub diff: Vec<String>, // previous buffer
}

impl Edit {
    pub fn new_at_curs(cursor: (usize, usize)) -> Edit {
        let start_l: usize = cursor.1;
        let start_x: usize = cursor.0;
        let end_line: usize = 0;
        let end_x: usize = 0;
        let diff_made: Vec<String> = vec![String::new()];

        Edit {
            start_line: start_l,
            start_x: start_x,
            end_line: end_line,
            end_x: end_x,
            diff: diff_made,
        }
    }

    pub fn edit_at_curs(&mut self, cursor: (usize, usize), ch: char) {
        let (col_n, line_n) = cursor;
        let line = &mut self.diff[line_n];
        let x_chars = col_n.clamp(0, line.chars().count());
        let byte_index = line
            .char_indices()
            .nth(x_chars)
            .map(|(i, _)| i)
            .unwrap_or_else(|| line.len());

        // todo once in future here...
    }

    pub fn undo_edit(&mut self, buf: &mut Vec<String>) {}

    pub fn dbg_show_edit(&mut self) -> Tab {
        let mut res: Tab = Tab::new(Some("Debug".to_string()));
        res.buf = self.diff.clone();
        res
    }
}
