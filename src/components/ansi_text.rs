use iocraft::{CanvasTextStyle, Color, Component, ComponentDrawer, ComponentUpdater, Hooks, Props};

#[derive(Default, Props)]
pub struct AnsiTextProps {
    pub content: String,
}

#[derive(Default)]
pub struct AnsiText {
    lines: Vec<Vec<AnsiCell>>,
    width: usize,
    height: usize,
}

#[derive(Default, Clone, Copy)]
struct AnsiCell {
    char: char,
    fg: Option<Color>,
    bg: Option<Color>,
}

#[derive(Default, Clone, Copy)]
struct AnsiState {
    fg: Option<Color>,
    bg: Option<Color>,
}

impl Component for AnsiText {
    type Props<'a> = AnsiTextProps;

    fn new(_props: &Self::Props<'_>) -> Self {
        Self::default()
    }

    fn update(
        &mut self,
        props: &mut Self::Props<'_>,
        _hooks: Hooks,
        updater: &mut ComponentUpdater,
    ) {
        // Convert \e to real ESC (0x1B) if the file uses escaped notation
        let content = props.content.replace("\\e[", "\x1b[");

        // Parse into cells
        self.lines = parse_ansi_to_cells(&content);
        self.height = self.lines.len();
        self.width = self.lines.iter().map(|l| l.len()).max().unwrap_or(0);

        updater.set_measure_func(Box::new({
            let width = self.width;
            let height = self.height;
            move |_, _, _| taffy::Size {
                width: width as f32,
                height: height as f32,
            }
        }));
    }

    fn draw(&mut self, drawer: &mut ComponentDrawer<'_>) {
        let mut canvas = drawer.canvas();

        for (y, line) in self.lines.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                // Set background color for this cell if needed
                if let Some(bg) = cell.bg {
                    canvas.set_background_color(x as isize, y as isize, 1, 1, bg);
                }

                // Render the character with foreground color
                if cell.char != '\0' && cell.char != ' ' || cell.bg.is_some() {
                    let mut style = CanvasTextStyle::default();
                    style.color = cell.fg;
                    let s = if cell.char == '\0' {
                        " "
                    } else {
                        &cell.char.to_string()
                    };
                    canvas.set_text(x as isize, y as isize, s, style);
                }
            }
        }
    }
}

fn parse_ansi_to_cells(input: &str) -> Vec<Vec<AnsiCell>> {
    let mut lines: Vec<Vec<AnsiCell>> = Vec::new();
    let mut current_line: Vec<AnsiCell> = Vec::new();
    let mut state = AnsiState::default();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Parse escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                let mut params = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == ';' {
                        params.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                // Consume the final character (usually 'm')
                if let Some(cmd) = chars.next() {
                    if cmd == 'm' {
                        state = parse_sgr(&params, state);
                    }
                }
            }
        } else if c == '\n' {
            lines.push(current_line);
            current_line = Vec::new();
        } else if c == '\r' {
            // Ignore carriage returns
        } else {
            current_line.push(AnsiCell {
                char: c,
                fg: state.fg,
                bg: state.bg,
            });
        }
    }

    // Don't forget the last line if it doesn't end with newline
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Parse SGR (Select Graphic Rendition) parameters
fn parse_sgr(params: &str, mut state: AnsiState) -> AnsiState {
    if params.is_empty() {
        return AnsiState::default();
    }

    let parts: Vec<&str> = params.split(';').collect();
    let mut i = 0;

    while i < parts.len() {
        match parts[i] {
            "0" => state = AnsiState::default(),
            "38" => {
                // Foreground color
                if i + 1 < parts.len() && parts[i + 1] == "2" {
                    // 24-bit RGB: 38;2;r;g;b
                    if i + 4 < parts.len() {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            parts[i + 2].parse::<u8>(),
                            parts[i + 3].parse::<u8>(),
                            parts[i + 4].parse::<u8>(),
                        ) {
                            state.fg = Some(Color::Rgb { r, g, b });
                        }
                        i += 4;
                    }
                } else if i + 1 < parts.len() && parts[i + 1] == "5" {
                    // 256-color: 38;5;n
                    if i + 2 < parts.len() {
                        if let Ok(n) = parts[i + 2].parse::<u8>() {
                            state.fg = Some(Color::AnsiValue(n));
                        }
                        i += 2;
                    }
                }
            }
            "48" => {
                // Background color
                if i + 1 < parts.len() && parts[i + 1] == "2" {
                    // 24-bit RGB: 48;2;r;g;b
                    if i + 4 < parts.len() {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            parts[i + 2].parse::<u8>(),
                            parts[i + 3].parse::<u8>(),
                            parts[i + 4].parse::<u8>(),
                        ) {
                            state.bg = Some(Color::Rgb { r, g, b });
                        }
                        i += 4;
                    }
                } else if i + 1 < parts.len() && parts[i + 1] == "5" {
                    // 256-color: 48;5;n
                    if i + 2 < parts.len() {
                        if let Ok(n) = parts[i + 2].parse::<u8>() {
                            state.bg = Some(Color::AnsiValue(n));
                        }
                        i += 2;
                    }
                }
            }
            "39" => state.fg = None, // Default foreground
            "49" => state.bg = None, // Default background
            _ => {}
        }
        i += 1;
    }

    state
}
