use crate::highlighter::Highlighter;
use anyhow::Result;
use crossterm::event::KeyCode;
use ropey::Rope;
use std::fs;
use std::path::PathBuf;

pub enum Action {
    Continue,
    Quit,
    Redraw,
    Save,
}

pub struct Editor {
    pub rope: Rope,
    pub cursor: (usize, usize), // (line, col)
    pub file_path: PathBuf,
    pub modified: bool,
    pub block: bool,
    pub block_start: (usize, usize), // (line, col)
    pub block_end: (usize, usize),   // (line, col)
    pub highlighter: Highlighter,
    pub tab_size: usize,
    pub frame_count: usize,
}

impl Editor {
    pub fn new(file_path: PathBuf, file_line: usize) -> Result<Self> {
        let content = fs::read_to_string(&file_path).unwrap_or_default();
        let rope = Rope::from_str(&content);
        let cursor = (if file_line > 0 { file_line - 1 } else { 0 }, 0);

        let mut highlighter = Highlighter::new();

        // Auto-detect file type for syntax highlighting
        if let Some(ext) = file_path.extension()
            && let Some(ext_str) = ext.to_str()
        {
            highlighter.set_language(ext_str);
        }

        let editor = Self {
            rope,
            cursor,
            file_path,
            modified: false,
            block: false,
            block_start: (0, 0),
            block_end: (0, 0),
            highlighter,
            tab_size: 4,
            frame_count: 0,
        };

        Ok(editor)
    }

    pub fn process_key(&mut self, key: KeyCode) -> Action {
        match key {
            KeyCode::F(10) => Action::Quit,
            KeyCode::Esc => Action::Quit,
            KeyCode::F(2) => Action::Save,
            KeyCode::Char(c) => {
                self.insert_char(c);
                Action::Redraw
            }
            KeyCode::Enter => {
                self.insert_char('\n');
                Action::Redraw
            }
            KeyCode::Backspace => {
                self.delete_char();
                Action::Redraw
            }
            KeyCode::Delete => {
                self.delete_next_char();
                Action::Redraw
            }
            KeyCode::Left => {
                self.move_left();
                Action::Redraw
            }
            KeyCode::Right => {
                self.move_right();
                Action::Redraw
            }
            KeyCode::Up => {
                self.move_up();
                Action::Redraw
            }
            KeyCode::Down => {
                self.move_down();
                Action::Redraw
            }
            KeyCode::Home => {
                self.move_to_line_start();
                Action::Redraw
            }
            KeyCode::End => {
                self.move_to_line_end();
                Action::Redraw
            }
            KeyCode::Tab => {
                for _ in 0..self.tab_size {
                    self.insert_char(' ');
                }
                Action::Redraw
            }
            _ => Action::Continue,
        }
    }

    fn insert_char(&mut self, c: char) {
        let (line, col) = self.cursor;
        let line_start = self.rope.line_to_char(line);
        let pos = line_start + col;

        self.rope.insert_char(pos, c);
        self.move_right();
        self.modified = true;
    }

    fn delete_char(&mut self) {
        let (line, col) = self.cursor;
        if col > 0 {
            let line_start = self.rope.line_to_char(line);
            let pos = line_start + col - 1;
            self.rope.remove(pos..pos + 1);
            self.move_left();
        } else if line > 0 {
            self.join_lines();
        }
        self.modified = true;
    }

    fn delete_next_char(&mut self) {
        let (line, col) = self.cursor;
        let line_start = self.rope.line_to_char(line);
        let pos = line_start + col;
        if pos < self.rope.len_chars() {
            self.rope.remove(pos..pos + 1);
        }
        self.modified = true;
    }

    fn move_left(&mut self) {
        let (line, col) = self.cursor;
        if col > 0 {
            self.cursor.1 -= 1;
        } else if line > 0 {
            let prev_line_len = self.rope.line_to_char(line) - self.rope.line_to_char(line - 1) - 1;
            self.cursor.0 -= 1;
            self.cursor.1 = prev_line_len;
        }
    }

