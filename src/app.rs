use crate::tree::TreeNode;
use crate::ui::ui;
use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::layout::Position;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    DefaultTerminal,
};
use std::io;
use std::path::PathBuf;

pub struct App {
    pub h5_file_path: PathBuf,
    h5_file: hdf5::File,
    pub tree_state: tui_tree_widget::TreeState<String>,
    pub tree: TreeNode,
    pub search_query_left: String,
    pub search_query_right: String,
    pub mode: Mode,
    pub filter_tree_using_search: bool,
}

pub enum Mode {
    Normal,
    SearchQueryEditing,
}

impl App {
    pub fn new(h5_file_path: PathBuf) -> App {
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

        let h5_file = hdf5::File::open(&h5_file_path).expect("Couldn't open h5 file");
        let mut app = App {
            h5_file,
            h5_file_path,
            tree_state: tui_tree_widget::TreeState::default(),
            tree: TreeNode::new("emptytree", "empty tree", vec![])
                .expect("Failed to create root node"),
            search_query_left: String::new(),
            search_query_right: String::new(),
            mode: Mode::Normal,
            filter_tree_using_search: false,
        };
        app.tree = app.tree_from_h5().expect("Failed to parse HDF5 structure");
        app
    }

    fn relative_child_name<'b>(parent: &str, child: &'b str) -> &'b str {
        let x = child.strip_prefix(parent).unwrap();
        x.strip_prefix("/").unwrap_or(x)
    }

    fn tree_from_group(parent_name: &str, group: hdf5::Group) -> Result<TreeNode, std::io::Error> {
        // TODO: avoid circular walks
        let mut children = Vec::new();

        let text = App::relative_child_name(&parent_name, &group.name()).to_string();

        // The identifier for each TreeNode is the unmodified hdf5 group/dataset name.
        // The name is the full path inside the hdf5 file.
        // This allows us to retrieve the object later

        for child in group.groups().unwrap_or(vec![]) {
            children.push(App::tree_from_group(&group.name(), child)?);
        }
        for dataset in group.datasets().unwrap_or(vec![]) {
            children.push(TreeNode::new(
                dataset.name(),
                App::relative_child_name(&group.name(), &dataset.name()).to_string(),
                vec![],
            )?);
        }
        TreeNode::new(group.name(), text, children)
    }

    fn tree_from_h5(&self) -> Result<TreeNode, std::io::Error> {
        // TODO anonymous datasets
        App::tree_from_group(
            "/",
            self.h5_file.group("/").expect("Couldn't open root group"),
        )
    }

    pub fn get_text_for(&self, path_to_object: &str) -> String {
        // self.h5_file.group(name)
        match self.h5_file.dataset(path_to_object) {
            Ok(dataset) => {
                let shape = dataset.shape();
                let datatype = dataset.dtype();
                let space = dataset.space();
                let chunks = dataset.chunk();
                let chunk_info = match chunks {
                    Some(chunks) => format!("Chunked ({:?})", chunks),
                    None => "Contiguous".to_string(),
                };

                // Get compression info
                let compression = dataset.filters();
                let compression_info = format!("Filter pipeline: {:?}", compression);

                // Get storage size vs data size
                let storage_size = dataset.storage_size();
                let data_size = dataset.size() * dataset.dtype().map_or(0, |dt| dt.size());
                let compression_ratio = if data_size > 0 {
                    storage_size as f64 / data_size as f64
                } else {
                    f64::NAN
                };

                format!(
                    "Dataset info:\nShape: {:?}\nDatatype: {:?}\nSpace: {:?}\nStorage: {}\nCompression: {}\nStorage size: {} bytes\nData size: {} bytes\nCompression ratio: {:.2}",
                    shape, datatype, space, chunk_info, compression_info, storage_size, data_size, compression_ratio
                )
            }
            Err(_) => match self.h5_file.group(path_to_object) {
                Ok(group) => {
                    let num_groups = group.groups().unwrap_or(vec![]).len();
                    let num_datasets = group.datasets().unwrap_or(vec![]).len();
                    let attrs = group.attr_names().unwrap_or(vec![]);
                    let num_attrs = attrs.len();

                    format!(
                        "Group info:\nNumber of groups: {}\nNumber of datasets: {}\nNumber of attributes: {}\nAttribute names: {:?}",
                        num_groups,
                        num_datasets,
                        num_attrs,
                        attrs
                    )
                }
                Err(_) => "what is this?".to_string(),
            },
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

    fn on_keypress_normal_mode(&mut self, keycode: KeyCode) {
        let _ = match keycode {
            KeyCode::Left => self.tree_state.key_left(),
            KeyCode::Char('h') => self.tree_state.key_left(),
            KeyCode::Up => self.tree_state.key_up(),
            KeyCode::Char('k') => self.tree_state.key_up(),
            KeyCode::Down => self.tree_state.key_down(),
            KeyCode::Char('j') => self.tree_state.key_down(),
            KeyCode::Right => self.tree_state.key_right(),
            KeyCode::Char('l') => self.tree_state.key_right(),
            KeyCode::Home => self.tree_state.select_first(),
            KeyCode::End => self.tree_state.select_last(),
            KeyCode::Enter => self.tree_state.toggle_selected(),
            KeyCode::Char('s') => {
                self.filter_tree_using_search = !self.filter_tree_using_search;
                true
            }
            // KeyCode::Char('f') => {
            //     // self.tree_items.iter().for_each(|item| {
            //     //     // dbg!(item);
            //     //     self.tree_state.open(vec![item.identifier().to_string()]);
            //     // });
            //     // self.tree_state.open()
            //     true
            // },
            _ => false,
        };
        return;
    }

    // TODO does reversing work fine for all utf8 things?
    pub fn search_query_and_cursor(&self) -> (String, usize) {
        let rev_right: String = self.search_query_right.chars().rev().collect();
        (
            self.search_query_left.clone() + &rev_right,
            self.search_query_left.len(),
        )
    }

    fn on_keypress_search_mode(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::Char(to_insert) => {
                self.search_query_left.push(to_insert);
            }
            KeyCode::Left => {
                self.search_query_left
                    .pop()
                    .map(|c| self.search_query_right.push(c));
            }
            KeyCode::Right => {
                self.search_query_right
                    .pop()
                    .map(|c| self.search_query_left.push(c));
            }
            KeyCode::Home => self
                .search_query_right
                .extend(self.search_query_left.drain(..).rev()),
            KeyCode::End => self
                .search_query_left
                .extend(self.search_query_right.drain(..).rev()),
            KeyCode::Backspace => {
                self.search_query_left.pop();
            }
            KeyCode::Delete => {
                self.search_query_right.pop();
            }
            _ => {}
        };
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
                    match self.mode {
                        Mode::Normal => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                return Ok(true);
                            }
                            KeyCode::Char('/') => {
                                self.mode = Mode::SearchQueryEditing;
                            }
                            other => {
                                self.on_keypress_normal_mode(other);
                            }
                        },
                        Mode::SearchQueryEditing => match key.code {
                            KeyCode::Esc | KeyCode::Enter => {
                                self.mode = Mode::Normal;
                            }
                            other => {
                                self.on_keypress_search_mode(other);
                            }
                        },
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
