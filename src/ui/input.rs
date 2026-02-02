use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Text input state for add/edit dialogs
#[derive(Debug, Clone, Default)]
pub struct TextInput {
    /// Current input text
    pub value: String,
    /// Cursor position in the string
    pub cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_value(value: String) -> Self {
        let cursor = value.len();
        Self { value, cursor }
    }

    pub fn insert(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += 1;
        }
    }

    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }
}

/// Render a text input dialog
pub fn render_input_dialog(f: &mut Frame, title: &str, input: &TextInput, area: Rect) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Create text with cursor indicator
    let text = if input.cursor < input.value.len() {
        format!(
            "{}|{}",
            &input.value[..input.cursor],
            &input.value[input.cursor..]
        )
    } else {
        format!("{}|", &input.value)
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}
