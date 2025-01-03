use crate::app::{App, Mode};
use crate::tree;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use tui_tree_widget::Tree as WidgetTreeRoot;
use tui_tree_widget::TreeItem as WidgetTreeItem;

use std::hash::Hash;
const STYLE_HIGHLIGHT: Style = Style::new().fg(Color::White).bg(Color::Gray);
const STYLE_EXTRA_INFO: Style = Style::new().fg(Color::Gray);
const STYLE_MATCH: Style = Style::new().fg(Color::Magenta).bg(Color::Yellow);

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Length(40), Constraint::Min(0)]).split(frame.area());

    let left_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(chunks[0]);

    render_search(frame, app, left_layout[1]);
    render_tree(frame, app, left_layout[0]);
    render_object_info(frame, app, chunks[1]);
}
impl<IdT> tree::TreeNode<IdT>
where
    IdT: Eq + Hash + Clone,
{
    pub fn into_tree_item(&self) -> WidgetTreeItem<IdT> {
        let children: Vec<_> = self
            .children()
            .iter()
            .map(|child| child.into_tree_item())
            .collect();

        let matching_indices = self.matching_indices();
        let mut formatted_text = Line::from(
            self.text()
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if matching_indices.contains(&i) {
                        Span::styled(c.to_string(), STYLE_MATCH)
                    } else {
                        Span::raw(c.to_string())
                    }
                })
                .collect::<Vec<_>>(),
        );

        let num_children = self.recursive_num_children();
        if num_children > 0 {
            formatted_text.push_span(Span::styled(
                format!(" ({})", num_children),
                STYLE_EXTRA_INFO,
            ));
        }

        WidgetTreeItem::new(self.id(), formatted_text, children)
            .expect("Already checked for duplicate IDs")
    }
}

fn render_object_info(frame: &mut Frame, app: &mut App, area: Rect) {
    let object_info = Block::new()
        .title("object info")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let selected = app.tree_state.selected();
    let mut text = "Select on the left".to_string();
    if !selected.is_empty() {
        // selected is of form: ["/group1", "/group1/dataset1"]
        text = app.get_text_for(selected.last().unwrap().clone());
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
        .title(app.h5_file_path.to_str().unwrap_or("unknown file"))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let query = &app.search_query_and_cursor().0;
    match app.tree.filter(query) {
        Some(filtered_tree) => {
            let filtered_items = filtered_tree.into_tree_item();
            // Use root's children instead of root
            // app.tree_state.select(vec!["group1".to_string()]);

            let items = filtered_items.children();
            let tree_widget = WidgetTreeRoot::new(items)
                .expect("all item identifiers are unique")
                .highlight_style(STYLE_HIGHLIGHT)
                .block(tree_block);

            // println!("selected: {:?}", app.tree_state.selected());

            if !filtered_tree.contains_path(&app.tree_state.selected()) {
                app.tree_state.select(vec![]);
            }

            if app.tree_state.selected().is_empty() {
                app.tree_state.select(vec![items
                    .first()
                    .map_or(0, |item| item.identifier().clone())]);
            }

            frame.render_stateful_widget(tree_widget, area, &mut app.tree_state);
        }
        None => {
            frame.render_widget(
                Paragraph::new("No matches found")
                    .centered()
                    .block(tree_block)
                    .style(Style::default().bg(Color::Red)),
                area,
            );
        }
    }
}
