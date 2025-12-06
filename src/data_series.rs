use chrono::{Local, TimeZone};
use ratatui::{
    style::{Modifier, Style},
    text::Span,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct DataSeries {
    pub name: String,
    pub data: Vec<(i64, f64)>,
}

impl DataSeries {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_bounds(&self) -> (f64, f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 1.0, 0.0, 1.0);
        }

        let x_min = self.data[0].0 as f64;
        let x_max = self.data.iter().map(|&(x, _)| x).max().unwrap_or(0) as f64;

        (x_min, x_max, 0.0, 100.0)
    }

    pub fn get_labels(&self) -> (Vec<Span<'_>>, Vec<Span<'_>>) {
        if self.data.is_empty() {
            return (vec![], vec![]);
        }

        let mut x_labels = Vec::new();
        let idxs: Vec<usize> = if self.data.len() > 2 {
            vec![0, self.data.len() - 1]
        } else {
            vec![0]
        };

        for i in idxs {
            if let Some(dt) = Local.timestamp_opt(self.data[i].0, 0).single() {
                x_labels.push(Span::styled(
                    dt.format("%Y-%m-%d").to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            }
        }

        let y_labels: Vec<Span> = (0..11)
            .map(|i| {
                Span::styled(
                    format!("{}%", i * 10),
                    Style::default().add_modifier(Modifier::BOLD),
                )
            })
            .collect();

        (x_labels, y_labels)
    }

    pub fn get_average_loss(&self, max_n: usize) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }

        let count = self.data.len().min(max_n);
        let sum: f64 = self.data.iter().rev().take(count).map(|&(_, y)| y).sum();
        sum / count as f64
    }
}
