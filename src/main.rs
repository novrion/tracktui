use std::{error::Error, fs::File};
use serde::{Serialize, Deserialize};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Flex, Rect, Constraint, Layout},
    style::{Color, Style, Modifier, Stylize},
    symbols,
    text::{Span, Text, Line},
    prelude::{Alignment},
    widgets::{Cell, Row, Padding, Clear, Axis, Block, Chart, Dataset, GraphType, Paragraph, Table, TableState},
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
    data_series: Vec<DataSeries>,
    selected_serie: usize,

    // Graph View
    input_mode: InputMode,
    input_field: InputField,
    input_x: String,
    input_y: String,
    status_msg: String,

    // Table View
    table_state: TableState,
    confirm_delete: bool,
    confirm_idx: usize,

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
            status_msg: format!("h: help"),
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
        match self.mode {
            ViewMode::Graph => self.draw_graph_view(frame),
            ViewMode::Menu => self.draw_menu_view(frame),
            ViewMode::Table => self.draw_table_view(frame),
            ViewMode::Help => self.draw_help_view(frame),
        }
    }

    fn draw_menu_view(&self, frame: &mut Frame) {
        let lines = vec![
            Line::from(vec!["h".bold(), "   Help".into()]),
            Line::from(vec!["g".bold(), "   Graph".into()]),
            Line::from(vec!["t".bold(), "   Table".into()]),
            Line::from(vec!["q".bold(), "   Quit".into()]),
        ];

        let area = center(
            frame.area(),
            Constraint::Length(10),
            Constraint::Length(lines.len() as u16),
        );

        let text = Text::from(lines);
        let menu = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(Clear, area);
        frame.render_widget(menu, area);
    }

    fn draw_help_view(&mut self, frame: &mut Frame) {
        let lines = vec![
            Line::from(vec!["h".bold(), "   Help".into()]),
            Line::from(vec!["m".bold(), "   Menu".into()]),
            Line::from(vec!["g".bold(), "   Graph".into()]),
            Line::from(vec!["t".bold(), "   Table".into()]),
            Line::from(vec!["q".bold(), "   Quit".into()]),
            Line::from(""),
            Line::from(vec!["ENTER".bold(), "   Confirm".into()]),
            Line::from(vec!["ESC".bold(), "   Deselect".into()]),
            Line::from(vec!["TAB".bold(), "   Cycle".into()]),
            Line::from(""),
            Line::from(vec!["⇆".bold(), "   Cycle l/r".into()]),
            Line::from(vec!["⇅".bold(), "   Cycle u/d".into()]),
            Line::from(""),
            Line::from(""),
            Line::from(vec!["Graph View".bold().underlined()]),
            Line::from(""),
            Line::from(vec!["i".bold(), "   Insert data".into()]),
            Line::from(""),
            Line::from(""),
            Line::from(vec!["Table View".bold().underlined()]),
            Line::from(""),
            Line::from(vec!["d".bold(), "   Delete".into()]),
        ];

        let area = center(
            frame.area(),
            Constraint::Length(30),
            Constraint::Length(lines.len() as u16 + 2),
        );

        let text = Text::from(lines);
        let help = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(help, area);
    }

    fn draw_table_view(&mut self, frame: &mut Frame) {
        let area = center(
            frame.area(),
            Constraint::Length(20),
            Constraint::Percentage(50),
        );

        let chunks = Layout::vertical(vec![
            Constraint::Min(5),
            Constraint::Length(4),
        ]).split(area);

        self.draw_table(frame, chunks[0]);

        match self.confirm_delete {
            true => {
                let text = Text::from(vec![
                    Line::from(vec!["Delete?".bold()]),
                    Line::from(vec![
                        if self.confirm_idx == 0 { "Yes".bold() }
                        else { "Yes".into() },
                        "  ".into(),
                        if self.confirm_idx == 1 { "No".bold() }
                        else { "No".into() }
                    ]),
                ]);
                let content = Paragraph::new(text).centered();
                frame.render_widget(content, chunks[1]);
            }
            false => {
                let content = Paragraph::new("h: help").centered();
                frame.render_widget(content, chunks[1]);
            }
        }
    }

    fn draw_table(&mut self, frame: &mut Frame, area: Rect) {
        let header = Row::new(vec!["X", "Y"])
            .style(Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD))
            .bottom_margin(1);

        let rows: Vec<Row> = self.data_series[self.selected_serie].data
            .iter()
            .map(|&(x, y)| {
                Row::new(vec![Cell::from(x.to_string()), Cell::from(y.to_string())])
            })
            .collect();

        let widths = [
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::bordered()
                .title("  Table ⇅ ")
                .title_alignment(Alignment::Center)
                .padding(Padding::uniform(2)))
            .column_spacing(1)
            .row_highlight_style(
                Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
            );

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }


    fn draw_graph_view(&mut self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Length(3), // Input
            Constraint::Min(10), // Graph
        ]).split(frame.area());

        // Input
        self.draw_input_bar(frame, chunks[0]);

        // Graph
        self.draw_graph(frame, chunks[1]);
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
            .block(Block::bordered().title(" Status ").padding(Padding::left(1)));
        frame.render_widget(status, input_chunks[2]);
    }

    fn draw_input_box(&mut self, frame: &mut Frame, area: Rect, content: String, title: String, style: Style) {
        let input_box = Paragraph::new(content)
            .block(Block::bordered().title(title).padding(Padding::left(1)))
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

    fn select_previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.data_series[self.selected_serie].data.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn select_next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.data_series[self.selected_serie].data.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn cycle_confirm_idx(&mut self) {
        self.confirm_idx = match self.confirm_idx {
            1 => 0,
            0 => 1,
            _ => 0
        }
    }
    
    fn handle_table_input(&mut self, key: KeyCode) {
        match self.confirm_delete {
            false => {
                match key {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('g') => self.mode = ViewMode::Graph,
                    KeyCode::Char('m') => self.mode = ViewMode::Menu,
                    KeyCode::Char('h') => self.mode = ViewMode::Help,
                    KeyCode::Up | KeyCode::Char('k') => self.select_next(),
                    KeyCode::Down | KeyCode::Char('j') => self.select_previous(), 
                    KeyCode::Char('d') => self.confirm_delete = true,
                    KeyCode::Esc => self.mode = ViewMode::Menu,
                    _ => {}
                }
            },
            true => {
                match key {
                    KeyCode::Esc => self.confirm_delete = false,
                    KeyCode::Left => self.confirm_idx = 0,
                    KeyCode::Right => self.confirm_idx = 1,
                    KeyCode::Tab => self.cycle_confirm_idx(),
                    KeyCode::Enter => {
                        if self.confirm_idx == 0 {
                            if let Some(i) = self.table_state.selected() {
                                self.data_series[self.selected_serie].data.remove(i);
                                self.confirm_delete = false;
                            }
                        } else {
                            self.confirm_delete = false;
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn handle_help_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('g') => self.mode = ViewMode::Graph,
            KeyCode::Char('m') => self.mode = ViewMode::Menu,
            KeyCode::Char('t') => self.mode = ViewMode::Table,
            KeyCode::Esc => self.mode = ViewMode::Menu,
            _ => {}
        }
    }

    fn handle_menu_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('g') => self.mode = ViewMode::Graph,
            KeyCode::Char('t') => self.mode = ViewMode::Table,
            KeyCode::Char('h') => self.mode = ViewMode::Help,
            _ => {}
        }
    }

    fn cycle_field(&mut self) {
        self.input_field = match self.input_field {
            InputField::X => { InputField::Y }
            InputField::Y => { InputField::X }
        };
    }

    fn handle_graph_input(&mut self, key: KeyCode) {
        match self.input_mode {

            InputMode::Normal => {
                match key {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('h') => self.mode = ViewMode::Help,
                    KeyCode::Char('m') => self.mode = ViewMode::Menu,
                    KeyCode::Char('t') => self.mode = ViewMode::Table,
                    KeyCode::Char('i') => {
                        self.input_mode = InputMode::Insert;
                        self.input_field = InputField::X;
                        self.input_x.clear();
                        self.input_y.clear();
                        self.status_msg = format!("h: help");
                    }
                    KeyCode::Esc => self.mode = ViewMode::Menu,
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
                            InputField::X => self.input_x.pop(),
                            InputField::Y => self.input_y.pop(),
                        };
                    }
                    KeyCode::Tab => self.cycle_field(),
                    KeyCode::Enter => {
                        self.cycle_field();
                        if !self.input_y.is_empty() && !self.input_x.is_empty() {
                            self.try_insert_point();
                        }
                    }
                    KeyCode::Left => self.input_field = InputField::X,
                    KeyCode::Right => self.input_field = InputField::Y,
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.input_x.clear();
                        self.input_y.clear();
                        self.status_msg = format!("h: help");
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
