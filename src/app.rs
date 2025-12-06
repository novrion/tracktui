use chrono::Local;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::TableState};

use crate::data_series::DataSeries;
use crate::io::{read_csv, write_csv};
use crate::models::{InputMode, ViewMode};

pub struct App {
    pub mode: ViewMode,
    pub data_series: Vec<DataSeries>,
    pub selected_serie: usize,
    pub input_mode: InputMode,
    pub input: String,
    pub status_msg: String,
    pub table_state: TableState,
    pub confirm: bool,
    pub confirm_idx: usize,
    pub exit: bool,
    pub startup: bool,
    pub startup_questions: Vec<String>,
    pub startup_answers: Vec<u32>,
    pub ans_idx: usize,
    pub startup_question_idx: usize,
}

impl App {
    pub fn new(startup: bool) -> Self {
        Self {
            mode: ViewMode::Menu,
            data_series: Vec::new(),
            selected_serie: 0,
            input_mode: InputMode::Normal,
            input: String::new(),
            status_msg: "h: help".to_string(),
            table_state: TableState::default(),
            confirm: false,
            confirm_idx: 0,
            exit: false,
            startup: startup,
            startup_questions: vec![
                "Sleep".to_string(),
                "Intelligent Work?".to_string(),
                "Exercise?".to_string(),
                "Healthy Food?".to_string(),
            ],
            ans_idx: 0,
            startup_answers: vec![],
            startup_question_idx: 0,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        match read_csv("data.csv") {
            Ok(series) => self.data_series = series,
            Err(e) => {
                self.status_msg = format!("Could not load data.csv: {}", e);
            }
        }

        if self.data_series.is_empty() {
            self.data_series.push(DataSeries::new());
        }
        
        if self.startup {
            self.mode = ViewMode::Question;
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        if let Err(e) = write_csv(&self.data_series, "data.csv") {
            self.status_msg = format!("Could not write to data.csv: {}", e);
            terminal.draw(|frame| self.draw(frame))?;
            event::read()?;
        }

        Ok(())
    }

    pub fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                self.handle_key(key.code);
            }
        }
        Ok(())
    }

    pub fn calculate_score_from_questions(&mut self) -> f64 {
        let ans = &self.startup_answers;
        let w: Vec<f64> = vec![0.15, 0.7, 0.1, 0.05];

        let max_score: f64 = 4.0;
        let mut score: f64 = 0.0;

        for i in 0..ans.len() {
            score += ans[i] as f64 * w[i];
        }

        (1.0 - score / max_score) * 100.0
    }

    pub fn try_insert_point(&mut self, val: Option<f64>) {
        let yesterday = Local::now().date_naive() - chrono::Duration::days(1);
        let timestamp = yesterday
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp();

        let serie = &mut self.data_series[self.selected_serie];

        match val {
            Some(v) => {
                serie.data.push((timestamp, v));
                serie.data.sort_by_key(|&(x, _)| x);
            }
            None => match self.input.parse::<f64>() {
                Ok(v @ 0.0..=100.0) => {
                    serie.data.push((timestamp, v));
                    serie.data.sort_by_key(|&(x, _)| x);

                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                    self.status_msg =
                        format!("Inserted ({}, {}%)", yesterday.format("%Y-%m-%d"), v);
                }
                Ok(_) => {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                    self.status_msg = "Loss must be 0-100%".to_string();
                }
                Err(_) => {
                    self.status_msg = "Invalid number".to_string();
                }
            },
        }
    }
}
