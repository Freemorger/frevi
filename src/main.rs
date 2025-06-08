mod app;
use crate::app::App;
use crossterm::event::{self, Event};
use ratatui::{
    self, Frame,
    crossterm::terminal,
    layout::{
        Constraint::{self, Fill, Length, Min},
        Layout, Position,
    },
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

fn main() {
    let mut app = App::new();
    app.running = true;

    let mut terminal = ratatui::init();
    while (app.running) {
        terminal
            .draw(|f| draw(f, &app))
            .expect("failed to render frame");
        let event = event::read().expect("failed to read event");
        app.handle_input(event);
    }
    ratatui::restore();
}

fn draw(frame: &mut Frame, app: &App) {
    let mut vert_length: u16 = 0;
    let mut stat_length: u16 = 1;
    if (!app.cur_filename.is_empty()) {
        vert_length = 2;
    }

    let vertical = Layout::vertical([Length(vert_length), Min(0), Length(stat_length)]);
    let [title_area, main_area, status_area] = vertical.areas(frame.area());
    let (left_area, right_area);
    if (app.left_area_open) {
        let horizontal = Layout::horizontal([Fill(1); 2]);
        let [l, r] = horizontal.areas(main_area);
        left_area = Some(l);
        right_area = r;
    } else {
        right_area = main_area;
        left_area = None;
    }
    let available_length: u16 = frame.area().height - vert_length - stat_length;

    let title = Paragraph::new(app.cur_filename.clone())
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::Yellow).bold());
    frame.render_widget(title, title_area);

    let start_line = if (app.scroll_offset >= app.input_buf.len()) {
        app.input_buf.len()
    } else {
        app.scroll_offset
    };
    let end_line = if (app.scroll_offset + (available_length as usize)) > app.input_buf.len() {
        app.input_buf.len()
    } else {
        app.scroll_offset + (available_length as usize)
    };

    let visible_text = app.input_buf[start_line..end_line]
        .iter()
        .enumerate()
        .map(|(i, line)| format!("{}: {}", i + start_line + 1, line))
        .collect::<Vec<String>>()
        .join("\n");

    let paragraph = Paragraph::new(visible_text)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(paragraph, right_area);

    let mut status_str = app.command_buf.clone();

    if (app.insert_mode) {
        status_str.push_str("\t -- INSERT -- \t");

        let digits_ctr = num_decimal_digits(app.cursor_pos_xy.1) as u16;
        frame.set_cursor_position(Position::new(
            right_area.x + app.cursor_pos_xy.0 + 2 + digits_ctr, //adding y for line counter
            right_area.y + app.cursor_pos_xy.1,
        ));
    } else if (!app.command_buf.is_empty()) {
        frame.set_cursor_position(Position::new(
            status_area.x + app.cursor_pos_xy.0,
            status_area.y,
        ));
    }

    let status_text = Text::raw(status_str);
    frame.render_widget(status_text, status_area);

    // let debug_info = format!(
    //     "Scroll: {}/{} | Lines: {}-{} | Total: {}",
    //     app.scroll_offset,
    //     app.input_buf
    //         .len()
    //         .saturating_sub(available_length as usize),
    //     start_line + 1,
    //     end_line,
    //     app.input_buf.len()
    // );

    // let status = Paragraph::new(debug_info);
    // frame.render_widget(status, status_area);
}

fn num_decimal_digits<T: std::fmt::Display>(n: T) -> usize {
    n.to_string().chars().filter(|c| c.is_digit(10)).count()
}
