use crossterm::event::KeyCode;

use crate::app::App;
use crate::models::{InputMode, ViewMode};

impl App {
    pub fn handle_key(&mut self, key: KeyCode) {
        match self.mode {
            ViewMode::Graph => self.handle_graph_input(key),
            ViewMode::Table => self.handle_table_input(key),
            ViewMode::Menu => self.handle_menu_input(key),
            ViewMode::Help => self.handle_help_input(key),
            ViewMode::Question => self.handle_question_input(key),
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

    fn handle_table_input(&mut self, key: KeyCode) {
        if self.confirm {
            match key {
                KeyCode::Esc => self.confirm = false,
                KeyCode::Left => self.confirm_idx = 0,
                KeyCode::Right => self.confirm_idx = 1,
                KeyCode::Tab => self.confirm_idx = 1 - self.confirm_idx,
                KeyCode::Enter => {
                    if self.confirm_idx == 0 {
                        if let Some(i) = self.table_state.selected() {
                            self.data_series[self.selected_serie].data.remove(i);
                        }
                    }
                    self.confirm = false;
                }
                _ => {}
            }
        } else {
            match key {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Char('g') => self.mode = ViewMode::Graph,
                KeyCode::Char('m') => self.mode = ViewMode::Menu,
                KeyCode::Char('h') => self.mode = ViewMode::Help,
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Tab => self.select_next(),
                KeyCode::Down | KeyCode::Char('j') => self.select_previous(),
                KeyCode::Char('d') if self.table_state.selected().is_some() => {
                    self.confirm = true;
                }
                KeyCode::Esc => self.mode = ViewMode::Menu,
                _ => {}
            }
        }
    }

    fn handle_graph_input(&mut self, key: KeyCode) {
        match self.input_mode {
            InputMode::Normal => match key {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Char('h') => self.mode = ViewMode::Help,
                KeyCode::Char('m') => self.mode = ViewMode::Menu,
                KeyCode::Char('t') => self.mode = ViewMode::Table,
                KeyCode::Char('i') => {
                    self.input_mode = InputMode::Insert;
                    self.input.clear();
                }
                KeyCode::Esc => self.mode = ViewMode::Menu,
                _ => {}
            },
            InputMode::Insert => match key {
                KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-' => {
                    if self.input.len() < 5 {
                        self.input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter if !self.input.is_empty() => self.try_insert_point(None),
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                    self.status_msg = "h: help".to_string();
                }
                _ => {}
            },
        }
    }

    fn handle_question_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Esc => {
                self.mode = ViewMode::Menu;
                self.ans_idx = 0;
                self.startup_question_idx = self.startup_questions.len();
            },
            KeyCode::Left => Self::cycle_idx(&mut self.ans_idx, -1, false, 4),
            KeyCode::Right => Self::cycle_idx(&mut self.ans_idx, 1, false, 4),
            KeyCode::BackTab => Self::cycle_idx(&mut self.ans_idx, -1, true, 4),
            KeyCode::Tab => Self::cycle_idx(&mut self.ans_idx, 1, true, 4),
            KeyCode::Enter => {
                self.startup_answers.push(self.ans_idx as u32);
                self.startup_question_idx += 1;
                self.ans_idx = 0;

                if self.startup_question_idx >= self.startup_questions.len() {
                    let val = self.calculate_score_from_questions();
                    self.try_insert_point(Some(val));
                    self.mode = ViewMode::Menu;
                }
            }
            _ => {}
        }
    }

    fn cycle_idx(idx: &mut usize, d: i32, wrap: bool, max: usize) {
        let new_idx = *idx as i32 + d;
        if wrap {
            *idx = (((new_idx % (max as i32 + 1) + (max as i32 + 1)) % (max as i32 + 1)) as usize)
                .max(0);
        } else {
            *idx = new_idx.clamp(0, max as i32) as usize;
        }
    }

    fn select_next(&mut self) {
        let len = self.data_series[self.selected_serie].data.len();
        if len == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
        self.table_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let len = self.data_series[self.selected_serie].data.len();
        if len == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| if i >= len - 1 { 0 } else { i + 1 });
        self.table_state.select(Some(i));
    }
}
