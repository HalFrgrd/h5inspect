use crate::app::App;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::Tree;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Length(40), Constraint::Min(0)]).split(frame.area());

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
    frame.render_stateful_widget(tree_widget, chunks[0], &mut app.tree_state);

    // let selected = app.tree_state.selected() {}
    // let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
    // let text

    let selected = app.tree_state.selected();
    let mut text = "Select on the left";
    if !selected.is_empty() {
        // println!("{selected:?}");
        // selected is of form: ["/", "/group1", "/group1/dataset1"]
        text = app.get_text_for(selected.last().unwrap());
    }
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    frame.render_widget(paragraph.clone().block(object_info), chunks[1]);
}
