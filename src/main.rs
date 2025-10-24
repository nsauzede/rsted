mod editor;
mod filesystem;
mod highlighter;
mod ui;
use anyhow::Result;
use clap::Parser;
use crossterm::{
    cursor::Show,
    event::{self, DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use editor::Editor;
use filesystem::{AppEvent, FileWatcher};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
#[derive(Parser)]
#[command(name = "rsted", about = "Simple Rust CLI text/source editor")]
struct Cli {
    /// File to open, optionally with line number (file[:lineno])
    #[arg(value_name = "file[:<lineno>]")]
    file: String,
    /// Start at line number
    #[arg(value_name = "+<lineno>", allow_hyphen_values = true)]
    line: Option<String>,
}
fn parse_args(file_arg: String, line_arg: Option<String>) -> (PathBuf, usize) {
    let (path_str, mut line_num) = if let Some((path, line_str)) = file_arg.rsplit_once(':') {
        if let Ok(num) = line_str.parse::<usize>() {
            (path, Some(num))
        } else {
            (file_arg.as_str(), None)
        }
    } else {
        (file_arg.as_str(), None)
    };
    // Parse +lineno argument (file:lineno takes precedence)
    if line_num.is_none()
        && let Some(line_str) = line_arg.and_then(|s| s.strip_prefix('+').map(String::from)) {
            line_num = line_str.parse().ok();
        }
    let file_path = PathBuf::from(path_str);
    let line_num = line_num.unwrap_or(1);
    (file_path, line_num)
}
fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let (file_path, file_line) = parse_args(cli.file, cli.line);
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut editor = Editor::new(file_path.clone(), file_line)?;
    let (tx, rx) = mpsc::channel();
    let _file_watcher = FileWatcher::new(file_path.clone(), tx);
    run_app(&mut terminal, &mut editor, rx)?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    )?;
    terminal.show_cursor()?;
    Ok(())
}
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    editor: &mut Editor,
    event_rx: Receiver<AppEvent>,
) -> io::Result<()> {
    loop {
        //        if let Ok(AppEvent::FileChange(_)) = event_rx.try_recv() {
        if let Ok(AppEvent::FileChange) = event_rx.try_recv()
            && !editor.modified
        {
            editor.reload();
        }
        if event::poll(Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                match editor.process_key(key.code) {
                    editor::Action::Continue => {}
                    editor::Action::Quit => break Ok(()),
                    editor::Action::Redraw => {}
                    editor::Action::Save => if editor.save().is_err() {},
                }
            } else if let event::Event::Mouse(mouse) = event::read()? {
                match mouse.kind {
                    //
                    //                    MouseEventKind::Moved => editor.mouse_move(mouse.column, mouse.row),
                    MouseEventKind::Drag(MouseButton::Left) => {
                        editor.mouse_drag(mouse.column, mouse.row)
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        editor.mouse_down(mouse.column, mouse.row)
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        editor.mouse_up(mouse.column, mouse.row)
                    }
                    //                    MouseEventKind::ScrollUp => editor.scroll_up(),
                    //                    MouseEventKind::ScrollDown => editor.scroll_down(),
                    _ => {}
                }
            }
        }
        terminal.draw(|f| {
            ui::draw(f, editor);
            let cursor_pos = editor.cursor;
            let x = cursor_pos.1 as u16 + 1;
            let y = cursor_pos.0 as u16 + 1;
            f.set_cursor_position((x, y));
        })?;
    }
}
