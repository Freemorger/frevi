use std::char;

pub struct Edit {
    pub start_line: usize,
    pub start_x: usize,
    pub end_line: usize,
    pub end_x: usize,
    pub diff: Vec<String>,
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
        let (col, line) = cursor;
        // something gotta be here
    }

    pub fn undo_edit(&mut self, buf: &mut Vec<String>) {}
}