    fn move_right(&mut self) {
        let (line, col) = self.cursor;
        let line_len = self.get_line_len(line);
        if col < line_len {
            self.cursor.1 += 1;
        } else if line + 1 < self.rope.len_lines() {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            let line_len = self.get_line_len(self.cursor.0);
            self.cursor.1 = self.cursor.1.min(line_len);
        }
    }

    fn move_down(&mut self) {
        if self.cursor.0 + 1 < self.rope.len_lines() {
            self.cursor.0 += 1;
            let line_len = self.get_line_len(self.cursor.0);
            self.cursor.1 = self.cursor.1.min(line_len);
        }
    }

    fn move_to_line_start(&mut self) {
        self.cursor.1 = 0;
    }

    fn move_to_line_end(&mut self) {
        self.cursor.1 = self.get_line_len(self.cursor.0);
    }

    fn get_line_len(&self, line: usize) -> usize {
        let mut len = self
            .rope
            .line_to_char(line + 1)
            .saturating_sub(self.rope.line_to_char(line));
        if len > 0 && self.rope.get_char(self.rope.line_to_char(line) + len - 1) == Some('\n') {
            len -= 1;
        }
        len
    }
    fn join_lines(&mut self) {
        let (line, _) = self.cursor;
        if line > 0 {
            let prev_line_end = self.rope.line_to_char(line) - 1;
            let curr_line_start = self.rope.line_to_char(line);
            self.rope.remove(prev_line_end..curr_line_start);
            self.cursor.1 = self.rope.line_to_char(line) - self.rope.line_to_char(line - 1) - 1;
        }
    }

    pub fn get_lines(&self) -> Vec<String> {
        (0..self.rope.len_lines())
            .map(|i| {
                self.rope
                    .slice(self.rope.line_to_char(i)..self.rope.line_to_char(i + 1))
                    .to_string()
            })
            .collect()
    }

    pub fn _is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    pub fn save(&mut self) -> Result<()> {
        std::fs::write(&self.file_path, self.rope.to_string())?;
        self.modified = false;
        Ok(())
    }
    pub fn reload(&mut self) {
        if let Ok(content) = std::fs::read_to_string(&self.file_path)
            && content != self.rope
        {
            self.rope = ropey::Rope::from_str(&content);
            self.cursor = (0, 0);
            self.modified = false;
        }
    }
    pub fn mouse_down(&mut self, x: u16, y: u16) {
        let x = x.saturating_sub(1);
        let y = y.saturating_sub(1);
        self.cursor = (y as usize, x as usize);
        if self.cursor.0 < self.rope.len_lines() {
            let line_len = self.get_line_len(self.cursor.0);
            self.cursor.1 = self.cursor.1.min(line_len);
        } else {
            self.cursor.0 = self.rope.len_lines() - 1;
            let line_len = self.get_line_len(self.cursor.0);
            self.cursor.1 = self.cursor.1.min(line_len);
        }
        self.modified = false;
        self.scroll_to_cursor();
        self.block = false;
        self.block_start = self.cursor;
        self.block_end = self.cursor;
    }
    pub fn mouse_up(&mut self, _x: u16, _y: u16) {
        self.block = false;
    }
    pub fn mouse_drag(&mut self, x: u16, y: u16) {
        let x = x.saturating_sub(1);
        let y = y.saturating_sub(1);
        self.cursor = (y as usize, x as usize);
        if self.cursor.0 < self.rope.len_lines() {
            let line_len = self.get_line_len(self.cursor.0);
            self.cursor.1 = self.cursor.1.min(line_len);
        } else {
            self.cursor.0 = self.rope.len_lines() - 1;
            let line_len = self.get_line_len(self.cursor.0);
            self.cursor.1 = self.cursor.1.min(line_len);
        }
        self.modified = false;
        self.scroll_to_cursor();
        self.block = true;
        self.block_end = self.cursor;
    }

    pub fn _scroll_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.scroll_to_cursor();
        }
    }

    pub fn _scroll_down(&mut self) {
        let line_count = self.rope.len_lines().saturating_sub(1);
        if self.cursor.0 < line_count {
            self.cursor.0 += 1;
            self.scroll_to_cursor();
        }
    }

    fn scroll_to_cursor(&mut self) {
        // TODO: Adjust viewport to keep cursor visible
        // For now: Just move cursor
    }
}
