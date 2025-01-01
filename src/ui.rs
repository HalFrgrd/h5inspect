use crate::app::{App, Mode};
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::{Tree, TreeItem};

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

fn ismatch(name: &str, query: &str) -> bool {
    name.to_lowercase().contains(&query.to_lowercase())
}

fn filter_tree_items(tree_item: &TreeItem<String>, query: &str) -> Option<TreeItem<String>> {
    
    let i_should_stay = ismatch(tree_item.identifier(), query);
    
    let children_to_keep = tree_item.children().iter().filter(|child| {
        ismatch(child.identifier(), query)
    }).collect::<Vec<_>>();

    if i_should_stay || !children_to_keep.is_empty() {
        return Some(
            TreeItem::new(
                tree_item.identifier(),
                tree_item.name(),
                children_to_keep,
            )   
        );
    }
    None
}

fn render_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    let tree_block = Block::new()
        .title("asd")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let filtered_items = if app.filter_tree_using_search {
        let query = &app.search_query_and_cursor().0;
        app.tree_items.iter().filter_map(|item| filter_tree_items(item, query)).collect::<Vec<_>>()
    } else {
        app.tree_items.clone()
    };

    let tree_widget = Tree::new(&filtered_items)
        .expect("all item identifiers are unique")
        .highlight_style(Style::new().fg(Color::Black).bg(Color::Blue))
        .block(tree_block);
    frame.render_stateful_widget(tree_widget, area, &mut app.tree_state);
}
