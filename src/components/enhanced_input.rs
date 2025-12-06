use iocraft::prelude::*;
use std::collections::VecDeque;

//   | Feature              | Keybinding               |
//   |----------------------|--------------------------|
//   | Undo                 | Ctrl+Z                   |
//   | Redo                 | Ctrl+Shift+Z or Ctrl+Y   |
//   | Delete word before   | Ctrl+Backspace or Ctrl+W |
//   | Delete word after    | Ctrl+Delete              |
//   | Delete to line start | Ctrl+U                   |
//   | Delete to line end   | Ctrl+K                   |
//   | Word jump left       | Ctrl+Left or Alt+Left    |
//   | Word jump right      | Ctrl+Right or Alt+Right  |
//   | Line start           | Home or Ctrl+A           |
//   | Line end             | End or Ctrl+E            |
//   | Document start       | Ctrl+Home                |
//   | Document end         | Ctrl+End                 |
//   | Submit (multiline)   | Ctrl+Enter               |
//   | Insert tab           | Tab (4 spaces)           |

const MAX_UNDO_HISTORY: usize = 100;

#[derive(Clone, Debug, Default)]
struct EditorState {
    text: String,
    cursor: usize, // byte offset
}

impl EditorState {
    fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }
}

#[derive(Default)]
pub struct TextBuffer {
    current: EditorState,
    undo_stack: VecDeque<EditorState>,
    redo_stack: Vec<EditorState>,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            current: EditorState::new(),
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn text(&self) -> &str {
        &self.current.text
    }

    pub fn cursor(&self) -> usize {
        self.current.cursor
    }

    pub fn set_text(&mut self, text: String) {
        self.save_undo();
        self.current.cursor = text.len().min(self.current.cursor);
        self.current.text = text;
        self.redo_stack.clear();
    }

