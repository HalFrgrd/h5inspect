use crate::app::{App, Mode};
use ratatui::{
    layout::{Constraint, Layout, Position},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::Tree;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Length(40), Constraint::Min(0)]).split(frame.area());

    let left_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(chunks[0]);

    let search_block = Block::new()
        .title("Search")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let (search_query_text, search_query_cursor_pos) = app.search_query_and_cursor();
    let search_query = Paragraph::new(search_query_text.as_str()).block(search_block);
    frame.render_widget(search_query, left_layout[1]);
    match app.mode {
        Mode::Normal => {},
        Mode::SearchQueryEditing => {
            frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                left_layout[1].x + search_query_cursor_pos as u16 + 1,
                // Move one line down, from the border to the input line
                left_layout[1].y + 1,
            ))
        },
    }

    let tree_block = Block::new()
        .title(app.h5_file_path.to_str().unwrap_or("asd"))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    // frame.render_widget(tree_block, chunks[0]);

    let object_info = Block::new()
        .title("object info")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    // frame.render_widget(object_info, chunks[1]);

    // let selected = app.tree_state.selected();
    // if selected.is_empty() {
    //     app.tree_state.select_first();
    // }

    let tree_widget = Tree::new(&app.tree_items)
        .expect("all item identifiers are unique")
        .highlight_style(Style::new().fg(Color::Black).bg(Color::Blue))
        // .node_no_children_symbol(">")
        .block(tree_block);
    frame.render_stateful_widget(tree_widget, left_layout[0], &mut app.tree_state);

    // let selected = app.tree_state.selected() {}
    // let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
    // let text

    let selected = app.tree_state.selected();
    let mut text = "Select on the left".to_string();
    if !selected.is_empty() {
        // println!("{selected:?}");
        // selected is of form: ["/", "/group1", "/group1/dataset1"]
        text = app.get_text_for(selected.last().unwrap());
    }
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    frame.render_widget(paragraph.clone().block(object_info), chunks[1]);
}
