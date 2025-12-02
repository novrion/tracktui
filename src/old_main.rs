use std::{error::Error, fs::File};
use chrono::{Local, TimeZone};
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
            ..Default::default()
        }
    }

    fn get_bounds(&self) -> (f64, f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 1.0, 0.0, 1.0)
        }

        let x_min = self.data[0].0 as f64;
        let mut x_max = i64::MIN;
        for &(x, _y) in &self.data {
            x_max = x_max.max(x);
        }

        let y_min = 0.0;
        let y_max = 100.0;

        (x_min, x_max as f64, y_min, y_max)
    }

    fn get_labels(&self) -> (Vec<Span<'_>>, Vec<Span<'_>>) {
        let mut x_labels = Vec::new();
        let mut y_labels = Vec::new();

        let data_len = self.data.len();
        if data_len == 0 {
            return (vec![], vec![]);
        }

        // x_labels
        let mut idxs = vec![0];
        if data_len > 2 {
            idxs.push(data_len-1);
        }
        for i in idxs {
            if let Some(dt) = Local.timestamp_opt(self.data[i].0, 0).single() {
                x_labels.push(
                    Span::styled(dt.format("%Y-%m-%d").to_string(),
                        Style::default().add_modifier(Modifier::BOLD)
                    )
                );
            }
        }

        // y_labels
        for i in 0..11 {
            y_labels.push(Span::styled(format!("{}%", i * 10),
                          Style::default().add_modifier(Modifier::BOLD))
            );
        }

        (x_labels, y_labels)
    }

    fn get_average_loss(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }

        let mut sum = 0.0;
        let mut count = 0;
        for i in (0..self.data.len()).rev() {
            sum += self.data[i].1;
            count += 1;
            if count >= 20 {
                break;
            }
        }

        sum / count as f64
    }
}

