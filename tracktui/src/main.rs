use std::{error::Error, fs::File};
use serde::{Serialize, Deserialize};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Flex, Rect, Constraint, Layout},
    style::{Color, Style, Modifier},
    symbols,
    text::{Span},
    prelude::{Alignment},
    widgets::{Clear, Axis, Block, Chart, Dataset, GraphType, Paragraph},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}

#[derive(Default)]#[allow(dead_code)]
enum ViewMode {
    #[default]
    Graph,
    Table,
    Menu,
    Help,
}

#[derive(Default)]
enum InputMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Default)]
enum InputField {
    #[default]
    X,
    Y,
}

#[derive(Default)]
struct App {
    
    mode: ViewMode,
    clrscr: bool,

    data_series: Vec<DataSeries>,
    selected_serie: usize,

    // Input
    input_mode: InputMode,
    input_field: InputField,
    input_x: String,
    input_y: String,
    status_msg: String,

    exit: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct DataSeries {
    name: String,
    data: Vec<(f64, f64)>,
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

impl DataSeries {
    fn new() -> Self {
        Self {
            name: "Graph".to_string(),
            ..Default::default()
        }
    }

    fn get_bounds(&self) -> (f64, f64) {
        if self.data.is_empty() {
            return (1.0, 1.0)
        }

        let mut x_max = f64::NEG_INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for &(x, y) in &self.data {
            x_max = x_max.max(x);
            y_max = y_max.max(y);
        }
        (x_max, y_max)
    }

    fn get_labels(&self) -> (Vec<Span<'_>>, Vec<Span<'_>>) {
        let mut x_labels = Vec::new();
        let mut y_labels = Vec::new();
        let (x_max, y_max) = self.get_bounds();
        let n_labels = std::cmp::min(5, self.data.len());

        if n_labels == 0 {
            return (vec![], vec![]);
        }

        for i in 0..=n_labels {
            x_labels.push(Span::styled(format!("{:.2}", i as f64 / n_labels as f64 * x_max), Style::default().add_modifier(Modifier::BOLD)));
            y_labels.push(Span::styled(format!("{:.2}", i as f64 / n_labels as f64 * y_max), Style::default().add_modifier(Modifier::BOLD)));
        }

        (x_labels, y_labels)
    }
}

impl App {
    fn new() -> Self {
        Self {
            mode: ViewMode::Graph,
            selected_serie: 0,
            ..Default::default()
        }
    }
    
    fn write_csv(&mut self, path: String) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = csv::Writer::from_writer(file);
        
        wtr.write_record(&["name", "x", "y"])?;
        
        // Flatten: write each data point as a separate row
        for serie in &self.data_series {
            for &(x, y) in &serie.data {
                wtr.write_record(&[
                    serie.name.as_str(),
                    &x.to_string(),
                    &y.to_string(),
                ])?;
            }
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    fn read_csv(&mut self, path: String) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        use std::collections::HashMap;
        let mut series_map: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
        
        for result in rdr.records() {
            let record = result?;
            let name = record.get(0).ok_or("Missing name")?.to_string();
            let x: f64 = record.get(1).ok_or("Missing x")?.parse()?;
            let y: f64 = record.get(2).ok_or("Missing y")?.parse()?;
            
            series_map.entry(name).or_insert_with(Vec::new).push((x, y));
        }
        
        // Convert HashMap to Vec<DataSeries>
        for (name, mut data) in series_map {
            data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            self.data_series.push(DataSeries { name, data });
        }
        
        Ok(())
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {

        // Read csv
        if let Err(e) = self.read_csv("data.csv".to_string()) {
            self.status_msg = format!("Could not load data.csv: {}", e);
            self.data_series.push(DataSeries::new());
        }

        // Add series if none
        if self.data_series.is_empty() {
            self.data_series.push(DataSeries::new());
        }
        
        // Main loop
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        // Write csv
        if let Err(e) = self.write_csv("data.csv".to_string()) {
            self.status_msg = format!("Could not write to data.csv (Press any ket to exit): {}", e);
            terminal.draw(|frame| self.draw(frame))?;
            event::read()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        if self.clrscr {
            let background = Block::default().style(Style::default().bg(Color::Reset));
            frame.render_widget(background, frame.area());
            self.clrscr = false;
        }

        match self.mode {
            ViewMode::Graph => self.draw_graph_view(frame),
            ViewMode::Menu => self.draw_menu_view(frame),
            ViewMode::Table => self.draw_table_view(frame),
            ViewMode::Help => self.draw_help_view(frame),
        }
    }

    fn draw_menu_view(&self, frame: &mut Frame) {
        let lines = vec![
            "h\tHelp",
            "g\tGraph",
            "t\tTable",
            "q\tQuit",
        ];

        let area = center(
            frame.area(),
            Constraint::Length(15),
            Constraint::Length(lines.len() as u16 + 2)
        );

        let mut text = String::new();
        for line in lines {
            text += line;
            text += "\n";
        }

        let menu = Paragraph::new(text).block(Block::bordered());

        frame.render_widget(menu, area)
    }


    fn draw_table_view(&mut self, frame: &mut Frame) {
    }

    fn draw_help_view(&mut self, frame: &mut Frame) {

    }


    fn draw_graph_view(&mut self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Length(3), // Input
            Constraint::Min(10), // Graph
            Constraint::Length(3), // Instructions
        ]).split(frame.area());

        // Input
        self.draw_input_bar(frame, chunks[0]);

        // Graph
        self.draw_graph(frame, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Press 'i' to insert data, <TAB> to cycle between x and y, 'q' to quit")
            .block(Block::bordered().title(" Instructions "));
        frame.render_widget(instructions, chunks[2]);
    }

    fn draw_input_bar(&mut self, frame: &mut Frame, area: Rect) {
        let input_chunks = Layout::horizontal([
            Constraint::Length(8), // X
            Constraint::Length(8), // Y
            Constraint::Min(20), // Status
        ]).split(area);

        // X
        let x_style = match (&self.input_mode, &self.input_field) {
            (InputMode::Insert, InputField::X) => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };
        self.draw_input_box(frame, input_chunks[0], self.input_x.clone(), format!(" X "), x_style);

        // Y
        let y_style = match (&self.input_mode, &self.input_field) {
            (InputMode::Insert, InputField::Y) => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };
        self.draw_input_box(frame, input_chunks[1], self.input_y.clone(), format!(" Y "), y_style);

        // Status
        let status = Paragraph::new(self.status_msg.clone())
            .block(Block::bordered().title(" Status "));
        frame.render_widget(status, input_chunks[2]);
    }

    fn draw_input_box(&mut self, frame: &mut Frame, area: Rect, content: String, title: String, style: Style) {
        let input_box = Paragraph::new(content)
            .block(Block::bordered().title(title))
            .style(style);
            
        frame.render_widget(input_box, area);
    }

    fn draw_graph(&mut self, frame: &mut Frame, area: Rect) {
        let serie = &self.data_series[self.selected_serie];
        let dataset = Dataset::default()
            .name("")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&serie.data);

        let (x_max, y_max) = serie.get_bounds();
        let (x_labels, y_labels) = serie.get_labels();

        let chart = Chart::new(vec![dataset])
            .block(Block::bordered()
                .title(format!(" {} ", serie.name))
                .title_alignment(Alignment::Center))
            .x_axis(Axis::default()
                .title("X")
                .bounds([0.0, x_max])
                .labels(x_labels))
            .y_axis(Axis::default()
                .title("Y")
                .bounds([0.0, y_max])
                .labels(y_labels));

        frame.render_widget(chart, area);
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match self.mode {
                    ViewMode::Graph => self.handle_graph_input(key.code),
                    ViewMode::Table => self.handle_table_input(key.code),
                    ViewMode::Menu => self.handle_menu_input(key.code),
                    ViewMode::Help => self.handle_help_input(key.code),
                }
            }
        }
        Ok(())
    }
    
