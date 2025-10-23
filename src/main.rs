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
#[command(name = "rsted")]
#[command(about = "Midnight Commander-style console editor")]
struct Args {
    file: Option<PathBuf>,
}
fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let file_path = args
        .file
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("hello/src/main.rs"));
    println!("file_path={}", file_path.display());
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut editor = Editor::new(file_path.clone())?;
    let (tx, rx) = mpsc::channel();
    let _file_watcher = FileWatcher::new(file_path.clone(), tx);
    let _ = run_app(&mut terminal, &mut editor, rx)?;
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
        if let Ok(AppEvent::FileChange) = event_rx.try_recv() {
            if !editor.modified {
                editor.reload();
            }
        }
        if event::poll(Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                match editor.process_key(key.code) {
                    editor::Action::Continue => {}
                    editor::Action::Quit => break Ok(()),
                    editor::Action::Redraw => {}
                    editor::Action::Save => if let Err(_) = editor.save() {},
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