impl App {
    fn new() -> Self {
        Self {
            mode: ViewMode::Menu,
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
        let mut series_map: HashMap<String, Vec<(i64, f64)>> = HashMap::new();
        
        for result in rdr.records() {
            let record = result?;
            let name = record.get(0).ok_or("Missing name")?.to_string();
            let x: i64 = record.get(1).ok_or("Missing x")?.parse()?;
            let y: f64 = record.get(2).ok_or("Missing y")?.parse()?;
            
            series_map.entry(name).or_insert_with(Vec::new).push((x, y));
        }
        
        // Convert HashMap to Vec<DataSeries>
        for (name, mut data) in series_map {
            data.sort_by(|a, b| a.0.cmp(&b.0));
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
        let chunks = Layout::vertical(vec![
            Constraint::Percentage(10), // Padding
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ]).split(frame.area());

        // Title
        let title_text = "                                                               ▄▄
███▀▀██▀▀███                      ▀███        ██               ██
█▀   ██   ▀█                        ██        ██
     ██    ▀███▄███ ▄█▀██▄  ▄██▀██  ██  ▄██▀██████▀███  ▀███ ▀███
     ██      ██▀ ▀▀██   ██ ██▀  ██  ██ ▄█     ██    ██    ██   ██
     ██      ██     ▄█████ ██       ██▄██     ██    ██    ██   ██
     ██      ██    ██   ██ ██▄    ▄ ██ ▀██▄   ██    ██    ██   ██
   ▄████▄  ▄████▄  ▀████▀██▄█████▀▄████▄ ██▄▄ ▀████ ▀████▀███▄████▄";

        let title_area = center(
            chunks[1],
            Constraint::Length(67),
            Constraint::Length(10)
        );
        let title = Paragraph::new(title_text);

        // Menu
        let lines = vec![
            Line::from(vec!["h".bold(), "   Help".into()]),
            Line::from(vec!["g".bold(), "   Graph".into()]),
            Line::from(vec!["t".bold(), "   Table".into()]),
            Line::from(vec!["q".bold(), "   Quit".into()]),
        ];

        let menu_area = center(
            chunks[2],
            Constraint::Length(10),
            Constraint::Length(lines.len() as u16),
        );

        let menu_text = Text::from(lines);
        let menu = Paragraph::new(menu_text).alignment(Alignment::Center);

        // Render
        frame.render_widget(Clear, title_area);
        frame.render_widget(Clear, menu_area);
        frame.render_widget(title, title_area);
        frame.render_widget(menu, menu_area);
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
            Line::from(vec!["ESC  ".bold(), "   Deselect".into()]),
            Line::from(vec!["TAB  ".bold(), "   Cycle".into()]),
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
            Constraint::Length(16),
            Constraint::Length(lines.len() as u16 + 2),
        );

        let text = Text::from(lines);
        let help = Paragraph::new(text);
        frame.render_widget(help, area);
    }

    fn draw_table_view(&mut self, frame: &mut Frame) {
        let area = center(
            frame.area(),
            Constraint::Length(35),
            Constraint::Percentage(70),
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
        let header = Row::new(vec!["Date", "Loss"])
            .style(Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD))
            .bottom_margin(1);

        let rows: Vec<Row> = self.data_series[self.selected_serie].data
            .iter()
            .map(|&(x, y)| {
                let mut x_str = x.to_string();
                if let Some(dt) = Local.timestamp_opt(x, 0).single() {
                    x_str = dt.format("%Y-%m-%d").to_string();
                }
                Row::new(vec![Cell::from(x_str), Cell::from(format!("{}%", y))])
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
            Constraint::Length(12), // Input
            Constraint::Min(20), // Status
            Constraint::Length(12), // Average Loss
        ]).split(area);

        // Input
        let input_style = match &self.input_mode {
            InputMode::Insert => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };
        self.draw_input_box(frame, input_chunks[0], self.input.clone(), format!(" Input "), input_style);

        // Status
        let status = Paragraph::new(self.status_msg.clone())
            .block(Block::bordered().title(" Status ").padding(Padding::left(1)));
        frame.render_widget(status, input_chunks[1]);

        // Average Loss
        let serie = &self.data_series[self.selected_serie];
        let avg = Paragraph::new(format!("{:.1}%", serie.get_average_loss()))
            .block(Block::bordered().title(" 20d avg "))
            .alignment(Alignment::Center);
        frame.render_widget(avg, input_chunks[2]);
    }

    fn draw_input_box(&mut self, frame: &mut Frame, area: Rect, content: String, title: String, style: Style) {
        let input_box = Paragraph::new(content)
            .block(Block::bordered().title(title).padding(Padding::left(1)))
            .alignment(Alignment::Center)
            .style(style);
            
        frame.render_widget(input_box, area);
    }

    fn draw_graph(&mut self, frame: &mut Frame, area: Rect) {
        let serie = &self.data_series[self.selected_serie];
        let data: Vec<(f64, f64)> = serie.data.iter().map(|&(x, y)| (x as f64, y)).collect();
        let dataset = Dataset::default()
            .name("")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&data);

        let (x_min, x_max, y_min, y_max) = serie.get_bounds();
        let (x_labels, y_labels) = serie.get_labels();

        let title = match serie.name.len() {
            0 => "".to_string(),
            _ => format!(" {} ", serie.name)
        };

        let chart = Chart::new(vec![dataset])
            .block(Block::bordered()
                .title(title)
                .title_alignment(Alignment::Center))
            .x_axis(Axis::default()
                .title("")
                .bounds([x_min, x_max as f64])
                .labels(x_labels))
            .y_axis(Axis::default()
                .title("")
                .bounds([y_min, y_max])
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
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Tab => self.select_next(),
                    KeyCode::Down | KeyCode::Char('j') => self.select_previous(),
                    KeyCode::Char('d') => {
                        if self.table_state.selected().is_some() {
                            self.confirm_delete = true;
                        }
                    },
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
                        self.input.clear();
                        self.status_msg = format!("h: help");
                    }
                    KeyCode::Esc => self.mode = ViewMode::Menu,
                    _ => {}
                }
            }

            InputMode::Insert => {
                match key {
                    KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-'=> {
                        if self.input.len() < 5 {
                            self.input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    KeyCode::Enter => {
                        if !self.input.is_empty() {
                            self.try_insert_point();
                        }
                    }
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.input.clear();
                        self.status_msg = format!("h: help");
                    }
                    _ => {}
                }
            }
        }
    }

    fn try_insert_point(&mut self) {
        match self.input.parse::<f64>() {
            Ok(val) => {
                match val {
                    0.0..=100.0 => {
                        let serie = &mut self.data_series[self.selected_serie];

                        let yesterday = Local::now().date_naive() - chrono::Duration::days(1);
                        let timestamp = yesterday.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap().timestamp();

                        serie.data.push((timestamp, val));
                        serie.data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                        self.input_mode = InputMode::Normal;
                        self.input.clear();
                        self.status_msg = format!("Inserted point ({}, {}%)", yesterday.format("%Y-%m-%d"), val);
                    }
                    _ => {
                        self.input_mode = InputMode::Normal;
                        self.input.clear();
                        self.status_msg = format!("Loss must be between 0 and 100%")
                    }
                }
            }
            _ => {
                self.status_msg = "Error: enter a valid number between 0 and 100".to_string();
            }
        }
    }
}
