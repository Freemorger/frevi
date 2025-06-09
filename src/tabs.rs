pub struct Tab {
    pub filename: String,
    pub buf: Vec<String>,
    pub cursor_xy: (usize, usize),
    pub displayed_name: String,
    pub changed: bool,
}

impl Tab {
    pub fn new(filename: String, displayed_name: Option<String>) -> Tab {
        let fname: String = filename;
        let buf: Vec<String> = vec![String::new()];
        let cursor_pos: (usize, usize) = (0, 0);
        let displayed_n = displayed_name.unwrap_or(fname.clone());
        let changes: bool = false;

        Tab {
            filename: fname,
            buf: buf,
            cursor_xy: cursor_pos,
            displayed_name: displayed_n,
            changed: changes,
        }
    }
}
