use crate::ui::ui;
use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::layout::Position;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    DefaultTerminal,
};
use std::io;
use std::path::PathBuf;
use tui_tree_widget::{Tree, TreeItem, TreeState};

pub struct App<'a> {
    pub h5_file_path: PathBuf,
    pub tree_state: TreeState<String>,
    pub tree_items: Vec<TreeItem<'a, String>>,
}

impl<'a> App<'a> {
    pub fn new(h5_file_path: PathBuf) -> App<'a> {
        // let items = vec![
        //     TreeItem::new_leaf("leaf_1_id".to_string(), "leaf_1".to_string()),
        //     TreeItem::new_leaf("leaf_2_id".to_string(), "leaf_2".to_string()),
        //     TreeItem::new(
        //         "asd_id".to_string(),
        //         "asd".to_string(),
        //         [
        //             TreeItem::new_leaf("leaf_3_id".to_string(), "leaf_3".to_string()),
        //             TreeItem::new_leaf("leaf_4_id".to_string(), "leaf_4".to_string()),
        //         ]
        //         .to_vec(),
        //     )
        //     .unwrap(),
        //     TreeItem::new(
        //         "no_child_id".to_string(),
        //         "no children".to_string(),
        //         Vec::new(),
        //     )
        //     .unwrap(),
        // ];

        let mut app = App {
            h5_file_path,
            tree_state: TreeState::default(),
            tree_items: vec![],
        };
        app.tree_items = app.tree_from_h5().unwrap();
        app
    }

    fn tree_from_group(group: hdf5::Group) -> Result<Vec<TreeItem<'a, String>>, std::io::Error> {
        let mut result = Vec::new();

        // println!("Found group={group:?}");
        for child in group.groups().unwrap_or(vec![]) {
            result.push(TreeItem::new(
                child.name().clone(),
                child.name().clone(),
                App::tree_from_group(child)?,
            )?);
        }
        for dataset in group.datasets().unwrap_or(vec![]) {
            // println!("Found dataset={}", child.name());
            result.push(TreeItem::new_leaf(
                dataset.name().clone(),
                dataset.name().clone(),
            ));
        }
        Ok(result)
    }

    fn tree_from_h5(&self) -> Result<Vec<TreeItem<'a, String>>, std::io::Error> {
        let file = hdf5::File::open(self.h5_file_path.clone())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, "asd"))?;
        // dbg!(&file);
        //

        // Ok(())
        App::tree_from_group(file.group("/").expect("Couldn't open root group"))

        // let dataset = file.dataset("/group1/something").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, "asd"))?;

        // Ok(TreeItem::new(dataset.name().clone(),dataset.name().clone(), vec![])?)
        // Ok(result)
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