    fn save_undo(&mut self) {
        self.undo_stack.push_back(self.current.clone());
        if self.undo_stack.len() > MAX_UNDO_HISTORY {
            self.undo_stack.pop_front();
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(state) = self.undo_stack.pop_back() {
            self.redo_stack.push(self.current.clone());
            self.current = state;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push_back(self.current.clone());
            self.current = state;
            true
        } else {
            false
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.save_undo();
        self.current.text.insert(self.current.cursor, c);
        self.current.cursor += c.len_utf8();
        self.redo_stack.clear();
    }

    pub fn insert_str(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.save_undo();
        self.current.text.insert_str(self.current.cursor, s);
        self.current.cursor += s.len();
        self.redo_stack.clear();
    }

    pub fn delete_char_before(&mut self) -> bool {
        if self.current.cursor == 0 {
            return false;
        }
        self.save_undo();
        let prev_char_boundary = self.prev_char_boundary(self.current.cursor);
        self.current
            .text
            .drain(prev_char_boundary..self.current.cursor);
        self.current.cursor = prev_char_boundary;
        self.redo_stack.clear();
        true
    }

    pub fn delete_char_after(&mut self) -> bool {
        if self.current.cursor >= self.current.text.len() {
            return false;
        }
        self.save_undo();
        let next_char_boundary = self.next_char_boundary(self.current.cursor);
        self.current
            .text
            .drain(self.current.cursor..next_char_boundary);
        self.redo_stack.clear();
        true
    }

    pub fn delete_word_before(&mut self) -> bool {
        if self.current.cursor == 0 {
            return false;
        }
        self.save_undo();
        let word_start = self.find_word_start(self.current.cursor);
        self.current.text.drain(word_start..self.current.cursor);
        self.current.cursor = word_start;
        self.redo_stack.clear();
        true
    }

    pub fn delete_word_after(&mut self) -> bool {
        if self.current.cursor >= self.current.text.len() {
            return false;
        }
        self.save_undo();
        let word_end = self.find_word_end(self.current.cursor);
        self.current.text.drain(self.current.cursor..word_end);
        self.redo_stack.clear();
        true
    }

    pub fn delete_to_line_start(&mut self) -> bool {
        if self.current.cursor == 0 {
            return false;
        }
        self.save_undo();
        let line_start = self.find_line_start(self.current.cursor);
        if line_start == self.current.cursor {
            // Delete the newline before
            let prev = self.prev_char_boundary(self.current.cursor);
            self.current.text.drain(prev..self.current.cursor);
            self.current.cursor = prev;
        } else {
            self.current.text.drain(line_start..self.current.cursor);
            self.current.cursor = line_start;
        }
        self.redo_stack.clear();
        true
    }

    pub fn delete_to_line_end(&mut self) -> bool {
        if self.current.cursor >= self.current.text.len() {
            return false;
        }
        self.save_undo();
        let line_end = self.find_line_end(self.current.cursor);
        if line_end == self.current.cursor {
            // Delete the newline after
            let next = self.next_char_boundary(self.current.cursor);
            self.current.text.drain(self.current.cursor..next);
        } else {
            self.current.text.drain(self.current.cursor..line_end);
        }
        self.redo_stack.clear();
        true
    }

    pub fn move_cursor_left(&mut self) -> bool {
        if self.current.cursor == 0 {
            return false;
        }
        self.current.cursor = self.prev_char_boundary(self.current.cursor);
        true
    }

    pub fn move_cursor_right(&mut self) -> bool {
        if self.current.cursor >= self.current.text.len() {
            return false;
        }
        self.current.cursor = self.next_char_boundary(self.current.cursor);
        true
    }

    pub fn move_cursor_word_left(&mut self) -> bool {
        if self.current.cursor == 0 {
            return false;
        }
        self.current.cursor = self.find_word_start(self.current.cursor);
        true
    }

    pub fn move_cursor_word_right(&mut self) -> bool {
        if self.current.cursor >= self.current.text.len() {
            return false;
        }
        self.current.cursor = self.find_word_end(self.current.cursor);
        true
    }

    pub fn move_cursor_home(&mut self) -> bool {
        let line_start = self.find_line_start(self.current.cursor);
        if self.current.cursor != line_start {
            self.current.cursor = line_start;
            true
        } else {
            false
        }
    }

    pub fn move_cursor_end(&mut self) -> bool {
        let line_end = self.find_line_end(self.current.cursor);
        if self.current.cursor != line_end {
            self.current.cursor = line_end;
            true
        } else {
            false
        }
    }

    pub fn move_cursor_up(&mut self) -> bool {
        let line_start = self.find_line_start(self.current.cursor);
        if line_start == 0 {
            return false;
        }
        let col = self.current.cursor - line_start;
        let prev_line_end = line_start - 1; // newline char
        let prev_line_start = self.find_line_start(prev_line_end);
        let prev_line_len = prev_line_end - prev_line_start;
        self.current.cursor = prev_line_start + col.min(prev_line_len);
        true
    }

    pub fn move_cursor_down(&mut self) -> bool {
        let line_start = self.find_line_start(self.current.cursor);
        let line_end = self.find_line_end(self.current.cursor);
        if line_end >= self.current.text.len() {
            return false;
        }
        let col = self.current.cursor - line_start;
        let next_line_start = line_end + 1; // after newline
        let next_line_end = self.find_line_end(next_line_start);
        let next_line_len = next_line_end - next_line_start;
        self.current.cursor = next_line_start + col.min(next_line_len);
        true
    }

    pub fn move_cursor_to_start(&mut self) -> bool {
        if self.current.cursor != 0 {
            self.current.cursor = 0;
            true
        } else {
            false
        }
    }

    pub fn move_cursor_to_end(&mut self) -> bool {
        let end = self.current.text.len();
        if self.current.cursor != end {
            self.current.cursor = end;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        if !self.current.text.is_empty() {
            self.save_undo();
            self.current.text.clear();
            self.current.cursor = 0;
            self.redo_stack.clear();
        }
    }

    fn prev_char_boundary(&self, pos: usize) -> usize {
        let mut pos = pos.saturating_sub(1);
        while pos > 0 && !self.current.text.is_char_boundary(pos) {
            pos -= 1;
        }
        pos
    }

    fn next_char_boundary(&self, pos: usize) -> usize {
        let mut pos = pos + 1;
        while pos < self.current.text.len() && !self.current.text.is_char_boundary(pos) {
            pos += 1;
        }
        pos.min(self.current.text.len())
    }

    fn find_word_start(&self, pos: usize) -> usize {
        if pos == 0 {
            return 0;
        }
        let text = &self.current.text;
        let mut idx = self.prev_char_boundary(pos);

        // Skip whitespace going backwards
        while idx > 0 {
            let c = text[idx..].chars().next().unwrap();
            if !c.is_whitespace() {
                break;
            }
            idx = self.prev_char_boundary(idx);
        }

        // Find start of word
        while idx > 0 {
            let prev_idx = self.prev_char_boundary(idx);
            let c = text[prev_idx..].chars().next().unwrap();
            if c.is_whitespace() || is_word_boundary_char(c) {
                break;
            }
            idx = prev_idx;
        }

        idx
    }

    fn find_word_end(&self, pos: usize) -> usize {
        let text = &self.current.text;
        let len = text.len();
        if pos >= len {
            return len;
        }

        let mut idx = pos;

        // Skip current word
        while idx < len {
            let c = text[idx..].chars().next().unwrap();
            if c.is_whitespace() || is_word_boundary_char(c) {
                break;
            }
            idx = self.next_char_boundary(idx);
        }

        // Skip whitespace
        while idx < len {
            let c = text[idx..].chars().next().unwrap();
            if !c.is_whitespace() {
                break;
            }
            idx = self.next_char_boundary(idx);
        }

        idx
    }

    fn find_line_start(&self, pos: usize) -> usize {
        let text = &self.current.text;
        if pos == 0 {
            return 0;
        }
        // Search backwards for newline
        text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0)
    }

    fn find_line_end(&self, pos: usize) -> usize {
        let text = &self.current.text;
        // Search forwards for newline
        text[pos..]
            .find('\n')
            .map(|i| pos + i)
            .unwrap_or(text.len())
    }
}

fn is_word_boundary_char(c: char) -> bool {
    matches!(
        c,
        '(' | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '<'
            | '>'
            | '.'
            | ','
            | ';'
            | ':'
            | '"'
            | '\''
            | '`'
            | '/'
            | '\\'
            | '|'
            | '!'
            | '?'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '-'
            | '+'
            | '='
            | '~'
    )
}

#[derive(Default, Props)]
pub struct EnhancedInputProps {
    pub value: String,
    pub on_change: HandlerMut<'static, String>,
    pub on_submit: HandlerMut<'static, String>,
    pub color: Option<Color>,
    pub cursor_color: Option<Color>,
    pub multiline: bool,
    /// If true, Shift+Enter inserts newline and Enter submits.
    /// If false (default in multiline), Enter inserts newline and Ctrl+Enter submits.
    pub submit_on_enter: bool,
    pub has_focus: bool,
}

#[component]
pub fn EnhancedInput(
    mut hooks: Hooks,
    props: &mut EnhancedInputProps,
) -> impl Into<AnyElement<'static>> {
    let mut buffer = hooks.use_ref(|| TextBuffer::new());
    let text_input_handle = hooks.use_ref(TextInputHandle::default);
    let mut on_change = props.on_change.take();
    let mut on_submit = props.on_submit.take();
    let multiline = props.multiline;
    let submit_on_enter = props.submit_on_enter;

    // Sync external value changes to buffer
    {
        let mut buf = buffer.write();
        if buf.text() != props.value {
            buf.set_text(props.value.clone());
        }
    }

    hooks.use_terminal_events({
        let mut buffer = buffer.clone();
        let mut text_input_handle = text_input_handle.clone();
        move |event| {
            if let TerminalEvent::Key(KeyEvent {
                kind,
                code,
                modifiers,
                ..
            }) = event
            {
                if kind == KeyEventKind::Release {
                    return;
                }

                let ctrl = modifiers.contains(KeyModifiers::CONTROL);
                let shift = modifiers.contains(KeyModifiers::SHIFT);
                let alt = modifiers.contains(KeyModifiers::ALT);

                let mut buf = buffer.write();
                let mut changed = false;

                // Enter key handling depends on mode:
                // - submit_on_enter: Enter submits, Shift+Enter inserts newline
                // - !submit_on_enter (multiline default): Enter inserts newline, Ctrl+Enter submits
                // - single-line: Enter always submits
                match code {
                    // Submit conditions
                    KeyCode::Enter if !multiline => {
                        let text = buf.text().to_string();
                        drop(buf);
                        (on_submit)(text);
                        return;
                    }
                    KeyCode::Enter if submit_on_enter && !shift => {
                        let text = buf.text().to_string();
                        drop(buf);
                        (on_submit)(text);
                        return;
                    }
                    KeyCode::Enter if !submit_on_enter && ctrl => {
                        let text = buf.text().to_string();
                        drop(buf);
                        (on_submit)(text);
                        return;
                    }
                    // Newline in multiline mode
                    KeyCode::Enter if multiline => {
                        buf.insert_char('\n');
                        changed = true;
                    }
                    // Undo
                    KeyCode::Char('z') if ctrl && !shift => {
                        changed = buf.undo();
                    }
                    // Redo (Ctrl+Shift+Z or Ctrl+Y)
                    KeyCode::Char('z') if ctrl && shift => {
                        changed = buf.redo();
                    }
                    KeyCode::Char('y') if ctrl => {
                        changed = buf.redo();
                    }
                    // Delete word before cursor (Ctrl+Backspace or Ctrl+W)
                    KeyCode::Backspace if ctrl => {
                        changed = buf.delete_word_before();
                    }
                    KeyCode::Char('w') if ctrl => {
                        changed = buf.delete_word_before();
                    }
                    // Delete word after cursor (Ctrl+Delete)
                    KeyCode::Delete if ctrl => {
                        changed = buf.delete_word_after();
                    }
                    // Delete to line start (Ctrl+U)
                    KeyCode::Char('u') if ctrl => {
                        changed = buf.delete_to_line_start();
                    }
                    // Delete to line end (Ctrl+K)
                    KeyCode::Char('k') if ctrl => {
                        changed = buf.delete_to_line_end();
                    }
                    // Regular backspace
                    KeyCode::Backspace => {
                        changed = buf.delete_char_before();
                    }
                    // Regular delete
                    KeyCode::Delete => {
                        changed = buf.delete_char_after();
                    }
                    // Word movement (Ctrl+Left/Right or Alt+Left/Right)
                    KeyCode::Left if ctrl || alt => {
                        buf.move_cursor_word_left();
                    }
                    KeyCode::Right if ctrl || alt => {
                        buf.move_cursor_word_right();
                    }
                    // Regular cursor movement
                    KeyCode::Left => {
                        buf.move_cursor_left();
                    }
                    KeyCode::Right => {
                        buf.move_cursor_right();
                    }
                    KeyCode::Up if multiline => {
                        buf.move_cursor_up();
                    }
                    KeyCode::Down if multiline => {
                        buf.move_cursor_down();
                    }
                    // Home/End
                    KeyCode::Home if ctrl => {
                        buf.move_cursor_to_start();
                    }
                    KeyCode::End if ctrl => {
                        buf.move_cursor_to_end();
                    }
                    KeyCode::Home => {
                        buf.move_cursor_home();
                    }
                    KeyCode::End => {
                        buf.move_cursor_end();
                    }
                    // Emacs-style bindings
                    KeyCode::Char('a') if ctrl => {
                        buf.move_cursor_home();
                    }
                    KeyCode::Char('e') if ctrl => {
                        buf.move_cursor_end();
                    }
                    // Character input
                    KeyCode::Char(c) if !ctrl => {
                        buf.insert_char(c);
                        changed = true;
                    }
                    KeyCode::Tab => {
                        buf.insert_str("    ");
                        changed = true;
                    }
                    _ => {}
                }

                // Update cursor position in the TextInput handle
                let cursor = buf.cursor();
                text_input_handle.write().set_cursor_offset(cursor);

                if changed {
                    let text = buf.text().to_string();
                    drop(buf);
                    (on_change)(text);
                }
            }
        }
    });

    let value = buffer.read().text().to_string();

    element! {
        TextInput(
            has_focus: props.has_focus,
            value: value,
            color: props.color,
            cursor_color: props.cursor_color,
            multiline: props.multiline,
            handle: Some(text_input_handle),
            on_change: move |_new_value| {
                // Ignore external changes from TextInput - we manage state ourselves
            },
        )
    }
}
