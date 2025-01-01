use ratatui::layout::Position;
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

    pub fn on_click(&mut self, column: u16, row: u16) {
        let position = Position::new(column, row);

        if let Some(id) = self.tree_state.rendered_at(position) {
            self.tree_state.toggle(id.to_vec());
        }
    }
}
