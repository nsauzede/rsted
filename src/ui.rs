use crate::editor::Editor;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
pub fn draw(f: &mut Frame, editor: &mut Editor) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.area());
    draw_editor_area(f, editor, chunks[0]);
    draw_status_bar(f, editor, chunks[1]);
}
fn draw_editor_area(f: &mut Frame, editor: &mut Editor, area: ratatui::layout::Rect) {
    let lines: Vec<Line> = editor
        .get_lines()
        .into_iter()
        .map(|line| {
            let chunks = editor.highlighter.highlight_line(&line);
            let spans: Vec<Span> = chunks
                .into_iter()
                .map(|(syntect_style, text)| {
                    let ratatui_style = Style::new().fg(Color::Rgb(
                        syntect_style.foreground.r,
                        syntect_style.foreground.g,
                        syntect_style.foreground.b,
                    ));
                    Span::styled(text.to_string(), ratatui_style)
                })
                .collect();
            Line::from(spans)
        })
        .collect();
    let title = format!(
        "{:16}   [{}{}--] {:2} L:[  0+ 0 {:3}/{:3}] *() ",
        editor.file_path.display(), //.file_name()
        //.unwrap_or_default()
        //.to_string_lossy()
        if editor.block { "B" } else { "-" },
        if editor.modified { "M" } else { "-" },
        editor.cursor.1,
        editor.cursor.0 + 1,
        editor.rope.len_lines()
    );
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, area);
}
fn draw_status_bar(f: &mut Frame, editor: &mut Editor, area: ratatui::layout::Rect) {
    let status = format!(
        " 1{:12} 2{:12} | block: start={:?} end={:?} | cnt={}",
        "Help", "Save", editor.block_start, editor.block_end, editor.frame_count
    );
    let paragraph = Paragraph::new(status).block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}
