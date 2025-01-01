use crate::ui::ui;
use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::layout::Position;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    DefaultTerminal,
};
use std::io;
use std::path::PathBuf;
use tui_tree_widget::{TreeItem, TreeState};

pub struct App<'a> {
    pub h5_file_path: PathBuf,
    h5_file: hdf5::File,
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
            h5_file: hdf5::File::open(h5_file_path.clone()).unwrap(),
            h5_file_path,
            // h5_file: hdf5::File::open(self.h5_file_path.clone()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            tree_state: TreeState::default(),
            tree_items: vec![],
        };
        app.tree_items = app.tree_from_h5().expect("Problem parsing hdf5 structure");
        app
    }

    fn relative_child_name<'b>(parent: &str, child: &'b str) -> &'b str {
        let x = child.strip_prefix(parent).unwrap();
        x.strip_prefix("/").unwrap_or(x)
    }

    fn tree_from_group(group: hdf5::Group) -> Result<Vec<TreeItem<'a, String>>, std::io::Error> {
        // TODO: avoid circular walks
        let mut result = Vec::new();

        // The identifier for each TreeItem is the unmodified hdf5 group/dataset name.
        // The name is the full path inside the hdf5 file.
        // This allows us to retrieve the object later

        for child in group.groups().unwrap_or(vec![]) {
            result.push(TreeItem::new(
                child.name(),
                App::relative_child_name(&group.name(), &child.name()).to_string(),
                App::tree_from_group(child)?,
            )?);
        }
        for dataset in group.datasets().unwrap_or(vec![]) {
            result.push(TreeItem::new_leaf(
                dataset.name(),
                App::relative_child_name(&group.name(), &dataset.name()).to_string(),
            ));
        }
        Ok(result)
    }

    fn tree_from_h5(&self) -> Result<Vec<TreeItem<'a, String>>, std::io::Error> {
        // TODO anonymous datasets
        App::tree_from_group(self.h5_file.group("/").expect("Couldn't open root group"))
    }

    pub fn get_text_for(&self, path_to_object: &str) -> &str {
        // self.h5_file.group(name)
        match self.h5_file.dataset(path_to_object) {
            Ok(_) => "asd",
            Err(_) => "not a dataset",
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
