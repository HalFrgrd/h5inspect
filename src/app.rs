use crate::ui::ui;
use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::layout::Position;
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode},
    DefaultTerminal,
};
use std::io;
use tui_tree_widget::{TreeItem, TreeState};

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
            TreeItem::new(
                "no_child_id".to_string(),
                "no children".to_string(),
                Vec::new(),
            )
            .unwrap(),
        ];

        App {
            file_name: "test file name.h5".to_string(),
            tree_state: TreeState::default(),
            tree_items: items,
        }
    }

    fn on_click(&mut self, column: u16, row: u16) {
        let position = Position::new(column, row);

        if let Some(id) = self.tree_state.rendered_at(position) {
            let arg = id.to_vec();
            self.tree_state.toggle(arg.clone());
            self.tree_state.select(arg);
        }
    }

    fn on_keypress(&mut self, keycode: KeyCode) {
        let _ = match keycode {
            KeyCode::Up => self.tree_state.key_up(),
            KeyCode::Left => self.tree_state.key_up(),
            KeyCode::Char('k') => self.tree_state.key_up(),
            KeyCode::Down => self.tree_state.key_down(),
            KeyCode::Right => self.tree_state.key_down(),
            KeyCode::Char('j') => self.tree_state.key_down(),
            KeyCode::Home => self.tree_state.select_first(),
            KeyCode::End => self.tree_state.select_last(),
            KeyCode::Enter => self.tree_state.toggle_selected(),
            _ => false,
        };
        return;
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<bool> {
        loop {
            terminal.draw(|f| ui(f, &mut self))?;

            if let Ok(true) = self.handle_events() {
                return Ok(true);
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<bool> {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(true);
                        }
                        other => {
                            self.on_keypress(other);
                        }
                    }
                }
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => self.on_click(mouse.column, mouse.row),
                _ => {}
            },
            _ => {}
        }
        return Ok(false);
    }
}
