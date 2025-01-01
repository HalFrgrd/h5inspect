use crossterm::event::KeyCode;
use ratatui::layout::Position;
use tui_tree_widget::{Tree, TreeItem, TreeState};

pub struct App {
    pub file_name: String,
    pub tree_state: TreeState<String>,
    pub tree_items: Vec<TreeItem<'static, String>>,
}

impl App {
    pub fn new() -> App {
        let items = vec![
            TreeItem::new_leaf("leaf_1_id".to_string(), "leaf_1".to_string()),
            TreeItem::new_leaf("leaf_2_id".to_string(), "leaf_2".to_string()),
            TreeItem::new(
                "asd_id".to_string(),
                "asd".to_string(),
                [
                    TreeItem::new_leaf("leaf_3_id".to_string(), "leaf_3".to_string()),
                    TreeItem::new_leaf("leaf_4_id".to_string(), "leaf_4".to_string()),
                ]
                .to_vec(),
            )
            .unwrap(),
        ];

        App {
            file_name: "test file name.h5".to_string(),
            tree_state: TreeState::default(),
            tree_items: items,
        }
    }

    pub fn on_click(&mut self, column: u16, row: u16) {
        let position = Position::new(column, row);

        if let Some(id) = self.tree_state.rendered_at(position) {
            let arg = id.to_vec();
            self.tree_state.toggle(arg.clone());
            self.tree_state.select(arg);
        }
    }

    pub fn on_keypress(&mut self, keycode: KeyCode) {
        let _ = match keycode {
            KeyCode::Up => self.tree_state.key_up(),
            KeyCode::Left => self.tree_state.key_up(),
            KeyCode::Down => self.tree_state.key_down(),
            KeyCode::Right => self.tree_state.key_down(),
            KeyCode::Home => self.tree_state.select_first(),
            KeyCode::End => self.tree_state.select_last(),
            KeyCode::Enter => self.tree_state.toggle_selected(),
            _ => false,
        };
        return;
    }
}
