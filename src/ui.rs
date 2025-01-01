use crate::app::{App, Mode};
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::Tree;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Length(40), Constraint::Min(0)]).split(frame.area());
    render_object_info(frame, app, chunks[1]);

    let left_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(chunks[0]);

    render_search(frame, app, left_layout[1]);
    render_tree(frame, app, left_layout[0]);
}

fn render_object_info(frame: &mut Frame, app: &mut App, area: Rect) {
    let object_info = Block::new()
        .title("object info")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let selected = app.tree_state.selected();
    let mut text = "Select on the left".to_string();
    if !selected.is_empty() {
        // println!("{selected:?}");
        // selected is of form: ["/", "/group1", "/group1/dataset1"]
        text = app.get_text_for(selected.last().unwrap());
    }
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    frame.render_widget(paragraph.clone().block(object_info), area);
}

fn render_search(frame: &mut Frame, app: &mut App, area: Rect) {
    let search_block = Block::new()
        .title("Search (type '/')")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let (search_query_text, search_query_cursor_pos) = app.search_query_and_cursor();
    let search_query = Paragraph::new(search_query_text.as_str()).block(search_block);
    frame.render_widget(search_query, area);
    match app.mode {
        Mode::Normal => {}
        Mode::SearchQueryEditing => {
            frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                area.x + search_query_cursor_pos as u16 + 1,
                // Move one line down, from the border to the input line
                area.y + 1,
            ))
        }
    }
}

fn render_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    let tree_block = Block::new()
        .title("asd")
        // .title(app.h5_file_path.to_str().unwrap_or("asd"))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let tree_widget = Tree::new(&app.tree_items)
        .expect("all item identifiers are unique")
        .highlight_style(Style::new().fg(Color::Black).bg(Color::Blue))
        // .node_no_children_symbol(">")
        .block(tree_block);
    frame.render_stateful_widget(tree_widget, area, &mut app.tree_state);
}

// fn ismatch(name: &str) -> bool {
//     name.contains("qwe")
// }

// fn filter_tree_items(tree_item: &TreeItem<String>) -> TreeItem<String> {
//     let mut filtered = tree_item.clone();
//     filtered.children().retain(|child| {
//         let name = child.data.as_str();
//         ismatch(name) || !child.children.is_empty()
//     });

//     for child in &mut filtered.children() {
//         *child = filter_tree_items(child);
//     }

//     filtered
// }
