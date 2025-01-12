use crate::events;
use crate::tree::TreeNode;
use crate::ui::ui;
use crossterm::event::{MouseButton, MouseEventKind};
use hdf5_metno as hdf5;
use ratatui::layout::Position;
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    DefaultTerminal,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

// pub type NodeIdT = String;
pub type NodeIdT = hdf5_metno_sys::h5i::hid_t;

enum Hdf5Object {
    Group(hdf5::Group),
    Dataset(hdf5::Dataset),
}

pub struct App {
    running: bool,
    pub h5_file_path: PathBuf,
    pub tree_state: tui_tree_widget::TreeState<NodeIdT>,
    pub tree: Option<TreeNode<NodeIdT>>,
    tree_node_to_object: HashMap<NodeIdT, Hdf5Object>,
    pub search_query_left: String,
    pub search_query_right: String,
    pub mode: Mode,
    rx: mpsc::Receiver<(TreeNode<NodeIdT>, HashMap<NodeIdT, Hdf5Object>)>,
    events: events::EventHandler,
}

pub enum Mode {
    Normal,
    SearchQueryEditing,
}

impl App {
    pub fn new(h5_file_path: PathBuf) -> Result<App, Box<dyn std::error::Error>> {
        let h5_file = hdf5::File::open(h5_file_path.clone())?;

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let (tree, tree_node_to_object) =
                App::tree_from_h5(&h5_file).expect("Failed to parse HDF5 structure");
            tx.send((tree, tree_node_to_object)).unwrap();
        });

        let events = events::EventHandler::new();

        Ok(App {
            running: true,
            h5_file_path,
            tree_state: tui_tree_widget::TreeState::default(),
            tree: None,
            tree_node_to_object: HashMap::new(),
            search_query_left: String::new(),
            search_query_right: String::new(),
            mode: Mode::Normal,
            rx,
            events,
        })
    }

    fn relative_child_name<'a>(parent: &str, child: &'a str) -> &'a str {
        let x = child.strip_prefix(parent).unwrap();
        if x == "/" {
            return "/";
        }
        x.strip_prefix("/").unwrap_or(x)
    }

    fn tree_from_h5(
        h5_file: &hdf5::File,
    ) -> Result<(TreeNode<NodeIdT>, HashMap<NodeIdT, Hdf5Object>), std::io::Error> {
        fn tree_from_group(
            parent_name: &str,
            group: hdf5::Group,
            tree_node_to_object: &mut HashMap<NodeIdT, Hdf5Object>,
        ) -> TreeNode<NodeIdT> {
            // TODO: avoid circular walks
            // The identifier for each TreeNode is the unmodified hdf5 group/dataset name.
            // The name is the full path inside the hdf5 file.
            // This allows us to retrieve the object later

            let mut children: Vec<_> = group
                .groups()
                .unwrap_or(vec![])
                .into_iter()
                .map(|child| tree_from_group(&group.name(), child, tree_node_to_object))
                .collect();

            let datasets = group.datasets().unwrap_or(vec![]);

            for dataset in datasets {
                let dataset_name = dataset.name();
                let text = App::relative_child_name(&group.name(), &dataset_name);
                let node_id = dataset.id();
                tree_node_to_object.insert(node_id, Hdf5Object::Dataset(dataset));
                children.push(TreeNode::new(node_id, text, vec![]));
            }

            let group_id = group.id();
            let text = App::relative_child_name(&parent_name, &group.name()).to_string();
            tree_node_to_object.insert(group_id, Hdf5Object::Group(group));
            TreeNode::new(group_id, text, children)
        }
        // TODO anonymous datasets

        let mut tree_node_to_object = HashMap::new();
        let root_group = h5_file.group("/").expect("Couldn't open root group");
        let tree = tree_from_group("", root_group, &mut tree_node_to_object);
        Ok((tree, tree_node_to_object))
    }

    fn get_text_for_dataset(dataset: &hdf5::Dataset) -> String {
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
        let compression_ratio = if storage_size > 0 {
            data_size as f64 / storage_size as f64
        } else {
            f64::NAN
        };

        format!(
            "Dataset info:Name: {}\n\nShape: {:?}\nDatatype: {:?}\nSpace: {:?}\nStorage: {}\nCompression: {}\nStorage size: {} bytes\nData size: {} bytes\nCompression ratio: {:.2}",
            dataset.name(), shape, datatype, space, chunk_info, compression_info, storage_size, data_size, compression_ratio
        )
    }

    fn get_text_for_group(group: &hdf5::Group) -> String {
        let num_groups = group.groups().unwrap_or(vec![]).len();
        let num_datasets = group.datasets().unwrap_or(vec![]).len();
        let attrs = group.attr_names().unwrap_or(vec![]);
        let num_attrs = attrs.len();

        format!(
            "Group info:\nName: {}\nNumber of groups: {}\nNumber of datasets: {}\nNumber of attributes: {}\nAttribute names: {:?}",
            group.name(),
            num_groups,
            num_datasets,
            num_attrs,
            attrs
        )
    }

    pub fn get_text_for(&self, id: NodeIdT) -> String {
        match self.tree_node_to_object.get(&id) {
            Some(object) => match object {
                Hdf5Object::Dataset(dataset) => App::get_text_for_dataset(dataset),
                Hdf5Object::Group(group) => App::get_text_for_group(group),
            },
            None => format!("Couldn't find object with id {}", id),
        }
    }

    fn on_click(&mut self, column: u16, row: u16) {
        let position = Position::new(column, row);

        dbg!(&position);

        if let Some(id) = self.tree_state.rendered_at(position) {
            let arg = id.to_vec();
            self.tree_state.toggle(arg.clone());
            self.tree_state.select(arg);
        }
    }

    fn on_up(&mut self) {
        self.tree_state.key_up();
    }

    fn on_down(&mut self) {
        self.tree_state.key_down();
    }

    fn on_keypress_normal_mode(&mut self, keycode: KeyCode) -> () {
        match keycode {
            KeyCode::Left => {
                self.tree_state.key_left();
            }
            KeyCode::Char('h') => {
                self.tree_state.key_left();
            }
            KeyCode::Up => {
                self.on_up();
            }
            KeyCode::Char('k') => {
                self.on_up();
            }
            KeyCode::Down => {
                self.on_down();
            }
            KeyCode::Char('j') => {
                self.on_down();
            }
            KeyCode::Right => {
                self.tree_state.key_right();
            }
            KeyCode::Char('l') => {
                self.tree_state.key_right();
            }
            KeyCode::Home => {
                self.tree_state.select_first();
            }
            KeyCode::End => {
                self.tree_state.select_last();
            }
            KeyCode::Enter => {
                self.tree_state.toggle_selected();
            }
            KeyCode::Tab => {
                self.on_tab();
            }
            KeyCode::BackTab => {
                self.on_shift_tab();
            }
            KeyCode::Char('f') => {
                if let Some(tree) = &self.tree {
                    let mut to_visit = vec![(tree, vec![tree.id()])];
                    while let Some((current, id_path)) = to_visit.pop() {
                        self.tree_state.open(id_path.clone());
                        to_visit.extend(current.children().iter().map(|c| {
                            let mut id_path = id_path.clone();
                            id_path.push(c.id());
                            (c, id_path)
                        }));
                    }
                }
            }
            _ => {}
        }
    }

    // TODO does reversing work fine for all utf8 things?
    pub fn search_query_and_cursor(&self) -> (String, usize) {
        let rev_right: String = self.search_query_right.chars().rev().collect();
        (
            self.search_query_left.clone() + &rev_right,
            self.search_query_left.len(),
        )
    }

    fn on_tab(&mut self) -> () {
        self.tree_state
            .select_relative(|x| x.map_or(0, |current| current.saturating_add(1)));
    }

    fn on_shift_tab(&mut self) -> () {
        self.tree_state
            .select_relative(|x| x.map_or(0, |current| current.saturating_sub(1)));
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
            KeyCode::Tab => {
                self.on_tab();
            }
            KeyCode::BackTab => {
                self.on_shift_tab();
            }
            KeyCode::Up => {
                self.on_up();
            }
            KeyCode::Down => {
                self.on_down();
            }
            _ => {}
        };
    }

    pub async fn run(
        mut self,
        mut terminal: DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok((tree, tree_node_to_object)) = self.rx.recv() {
            self.tree = Some(tree);
            self.tree_node_to_object = tree_node_to_object;
            self.tree_state.open(vec![self.tree.as_ref().unwrap().id()]);
        }

        while self.running {
            terminal.draw(|frame| ui(frame, &mut self))?;

            match self.events.next().await? {
                events::Event::Tick => {}
                events::Event::Key(key) => self.handle_keypress(key),
                events::Event::Mouse(mouse) => self.handle_mouse(mouse),
                events::Event::Resize(_, _) => {}
            }
        }
        Ok(())
    }

    fn handle_keypress(&mut self, key: crossterm::event::KeyEvent) {
        if key.kind == crossterm::event::KeyEventKind::Press {
            // if Ctrl+c is pressed, exit
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                self.running = false;
            }

            match self.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        self.running = false;
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

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.on_click(mouse.column, mouse.row),
            _ => {}
        }
    }
}
