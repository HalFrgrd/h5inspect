use tui_tree_widget::TreeState;

pub struct App {
    pub file_name: String,
    pub tree_state: TreeState<String>,
}

impl App {
    pub fn new() -> App {
        App {
            file_name: "test file name.h5".to_string(),
            tree_state: TreeState::default(),
        }
    }
}
