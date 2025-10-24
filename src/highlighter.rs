use std::sync::Arc;
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::{ParseState, SyntaxSet},
};

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_syntax: Option<Arc<syntect::parsing::SyntaxReference>>,
    parse_state: ParseState,
}

impl Highlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let binding = syntax_set.clone();
        let plain_syntax = binding.find_syntax_plain_text();

        Self {
            syntax_set,
            theme_set,
            current_syntax: Some(Arc::new(plain_syntax.clone())),
            parse_state: ParseState::new(plain_syntax),
        }
    }

    pub fn set_language(&mut self, extension: &str) {
        self.current_syntax = self
            .syntax_set
            .find_syntax_by_extension(extension)
            .or_else(|| Some(self.syntax_set.find_syntax_plain_text()))
            .map(|s| Arc::new(s.clone()));

        if let Some(syntax) = &self.current_syntax {
            self.parse_state = ParseState::new(syntax.as_ref());
        }
    }

    pub fn highlight_line<'a>(&mut self, line: &'a str) -> Vec<(Style, &'a str)> {
        if let Some(syntax) = &self.current_syntax {
            let mut highlight_lines =
                HighlightLines::new(syntax.as_ref(), &self.theme_set.themes["base16-ocean.dark"]);
            highlight_lines
                .highlight_line(line, &self.syntax_set)
                .expect("Highlighting failed")
        } else {
            vec![(Style::default(), line)]
        }
    }
}
