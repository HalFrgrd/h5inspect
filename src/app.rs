use crate::analysis;
use crate::analysis::AnalysisResult;
use crate::events;
use crate::h5_utils;
use crate::tree::TreeNode;
use crate::ui::ui;
use crossterm::event::{MouseButton, MouseEventKind};
use hdf5_metno as hdf5;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::vec;

use tokio;

#[allow(unused_imports)]
use log::*;

#[derive(Debug, Clone)]
pub enum Hdf5Object {
    Group(hdf5::Group),
    Dataset(Arc<hdf5::Dataset>),
}

impl PartialEq for Hdf5Object {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[derive(Debug)]
enum AsyncDataAnalysis {
    Loading,
    Ready(analysis::AnalysisResult),
}

pub type NodeIdT = hdf5_metno_sys::h5i::hid_t;

pub struct App {
    running: bool,
    pub h5_file_path: PathBuf,
    pub tree_state: tui_tree_widget::TreeState<NodeIdT>,
    pub tree_state_last_rendered_selected: Option<Vec<NodeIdT>>,
    pub tree: Option<TreeNode<NodeIdT>>,
    pub filtered_tree: Option<TreeNode<NodeIdT>>,
    pub search_query_left: String,
    pub search_query_right: String,
    pub mode: SelectionMode,
    pub show_logs: bool,
    pub object_info_scroll_state: u16,
    pub last_object_info_area: Rect,
    pub last_tree_area: Rect,
    pub last_search_query_area: Rect,
    pub animation_state: u8,
    node_id_to_analysis: Arc<Mutex<HashMap<NodeIdT, AsyncDataAnalysis>>>,
}

fn last_chars(s: &str, n: usize) -> &str {
    let len = s.chars().count();
    let start = len.saturating_sub(n);
    let idx = s.char_indices().nth(start).map(|(i, _)| i).unwrap_or(0);
    &s[idx..]
}

fn first_chars(s: &str, n: usize) -> &str {
    let mut char_indices = s.char_indices();
    let idx = char_indices.nth(n).map(|(i, _)| i).unwrap_or(s.len());
    &s[..idx]
}

pub enum SelectionMode {
    TreeBrowsing,
    SearchQueryEditing,
    ObjectInfoInspecting,
}

impl App {
    pub fn new(h5_file_path: PathBuf) -> App {
        App {
            running: true,
            h5_file_path,
            tree_state: tui_tree_widget::TreeState::default(),
            tree_state_last_rendered_selected: None,
            tree: None,
            filtered_tree: None,
            search_query_left: String::new(),
            search_query_right: String::new(),
            mode: SelectionMode::TreeBrowsing,
            show_logs: cfg!(debug_assertions),
            object_info_scroll_state: 0,
            last_object_info_area: Rect::new(0, 0, 0, 0),
            last_tree_area: Rect::new(0, 0, 0, 0),
            last_search_query_area: Rect::new(0, 0, 0, 0),
            animation_state: 0,
            node_id_to_analysis: Arc::new(Mutex::new(HashMap::new())),
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
                        .set_storage_dataset_size(dataset.storage_size())
                        .set_hdf5_object(Hdf5Object::Dataset(Arc::new(dataset))),
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

    pub fn get_num_active_data_analysis_tasks(&self) -> usize {
        let info_dict = self.node_id_to_analysis.lock().unwrap();
        info_dict
            .values()
            .filter(|&v| matches!(v, AsyncDataAnalysis::Loading))
            .count()
    }

    pub fn get_text_for(&mut self, path: &[NodeIdT]) -> Option<Vec<(String, String)>> {
        if let Some(tree) = &self.tree {
            match tree.get_selected_node(path) {
                Some(ref tree_node) => match &tree_node.hdf5_object {
                    Some(Hdf5Object::Dataset(dataset)) => {
                        let mut info = h5_utils::get_text_for_dataset(&dataset);

                        let k = tree_node.id().clone();

                        let stats_text: Vec<(String, String)>;
                        let loading_text = vec![(
                            "Stats".into(),
                            "Loading".to_owned() + &".".repeat((self.animation_state % 4).into()),
                        )];

                        let mut info_dict = self.node_id_to_analysis.lock().unwrap();

                        if let Some(node_info) = info_dict.get(&k) {
                            match node_info {
                                AsyncDataAnalysis::Loading => stats_text = loading_text,
                                AsyncDataAnalysis::Ready(val) => match val {
                                    analysis::AnalysisResult::Failed(s) => {
                                        stats_text =
                                            vec![("Stats".into(), format!("Failed! ({})", s))];
                                    }
                                    analysis::AnalysisResult::NotAvailable => {
                                        stats_text = vec![("Stats".into(), "Not available".into())];
                                    }
                                    analysis::AnalysisResult::Stats(stats) => {
                                        stats_text = stats.to_vec();
                                    }
                                },
                            }
                        } else {
                            // we need to kick of the processing for this dataset
                            stats_text = loading_text;
                            info_dict.insert(k.clone(), AsyncDataAnalysis::Loading);

                            let thread_arc: Arc<Mutex<HashMap<NodeIdT, AsyncDataAnalysis>>> =
                                Arc::clone(&self.node_id_to_analysis);
                            let thread_dataset = dataset.clone();
                            tokio::task::spawn_blocking(move || {
                                let analysis = analysis::hdf5_dataset_analysis(thread_dataset);

                                let processed_analysis = analysis
                                    .unwrap_or_else(|s| AnalysisResult::Failed(s.to_string()));

                                let mut info_dict = thread_arc.lock().unwrap();
                                info_dict.insert(
                                    k.clone(),
                                    AsyncDataAnalysis::Ready(processed_analysis),
                                );
                            });
                        };

                        info.extend(stats_text);

                        Some(info)
                    }
                    Some(Hdf5Object::Group(group)) => {
                        let size = tree_node.recursive_storage_data_size;
                        Some(h5_utils::get_text_for_group(&group, size))
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

        if self.last_tree_area.contains(position) {
            self.mode = SelectionMode::TreeBrowsing;
        } else if self.last_object_info_area.contains(position) {
            self.mode = SelectionMode::ObjectInfoInspecting;
        } else if self.last_search_query_area.contains(position) {
            self.mode = SelectionMode::SearchQueryEditing;
        }

        if let Some(id) = self.tree_state.rendered_at(position) {
            let arg = id.to_vec();
            self.tree_state.toggle(arg.clone());
            self.tree_state.select(arg);
        }
    }

    fn on_keypress_normal_mode(&mut self, keycode: KeyCode) -> () {
        match keycode {
            KeyCode::Left => {
                if self.filtered_tree.is_some() {
                    self.tree_state.key_left();
                }
            }
            KeyCode::Char('h') => {
                if self.filtered_tree.is_some() {
                    self.tree_state.key_left();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.filtered_tree.is_some() {
                    self.tree_state.key_up();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.filtered_tree.is_some() {
                    self.tree_state.key_down();
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.mode = SelectionMode::ObjectInfoInspecting;
            }
            KeyCode::Home => {
                if self.filtered_tree.is_some() {
                    self.tree_state.select_first();
                }
            }
            KeyCode::End => {
                if self.filtered_tree.is_some() {
                    self.tree_state.select_last();
                }
            }
            KeyCode::Enter => {
                if self.filtered_tree.is_some() {
                    self.tree_state.toggle_selected();
                }
            }
            KeyCode::Char('c') => {
                if self.filtered_tree.is_some() {
                    self.tree_state.toggle_selected();
                }
            }
            KeyCode::Tab => {
                if self.filtered_tree.is_some() {
                    self.on_tab();
                }
            }
            KeyCode::BackTab => {
                if self.filtered_tree.is_some() {
                    self.on_shift_tab();
                }
            }
            KeyCode::Char('f') => {
                self.open_all_tree_nodes();
            }
            KeyCode::Char('g') => {
                if self.filtered_tree.is_some() {
                    // it's a lot easier to go the first one this way than to use "gg" like in vim
                    self.tree_state.select_first();
                }
            }
            KeyCode::Char('G') => {
                if self.filtered_tree.is_some() {
                    self.tree_state.select_last();
                }
            }
            KeyCode::Char('?') => {
                self.show_logs = !self.show_logs;
            }
            KeyCode::PageDown => {
                if self.filtered_tree.is_some() {
                    self.tree_state.select_relative(|current| {
                        current.map_or(0, |current| current.saturating_add(50))
                    });
                }
            }
            KeyCode::PageUp => {
                if self.filtered_tree.is_some() {
                    self.tree_state.select_relative(|current| {
                        current.map_or(0, |current| current.saturating_sub(50))
                    });
                }
            }
            _ => {}
        }
    }

    pub fn search_query_and_cursor(&self, max_len: usize) -> (String, usize) {
        let rev_right: String = self.search_query_right.chars().rev().collect();
        let text = self.search_query_left.clone() + &rev_right;
        let cursor_pos = self.search_query_left.len();

        if cursor_pos > max_len {
            return (last_chars(&text, max_len).into(), max_len);
        } else {
            return (first_chars(&text, max_len).into(), cursor_pos);
        }
    }

    fn on_tab(&mut self) -> () {
        self.tree_state
            .select_relative(|x| x.map_or(0, |current| current.saturating_add(1)));
    }

    fn on_shift_tab(&mut self) -> () {
        self.tree_state
            .select_relative(|x| x.map_or(0, |current| current.saturating_sub(1)));
    }

    fn on_keypress_search_mode(&mut self, key: crossterm::event::KeyEvent) {
        let keycode = key.code;
        let mut refresh_filtered_tree = true;
        match keycode {
            KeyCode::Char(to_insert) => {
                self.search_query_left.push(to_insert);
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Move cursor to start of previous word
                    while let Some(c) = self.search_query_left.pop() {
                        self.search_query_right.push(c);
                        if c.is_whitespace() {
                            break;
                        }
                    }
                } else {
                    self.search_query_left
                        .pop()
                        .map(|c| self.search_query_right.push(c));
                }
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Move cursor to start of next word
                    while let Some(c) = self.search_query_right.pop() {
                        self.search_query_left.push(c);
                        if c.is_whitespace() {
                            break;
                        }
                    }
                } else {
                    self.search_query_right
                        .pop()
                        .map(|c| self.search_query_left.push(c));
                }
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

    fn on_keypress_object_info_mode(&mut self, keycode: crossterm::event::KeyCode) {
        match keycode {
            KeyCode::Up | KeyCode::Char('k') => {
                self.object_info_scroll_state = self.object_info_scroll_state.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.object_info_scroll_state = self.object_info_scroll_state.saturating_add(1);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.mode = SelectionMode::TreeBrowsing;
            }
            KeyCode::PageDown => {
                // This gets clamped when the ui figures out how many lines we have
                self.object_info_scroll_state = self.object_info_scroll_state.saturating_add(50);
            }
            KeyCode::PageUp => {
                self.object_info_scroll_state = self.object_info_scroll_state.saturating_sub(50);
            }
            KeyCode::End => {
                self.object_info_scroll_state = u16::MAX;
            }
            KeyCode::Home => {
                self.object_info_scroll_state = 0;
            }
            KeyCode::Char('?') => {
                self.show_logs = !self.show_logs;
            }
            _ => {}
        };
    }

    fn update_filtered_tree(&mut self) {
        let query = &self.search_query_and_cursor(usize::MAX).0;
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
        match &self.filtered_tree {
            Some(filtered_tree) => {
                let nothing_selected = self.tree_state.selected().is_empty();
                let selected_item = filtered_tree.get_selected_node(&self.tree_state.selected());
                let selected_item_is_in_tree = selected_item.is_some();
                let selected_item_is_direct_match =
                    selected_item.map_or(false, |t| t.is_direct_match);

                if nothing_selected || !selected_item_is_in_tree || !selected_item_is_direct_match {
                    let first_match = filtered_tree.path_to_first_match();
                    self.tree_state.select(first_match.clone());
                    for i in 0..first_match.len() {
                        self.tree_state.open(first_match[0..i].to_vec());
                    }
                }
                self.tree_state.scroll_selected_into_view();
            }
            None => {
                self.tree_state.select(vec![]);
            }
        }
    }

    pub fn set_last_object_info_area(&mut self, area: Rect) {
        self.last_object_info_area = area;
    }

    pub fn set_last_tree_area(&mut self, area: Rect) {
        self.last_tree_area = area;
    }

    pub fn set_last_search_query_area(&mut self, area: Rect) {
        self.last_search_query_area = area;
    }

    fn open_all_tree_nodes(&mut self) {
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
            if let Some(last_selected) = &self.tree_state_last_rendered_selected {
                if last_selected != self.tree_state.selected() {
                    // if the selected node has changed, reset the scroll state
                    self.object_info_scroll_state = 0;
                }
            }
            terminal.draw(|frame| ui(frame, &mut self))?;
            self.tree_state_last_rendered_selected = Some(self.tree_state.selected().to_vec());

            tokio::select! {
                Some(event) = events.receiver.recv() => {
                    match event {
                        events::Event::AnimationTick => {
                            self.animation_state = self.animation_state.wrapping_add(1);
                        }
                        events::Event::Key(key) => self.handle_keypress(key),
                        events::Event::Mouse(mouse) => self.handle_mouse(mouse),
                        events::Event::Resize => {}
                    }
                }
                Some(tree) = rx.recv() => {
                    self.tree = Some(tree);
                    // self.tree_state.open(vec![self.tree.as_ref().unwrap().id()]);
                    self.open_all_tree_nodes();
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
                SelectionMode::TreeBrowsing => match key.code {
                    KeyCode::Char('q') => {
                        self.running = false;
                    }
                    KeyCode::Char('/') => {
                        self.mode = SelectionMode::SearchQueryEditing;
                    }
                    other => {
                        self.on_keypress_normal_mode(other);
                    }
                },
                SelectionMode::SearchQueryEditing => match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.mode = SelectionMode::TreeBrowsing;
                    }
                    _ => {
                        self.on_keypress_search_mode(key);
                    }
                },
                SelectionMode::ObjectInfoInspecting => match key.code {
                    KeyCode::Char('q') => {
                        self.running = false;
                    }
                    KeyCode::Char('/') => {
                        self.mode = SelectionMode::SearchQueryEditing;
                    }
                    other => {
                        self.on_keypress_object_info_mode(other);
                    }
                },
            }
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        // log::debug!("mouse event: {:?}", mouse);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.on_click(mouse.column, mouse.row),
            MouseEventKind::ScrollDown => {
                if self
                    .last_object_info_area
                    .contains(Position::new(mouse.column, mouse.row))
                {
                    self.object_info_scroll_state = self.object_info_scroll_state.saturating_add(1);
                    self.mode = SelectionMode::ObjectInfoInspecting;
                } else if self
                    .last_tree_area
                    .contains(Position::new(mouse.column, mouse.row))
                {
                    self.tree_state.scroll_down(1);
                }
            }
            MouseEventKind::ScrollUp => {
                if self
                    .last_object_info_area
                    .contains(Position::new(mouse.column, mouse.row))
                {
                    self.object_info_scroll_state = self.object_info_scroll_state.saturating_sub(1);
                    self.mode = SelectionMode::ObjectInfoInspecting;
                } else if self
                    .last_tree_area
                    .contains(Position::new(mouse.column, mouse.row))
                {
                    self.tree_state.scroll_up(1);
                }
            }
            _ => {}
        }
    }
}
