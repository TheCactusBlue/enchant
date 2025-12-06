use iocraft::{CanvasTextStyle, Color, Component, ComponentDrawer, ComponentUpdater, Hooks, Props};

#[derive(Default, Props)]
pub struct AnsiTextProps {
    pub content: String,
}

/// A run of characters with the same style on a single line
#[derive(Clone)]
struct StyledRun {
    x: usize,
    text: String,
    /// Pre-computed character count for background width
    char_count: usize,
    fg: Option<Color>,
    bg: Option<Color>,
}

#[derive(Default)]
pub struct AnsiText {
    /// Pre-computed styled runs for efficient drawing
    runs: Vec<(usize, Vec<StyledRun>)>, // (y, runs on that line)
    width: usize,
    height: usize,
    last_content: String,
}

#[derive(Default, Clone, Copy, PartialEq)]
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
        // Only re-parse if content has changed
        if props.content == self.last_content {
            return;
        }
        self.last_content = props.content.clone();

        // Convert \e to real ESC (0x1B) if the file uses escaped notation
        let content = props.content.replace("\\e[", "\x1b[");

        // Parse into styled runs for efficient drawing
        let (runs, width, height) = parse_ansi_to_runs(&content);
        self.runs = runs;
        self.width = width;
        self.height = height;

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

        for (y, line_runs) in &self.runs {
            for run in line_runs {
                // Set background color for this run if needed
                if let Some(bg) = run.bg {
                    canvas.set_background_color(run.x as isize, *y as isize, run.char_count, 1, bg);
                }

                // Render the text with foreground color
                let mut style = CanvasTextStyle::default();
                style.color = run.fg;
                canvas.set_text(run.x as isize, *y as isize, &run.text, style);
            }
        }
    }
}

/// Parse ANSI text into styled runs for efficient drawing.
/// Returns (runs, width, height)
fn parse_ansi_to_runs(input: &str) -> (Vec<(usize, Vec<StyledRun>)>, usize, usize) {
    let mut all_runs: Vec<(usize, Vec<StyledRun>)> = Vec::new();
    let mut current_line_runs: Vec<StyledRun> = Vec::new();
    let mut current_run = StyledRun {
        x: 0,
        text: String::new(),
        char_count: 0,
        fg: None,
        bg: None,
    };
    let mut state = AnsiState::default();
    let mut chars = input.chars().peekable();
    let mut x = 0usize;
    let mut y = 0usize;
    let mut max_width = 0usize;

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
                        let new_state = parse_sgr(&params, state);
                        // If style changed, finish current run and start new one
                        if new_state != state {
                            if !current_run.text.is_empty() {
                                current_line_runs.push(current_run);
                                current_run = StyledRun {
                                    x,
                                    text: String::new(),
                                    char_count: 0,
                                    fg: new_state.fg,
                                    bg: new_state.bg,
                                };
                            } else {
                                current_run.fg = new_state.fg;
                                current_run.bg = new_state.bg;
                            }
                            state = new_state;
                        }
                    }
                }
            }
        } else if c == '\n' {
            // Finish current run and line
            if !current_run.text.is_empty() {
                current_line_runs.push(current_run);
            }
            if !current_line_runs.is_empty() {
                all_runs.push((y, current_line_runs));
            }
            max_width = max_width.max(x);
            current_line_runs = Vec::new();
            current_run = StyledRun {
                x: 0,
                text: String::new(),
                char_count: 0,
                fg: state.fg,
                bg: state.bg,
            };
            x = 0;
            y += 1;
        } else if c == '\r' {
            // Ignore carriage returns
        } else {
            current_run.text.push(c);
            current_run.char_count += 1;
            x += 1;
        }
    }

    // Don't forget the last run/line
    if !current_run.text.is_empty() {
        current_line_runs.push(current_run);
    }
    if !current_line_runs.is_empty() {
        all_runs.push((y, current_line_runs));
        max_width = max_width.max(x);
        y += 1;
    }

    (all_runs, max_width, y)
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
