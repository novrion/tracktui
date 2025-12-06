mod app;
mod data_series;
mod input;
mod io;
mod models;
mod ui;

use color_eyre::Result;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut startup: bool = false;
    if args.len() > 1 {
        startup = match args[1].as_str() {
            "startup" => true,
            _ => false
        };
    }

    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = app::App::new(startup).run(&mut terminal);
    ratatui::restore();
    result
}
