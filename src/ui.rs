use chrono::{Local, TimeZone};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Axis, Block, Cell, Chart, Clear, Dataset, GraphType, Padding, Paragraph, Row, Table,
    },
};

use crate::app::App;
use crate::models::ViewMode;

pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        match self.mode {
            ViewMode::Graph => self.draw_graph_view(frame),
            ViewMode::Menu => self.draw_menu_view(frame),
            ViewMode::Table => self.draw_table_view(frame),
            ViewMode::Help => self.draw_help_view(frame),
            ViewMode::Question => {
                let question = &self.startup_questions[self.startup_question_idx];
                self.draw_question_view(frame, question.clone());
            }
        }
    }

    fn draw_menu_view(&self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ])
        .split(frame.area());

        let title_text = "                                                               ▄▄
███▀▀██▀▀███                      ▀███        ██               ██
█▀   ██   ▀█                        ██        ██
     ██    ▀███▄███ ▄█▀██▄  ▄██▀██  ██  ▄██▀██████▀███  ▀███ ▀███
     ██      ██▀ ▀▀██   ██ ██▀  ██  ██ ▄█     ██    ██    ██   ██
     ██      ██     ▄█████ ██       ██▄██     ██    ██    ██   ██
     ██      ██    ██   ██ ██▄    ▄ ██ ▀██▄   ██    ██    ██   ██
   ▄████▄  ▄████▄  ▀████▀██▄█████▀▄████▄ ██▄▄ ▀████ ▀████▀███▄████▄";

        let title_area = center(chunks[1], Constraint::Length(67), Constraint::Length(10));
        let title = Paragraph::new(title_text);

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

        let menu = Paragraph::new(Text::from(lines)).alignment(Alignment::Center);

        frame.render_widget(Clear, title_area);
        frame.render_widget(Clear, menu_area);
        frame.render_widget(title, title_area);
        frame.render_widget(menu, menu_area);
    }

    fn draw_help_view(&self, frame: &mut Frame) {
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

        frame.render_widget(Paragraph::new(Text::from(lines)), area);
    }

    fn draw_question_view(&mut self, frame: &mut Frame, question: String) {
        let area = center(frame.area(), Constraint::Length(20), Constraint::Length(10));

        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).split(area);

        // question
        let title = Paragraph::new(Text::from(question)).centered();
        frame.render_widget(title, chunks[0]);

        // answer

        let style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let answer = Paragraph::new(Text::from(vec![Line::from(vec![
            if self.ans_idx == 0 {
                "0".bold().style(style)
            } else {
                "0".into()
            },
            "  ".into(),
            if self.ans_idx == 1 {
                "1".bold().style(style)
            } else {
                "1".into()
            },
            "  ".into(),
            if self.ans_idx == 2 {
                "2".bold().style(style)
            } else {
                "2".into()
            },
            "  ".into(),
            if self.ans_idx == 3 {
                "3".bold().style(style)
            } else {
                "3".into()
            },
            "  ".into(),
            if self.ans_idx == 4 {
                "4".bold().style(style)
            } else {
                "4".into()
            },
        ])]))
        .centered();

        frame.render_widget(answer, chunks[1]);
    }

    fn draw_table_view(&mut self, frame: &mut Frame) {
        let area = center(
            frame.area(),
            Constraint::Length(35),
            Constraint::Percentage(70),
        );

        let chunks = Layout::vertical([Constraint::Min(5), Constraint::Length(4)]).split(area);

        self.draw_table(frame, chunks[0]);

        let bottom = if self.confirm {
            Paragraph::new(Text::from(vec![
                Line::from(vec!["Delete?".bold()]),
                Line::from(vec![
                    if self.confirm_idx == 0 {
                        "Yes".bold()
                    } else {
                        "Yes".into()
                    },
                    "  ".into(),
                    if self.confirm_idx == 1 {
                        "No".bold()
                    } else {
                        "No".into()
                    },
                ]),
            ]))
            .centered()
        } else {
            Paragraph::new("h: help").centered()
        };

        frame.render_widget(bottom, chunks[1]);
    }

    fn draw_table(&mut self, frame: &mut Frame, area: Rect) {
        let header = Row::new(vec!["Date", "Loss"])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1);

        let rows: Vec<Row> = self.data_series[self.selected_serie]
            .data
            .iter()
            .map(|&(x, y)| {
                let x_str = Local
                    .timestamp_opt(x, 0)
                    .single()
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| x.to_string());
                Row::new(vec![Cell::from(x_str), Cell::from(format!("{:.1}%", y))])
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .header(header)
        .block(
            Block::bordered()
                .title("  Table ⇅ ")
                .title_alignment(Alignment::Center)
                .padding(Padding::uniform(2)),
        )
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn draw_graph_view(&mut self, frame: &mut Frame) {
        let chunks =
            Layout::vertical([Constraint::Length(3), Constraint::Min(10)]).split(frame.area());

        self.draw_input_bar(frame, chunks[0]);
        self.draw_graph(frame, chunks[1]);
    }

    fn draw_input_bar(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::horizontal([
            Constraint::Length(12),
            Constraint::Min(20),
            Constraint::Length(12),
        ])
        .split(area);

        let input_style = match self.input_mode {
            crate::models::InputMode::Insert => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };

        let input_box = Paragraph::new(self.input.clone())
            .block(Block::bordered().title(" Input ").padding(Padding::left(1)))
            .alignment(Alignment::Center)
            .style(input_style);
        frame.render_widget(input_box, chunks[0]);

        let status = Paragraph::new(self.status_msg.clone()).block(
            Block::bordered()
                .title(" Status ")
                .padding(Padding::left(1)),
        );
        frame.render_widget(status, chunks[1]);

        let avg = Paragraph::new(format!(
            "{:.1}%",
            self.data_series[self.selected_serie].get_average_loss(20)
        ))
        .block(Block::bordered().title(" 20d avg "))
        .alignment(Alignment::Center);
        frame.render_widget(avg, chunks[2]);
    }

    fn draw_graph(&self, frame: &mut Frame, area: Rect) {
        let serie = &self.data_series[self.selected_serie];
        let data: Vec<(f64, f64)> = serie.data.iter().map(|&(x, y)| (x as f64, y)).collect();

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&data);

        let (x_min, x_max, y_min, y_max) = serie.get_bounds();
        let (x_labels, y_labels) = serie.get_labels();

        let title = if serie.name.is_empty() {
            String::new()
        } else {
            format!(" {} ", serie.name)
        };

        let chart = Chart::new(vec![dataset])
            .block(
                Block::bordered()
                    .title(Span::from(title))
                    .title_alignment(Alignment::Center),
            )
            .x_axis(Axis::default().bounds([x_min, x_max]).labels(x_labels))
            .y_axis(Axis::default().bounds([y_min, y_max]).labels(y_labels));

        frame.render_widget(chart, area);
    }
}
