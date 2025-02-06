use crate::events;
use crate::h5_utils;
use crate::tree::TreeNode;
use crate::ui::ui;
use crossterm::event::{MouseButton, MouseEventKind};
use hdf5_metno as hdf5;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;
use std::path::PathBuf;

#[allow(unused_imports)]
use log::*;

#[derive(Debug, Clone)]
pub enum Hdf5Object {
    Group(hdf5::Group),
    Dataset(hdf5::Dataset),
}

impl PartialEq for Hdf5Object {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

pub type NodeIdT = hdf5_metno_sys::h5i::hid_t;

pub struct App {
    running: bool,
    pub h5_file_path: PathBuf,
    pub tree_state: tui_tree_widget::TreeState<NodeIdT>,
    pub tree: Option<TreeNode<NodeIdT>>,
    pub filtered_tree: Option<TreeNode<NodeIdT>>,
    pub search_query_left: String,
    pub search_query_right: String,
    pub mode: Mode,
    pub show_logs: bool,
}

pub enum Mode {
    Normal,
    SearchQueryEditing,
}

impl App {
    pub fn new(h5_file_path: PathBuf) -> App {
        App {
            running: true,
            h5_file_path,
            tree_state: tui_tree_widget::TreeState::default(),
            tree: None,
            filtered_tree: None,
            search_query_left: String::new(),
            search_query_right: String::new(),
            mode: Mode::Normal,
            show_logs: cfg!(debug_assertions),
        }
    }

    fn tree_from_h5(h5_file: &hdf5::File) -> Result<TreeNode<NodeIdT>, std::io::Error> {
        fn tree_from_group(group_name: &str, group: hdf5::Group) -> TreeNode<NodeIdT> {
            // TODO: avoid circular walks
            // The identifier for each TreeNode is the unmodified hdf5 group/dataset name.
            // The name is the full path inside the hdf5 file.
            // This allows us to retrieve the object later

            let mut children: Vec<_> = h5_utils::groups(&group)
                .unwrap_or(vec![])
                .into_iter()
                .map(|(name, child)| tree_from_group(&name, child))
                .collect();

            let datasets = h5_utils::datasets(&group).unwrap_or(vec![]);

            for (dataset_name, dataset) in datasets.into_iter() {
                let text = dataset_name.clone();
                let node_id = dataset.id();
                children.push(
                    TreeNode::new(node_id, text, vec![])
                        .set_dataset_size(dataset.size())
                        .set_hdf5_object(Hdf5Object::Dataset(dataset)),
                );
            }

            TreeNode::new(group.id(), group_name, children)
                .set_hdf5_object(Hdf5Object::Group(group))
        }
        // TODO anonymous datasets

        let root_name = "/";
        let root_group = h5_file.group(root_name).expect("Couldn't open root group");
        Ok(tree_from_group(root_name, root_group))
    }

    pub fn get_text_for(&self, path: &[NodeIdT]) -> Option<Vec<(String, String)>> {
        if let Some(tree) = &self.tree {
            match tree.get_selected_node(path) {
                Some(ref tree_node) => match &tree_node.hdf5_object {
                    Some(Hdf5Object::Dataset(dataset)) => {
                        Some(h5_utils::get_text_for_dataset(&dataset))
                    }
                    Some(Hdf5Object::Group(group)) => {
                        Some(h5_utils::get_text_for_group(&group))
                        // let size = tree_node.recursive_data_size;
                        // format!("{} {}", text, size)
                    }
                    None => {
                        debug!("No hdf5 object found at path {:?}", path);
                        None
                    }
                },
                None => {
                    debug!("Couldn't find object at path {:?}", path);
                    None
                }
            }
        } else {
            debug!("No tree found");
            None
        }
    }

    fn on_click(&mut self, column: u16, row: u16) {
        let position = Position::new(column, row);

        log::debug!("clicked at {:?}", position);

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
            KeyCode::Char('c') => {
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
            KeyCode::Char('g') => {
                // it's a lot easier to go the first one this way than to use "gg" like in vim
                self.tree_state.select_first();
            }
            KeyCode::Char('G') => {
                self.tree_state.select_last();
            }
            KeyCode::Char('?') => {
                self.show_logs = !self.show_logs;
            }
            KeyCode::PageDown => {
                self.tree_state.select_relative(|current| {
                    current.map_or(0, |current| current.saturating_add(50))
                });
            }
            KeyCode::PageUp => {
                self.tree_state.select_relative(|current| {
                    current.map_or(0, |current| current.saturating_sub(50))
                });
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
        let mut refresh_filtered_tree = true;
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
            keycode => {
                refresh_filtered_tree = false;
                self.on_keypress_normal_mode(keycode);
            }
        };
        if refresh_filtered_tree {
            self.update_filtered_tree();
        }
    }

    fn update_filtered_tree(&mut self) {
        let query = &self.search_query_and_cursor().0;
        match &self.tree {
            Some(tree) => {
                self.filtered_tree = tree.filter(query);
                self.update_selected_tree_item();
            }
            None => {
                self.filtered_tree = None;
            }
        }
    }

    fn update_selected_tree_item(&mut self) {
        if let Some(filtered_tree) = &self.filtered_tree {
            let nothing_selected = self.tree_state.selected().is_empty();
            let selected_item = filtered_tree.get_selected_node(&self.tree_state.selected());
            let selected_item_is_in_tree = selected_item.is_some();
            let selected_item_is_direct_match = selected_item.map_or(false, |t| t.is_direct_match);

            if nothing_selected || !selected_item_is_in_tree || !selected_item_is_direct_match {
                let first_match = filtered_tree.path_to_first_match();
                self.tree_state.select(first_match.clone());
                for i in 0..first_match.len() {
                    self.tree_state.open(first_match[0..i].to_vec());
                }
            }
            self.tree_state.scroll_selected_into_view();
        }
    }

    pub async fn run<B: ratatui::backend::Backend>(
        mut self,
        mut terminal: ratatui::Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let h5_file = h5_utils::open_file(&self.h5_file_path)?;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<TreeNode<NodeIdT>>(1);
        tokio::spawn(async move {
            let tree = App::tree_from_h5(&h5_file).expect("Failed to parse HDF5 structure");
            tx.send(tree).await.unwrap();
        });

        let mut events = events::EventHandler::new();

        while self.running {
            terminal.draw(|frame| ui(frame, &mut self))?;

            tokio::select! {
                Some(event) = events.receiver.recv() => {
                    match event {
                        events::Event::Tick => {}
                        events::Event::Key(key) => self.handle_keypress(key),
                        events::Event::Mouse(mouse) => self.handle_mouse(mouse),
                        events::Event::Resize => {}
                    }
                }
                Some(tree) = rx.recv() => {
                    self.tree = Some(tree);
                    self.tree_state.open(vec![self.tree.as_ref().unwrap().id()]);
                    self.update_filtered_tree();
                }
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
            MouseEventKind::ScrollDown => {
                self.tree_state.scroll_down(1);
            }
            MouseEventKind::ScrollUp => {
                self.tree_state.scroll_up(1);
            }
            _ => {}
        }
    }
}
