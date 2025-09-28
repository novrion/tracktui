use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();
    result
}

#[derive(Default)]
struct App {
    data_points: Vec<(f64, f64)>,
    exit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.data_points = vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5)];

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Min(10),
            Constraint::Length(3)
        ]).split(frame.area());

        let dataset = Dataset::Default()
            .name("Data Points")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style("cyan".)
    }
}
