use chrono::{Local, TimeZone};
use ratatui::{
    style::{Modifier, Style},
    text::Span,
};

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
struct App {
    mode: ViewMode,
    data_series: Vec<DataSeries>,
    selected_serie: usize,

    // Graph View
    input_mode: InputMode,
    input: String,
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
    data: Vec<(i64, f64)>,
}
