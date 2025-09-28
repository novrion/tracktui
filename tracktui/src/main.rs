use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    symbols,
    prelude::{Position},
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}

#[derive(Default)]
enum InputMode {
    #[default]
    Normal,
    AddingPoint,
}

#[derive(Default)]
enum InputField {
    #[default]
    X,
    Y,
}

#[derive(Default)]
struct App {
    data_points: Vec<(f64, f64)>,
    mode: InputMode,
    input_field: InputField,
    input_x: String,
    input_y: String,
    message: String,
    exit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            message: "Press 'a' to add point, 'q' to quit".to_string(),
            ..Default::default()
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Min(10), // Graph
            Constraint::Length(5), // Input
            Constraint::Length(3), // Status
        ]).split(frame.area());

        // Graph
        self.draw_graph(frame, layout[0]);

        // Input
        self.draw_input(frame, layout[1]);
        
        // Instructions
        let instructions = Paragraph::new("'a' to add point, 'q' to quit")
            .block(Block::bordered());
        frame.render_widget(instructions, layout[2]);
    }

    fn bounds(&self) -> (f64, f64) {
        let mut x_max = f64::NEG_INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        for &(x, y) in &self.data_points {
            x_max = x_max.max(x);
            y_max = y_max.max(y);
        }

        (x_max, y_max)
    }

    fn labels<F>(&self, value_extractor: F) -> Vec<String>
    where
        F: Fn(&(f64, f64)) -> f64,
    {
        let mut labels = Vec::new();

        let max = match self.data_points.last() {
            Some(&point) => value_extractor(&point),
            None => return vec!["0.0".to_string(), "1.0".to_string()]
        };

        let n_labels = std::cmp::min(10, self.data_points.len());
        for i in 0..=n_labels {
            labels.push(format!("{:.1}", i as f64 / n_labels as f64 * max));
        }

        labels
    }


    fn draw_graph(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let dataset = Dataset::default()
            .name("")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(Color::Cyan))
            .data(&self.data_points);

        let (x_max, y_max) = self.bounds();

        let chart = Chart::new(vec![dataset])
            .block(Block::bordered().title("Real-time data"))
            .x_axis(Axis::default()
                .title("Time")
                .bounds([0.0, x_max])
                .labels(self.labels(|&(x, _)| x)))
            .y_axis(Axis::default()
                .title("Level")
                .bounds([0.0, y_max])
                .labels(self.labels(|&(_, y)| y)));

        frame.render_widget(chart, area);
    }

    fn draw_input(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let input_chunks = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(area);

        // X
        let x_style = match (&self.mode, &self.input_field) {
            (InputMode::AddingPoint, InputField::X) => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };

        let x_title = match (&self.mode, &self.input_field) {
            (InputMode::AddingPoint, InputField::X) => "X Coordinate [Active]",
            _ => "X Coordinate",
        };

        let x_input = Paragraph::new(self.input_x.as_str())
            .block(Block::bordered().title(x_title))
            .style(x_style);
        frame.render_widget(x_input, input_chunks[0]);

        // Y
        let y_style = match (&self.mode, &self.input_field) {
            (InputMode::AddingPoint, InputField::Y) => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };

        let y_title = match (&self.mode, &self.input_field) {
            (InputMode::AddingPoint, InputField::Y) => "Y Coordinate [Active]",
            _ => "Y Coordinate",
        };

        let y_input = Paragraph::new(self.input_y.as_str())
            .block(Block::bordered().title(y_title))
            .style(y_style);
        frame.render_widget(y_input, input_chunks[1]);

        
        // Show cursor in input mode
        if matches!(self.mode, InputMode::AddingPoint) {
            let cursor_x = match self.input_field{
                InputField::X => input_chunks[0].x + self.input_x.len() as u16 + 1,
                InputField::Y => input_chunks[0].y + self.input_y.len() as u16 + 1,
            };
            frame.set_cursor_position(Position::new(cursor_x, area.y + 1));
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match self.mode {
                    InputMode::Normal => self.handle_normal_input(key.code),
                    InputMode::AddingPoint => self.handle_adding_point_input(key.code),
                }
            }
        }
        Ok(())
    }

    fn handle_normal_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('a') => {
                self.mode = InputMode::AddingPoint;
                self.input_field = InputField::X;
                self.input_x.clear();
                self.input_y.clear();
                self.message = "Enter X coordinate".to_string();
            }
            _ => {}
        }
    }

    fn handle_adding_point_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-' => {
                match self.input_field {
                    InputField::X => self.input_x.push(c),
                    InputField::Y => self.input_y.push(c),
                }
            }
            KeyCode::Backspace => {
                match self.input_field {
                    InputField::X => { self.input_x.pop(); },
                    InputField::Y => { self.input_y.pop(); },
                }
            }
            KeyCode::Tab => {
                self.input_field = match self.input_field {
                    InputField::X => {
                        self.message = "Enter Y coordinate".to_string();
                        InputField::Y
                    }
                    InputField::Y => {
                        self.message = "Enter X coordinate".to_string();
                        InputField::X
                    }
                };
            }
            KeyCode::Enter => {
                self.try_add_point();
            }
            KeyCode::Esc => {
                self.mode = InputMode::Normal;
                self.input_x.clear();
                self.input_y.clear();
                self.message = "Cancelled".to_string();
            }
            _ => {}
        }
    }

    fn try_add_point(&mut self) {
        match (self.input_x.parse::<f64>(), self.input_y.parse::<f64>()) {
            (Ok(x), Ok(y)) => {
                self.data_points.push((x, y));
                self.data_points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                self.mode = InputMode::Normal;
                self.input_x.clear();
                self.input_y.clear();
                self.message = format!("Added point ({:.2}, {:.2})", x, y);
            }
            _ => {
                self.message = "Error : Enter valid numbers for both X and Y".to_string();
            }
        }
    }
}
