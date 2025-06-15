mod app;
mod commands;
mod edits;
mod tabs;
use crate::app::App;
use crossterm::event::{self};
use ratatui::{
    self, Frame,
    layout::{
        Constraint::{Fill, Length, Min},
        Layout, Position,
    },
    prelude::Stylize,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Tabs},
};

fn main() {
    let mut app = App::new();
    app.running = true;

    match std::env::args().nth(1) {
        // opening file from cli
        Some(s) => match app.tabs[0].readf(s.clone()) {
            Ok(()) => {}
            Err(e) => {
                let err_msg: String = "While opening file: ".to_string() + &e.to_string();
                app.throw_status_message(err_msg.clone());
            }
        },
        None => {}
    };

    let mut terminal = ratatui::init();
    while app.running {
        terminal
            .draw(|f| draw(f, &app))
            .expect("failed to render frame");
        let event = event::read().expect("failed to read event");
        app.handle_input(event);
    }
    ratatui::restore();
}

fn draw(frame: &mut Frame, app: &App) {
    let curtab = &app.tabs[app.cur_tab];

    let mut vert_length: u16 = 2;
    let stat_length: u16 = 1;
    if !curtab.filename.is_empty() {
        vert_length = 3;
    }

    let vertical = Layout::vertical([Length(vert_length), Min(0), Length(stat_length)]);
    let [title_area, main_area, status_area] = vertical.areas(frame.area());
    let (left_area, right_area);
    if app.left_area_open {
        let horizontal = Layout::horizontal([Fill(1); 2]);
        let [l, r] = horizontal.areas(main_area);
        left_area = Some(l);
        right_area = r;
    } else {
        right_area = main_area;
        left_area = None;
    }
    let available_length: u16 = frame.area().height - vert_length - stat_length;

    let title_area_chunks = Layout::vertical([Fill(1), Length(vert_length - 1)]).split(title_area);
    let tabs = Tabs::new(
        app.tabs
            .iter()
            .map(|cur_tab| cur_tab.displayed_name.as_str()),
    )
    .select(app.cur_tab)
    .style(Style::default())
    .highlight_style(Style::default().fg(Color::LightCyan).bold())
    .divider("|");

    let title_text = match curtab.changed {
        true => curtab.filename.clone() + " *",
        false => curtab.filename.clone(),
    };
    let title = Paragraph::new(title_text)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::Yellow).bold());
    frame.render_widget(title, title_area_chunks[1]);
    frame.render_widget(tabs, title_area_chunks[0]);

    let buf_len = curtab.buf.len();
    let start_line = curtab.scroll_offset.min(buf_len);
    let end_line = (curtab.scroll_offset + available_length as usize).min(buf_len);

    let visible_text = curtab.buf[start_line..end_line]
        .iter()
        .enumerate()
        .map(|(i, line)| format!("{}: {}", i + start_line + 1, line))
        .collect::<Vec<String>>()
        .join("\n");

    let paragraph = Paragraph::new(visible_text)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(paragraph, right_area);

    if (app.left_area_open) {
        let left_tab = &app.left_area;
        let buf_len_left = left_tab.buf.len();
        let start_line_left = left_tab.scroll_offset.min(buf_len_left);
        let end_line_left = (left_tab.scroll_offset + available_length as usize).min(buf_len_left);

        let visible_text_left = curtab.buf[start_line_left..end_line_left]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{}: {}", i + start_line + 1, line))
            .collect::<Vec<String>>()
            .join("\n");

        let paragraph_left = Paragraph::new(visible_text_left)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Left);
        frame.render_widget(paragraph_left, left_area.unwrap());
    }

    let mut status_str = app.command_buf.clone();

    if app.insert_mode {
        status_str.push_str("\t -- INSERT -- \t");

        let digits_ctr =
            num_decimal_digits(curtab.scroll_offset + (curtab.cursor_xy.1 as usize) + 1) as u16;
        frame.set_cursor_position(Position::new(
            right_area.x + (curtab.cursor_xy.0 as u16) + 2 + digits_ctr, //adding y for line counter
            right_area.y + curtab.cursor_xy.1 as u16,
        ));
    } else if !app.command_buf.is_empty() {
        frame.set_cursor_position(Position::new(
            status_area.x + app.cursor_pos_xy.0,
            status_area.y,
        ));
    }

    let status_text = Text::raw(status_str);
    frame.render_widget(status_text, status_area);
}

fn num_decimal_digits<T: std::fmt::Display>(n: T) -> usize {
    n.to_string().chars().filter(|c| c.is_digit(10)).count()
}
