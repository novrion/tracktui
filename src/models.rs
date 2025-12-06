#[derive(Default, PartialEq)]#[allow(dead_code)]
pub enum ViewMode {
    #[default]
    Graph,
    Table,
    Menu,
    Help,
    Question,
}

#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}