    fn handle_table_input(&mut self, key: KeyCode) {

    }

    fn handle_help_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('g') => self.set_mode(ViewMode::Graph),
            KeyCode::Char('m') => self.set_mode(ViewMode::Menu),
            KeyCode::Char('t') => self.set_mode(ViewMode::Table),
            _ => {}
        }
    }

    fn handle_menu_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('g') => self.set_mode(ViewMode::Graph),
            KeyCode::Char('t') => self.set_mode(ViewMode::Table),
            KeyCode::Char('h') => self.set_mode(ViewMode::Help),
            _ => {}
        }
    }

    fn set_mode(&mut self, mode: ViewMode) {
        self.clrscr = true;
        self.mode = mode;
    }

    fn handle_graph_input(&mut self, key: KeyCode) {
        match self.input_mode {

            InputMode::Normal => {
                match key {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('h') => self.set_mode(ViewMode::Help),
                    KeyCode::Char('m') => self.set_mode(ViewMode::Menu),
                    KeyCode::Char('t') => self.set_mode(ViewMode::Table),
                    KeyCode::Char('i') => {
                        self.input_mode = InputMode::Insert;
                        self.input_field = InputField::X;
                        self.input_x.clear();
                        self.input_y.clear();
                        self.status_msg.clear();
                    }
                    _ => {}
                }
            }

            InputMode::Insert => {
                match key {
                    KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-'=> {
                        match self.input_field {
                            InputField::X => {
                                if self.input_x.len() < 5 {
                                    self.input_x.push(c);
                                }
                            },
                            InputField::Y => {
                                if self.input_y.len() < 5 {
                                    self.input_y.push(c);
                                }
                            },
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
                            InputField::X => { InputField::Y }
                            InputField::Y => { InputField::X }
                        };
                    }
                    KeyCode::Enter => {
                        self.try_insert_point();
                    }
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.input_x.clear();
                        self.input_y.clear();
                        self.status_msg.clear();
                    }
                    _ => {}
                }
            }
        }
    }

    fn try_insert_point(&mut self) {
        match (self.input_x.parse::<f64>(), self.input_y.parse::<f64>()) {
            (Ok(x), Ok(y)) => {
                let serie = &mut self.data_series[self.selected_serie];
                serie.data.push((x, y));
                serie.data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                self.input_mode = InputMode::Normal;
                self.input_x.clear();
                self.input_y.clear();
                self.status_msg = format!("Inserted point ({:.2}, {:.2})", x, y);
            }
            _ => {
                self.status_msg = "Error: enter valid numbers for x and y".to_string();
            }
        }
    }
}
