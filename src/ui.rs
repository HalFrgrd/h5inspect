use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::app::App;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Length(40), Constraint::Min(0)]).split(frame.area());

    let tree_block = Block::new()
        .title(app.file_name.clone())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    // frame.render_widget(tree_block, chunks[0]);

    let object_info = Block::new()
        .title("object info")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    frame.render_widget(object_info, chunks[1]);

    let tree_widget = Tree::new(&app.tree_items)
        .expect("all item identifiers are unique")
        .highlight_style(Style::new().fg(Color::Black).bg(Color::Blue))
        .block(tree_block);
    frame.render_stateful_widget(tree_widget, chunks[0], &mut app.tree_state);
}
