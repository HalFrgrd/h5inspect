use crate::app::{App, Mode};
use crate::tree;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use tui_logger;
use tui_tree_widget::Tree as WidgetTreeRoot;
use tui_tree_widget::TreeItem as WidgetTreeItem;

use std::hash::Hash;
const STYLE_HIGHLIGHT: Style = Style::new().bg(Color::DarkGray);
const STYLE_DEFAULT_TEXT: Style = Style::new().fg(Color::White);
const STYLE_EXTRA_INFO: Style = Style::new().fg(Color::Gray);
const STYLE_MATCH: Style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Min(0)]).split(frame.area());

    let left_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(chunks[0]);

    render_search(frame, app, left_layout[1]);
    render_tree(frame, app, left_layout[0]);

    let right_layout =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).split(chunks[1]);

    if app.show_logs {
        render_object_info(frame, app, right_layout[0]);
        render_logger(frame, right_layout[1]);
    } else {
        render_object_info(frame, app, chunks[1]);
    }

    app.set_last_object_info_area(right_layout[0]);
    app.set_last_tree_area(left_layout[0]);
}
impl<IdT> tree::TreeNode<IdT>
where
    IdT: Eq + Hash + Clone + std::fmt::Debug,
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
        .title("Object info")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let selected = app.tree_state.selected();
    let mut paragraph = Paragraph::new("Select on the left".to_string());
    if !selected.is_empty() {
        // selected is of form: ["/group1", "/group1/dataset1"]
        let info = app.get_text_for(selected);
        if let Some(info) = info {
            let mut lines = vec![];

            for (key, value) in info {
                let lines_in_value: Vec<_> = value.split('\n').map(|s| s.to_string()).collect();
                let key_width = 24_usize;
                let spacing = " ".repeat(key_width.saturating_sub(key.chars().count()));

                // First line with key
                lines.push(Line::from(vec![
                    Span::raw(key),
                    Span::raw(spacing),
                    Span::styled(
                        lines_in_value[0].clone(),
                        Style::default().fg(Color::Magenta),
                    ),
                ]));

                // Subsequent lines indented
                for line in lines_in_value.iter().skip(1) {
                    lines.push(Line::from(vec![
                        Span::raw(" ".repeat(key_width)),
                        Span::styled(line.clone(), Style::default().fg(Color::Magenta)),
                    ]));
                }
            }

            paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        }
    }

    let num_lines_when_rendered: u16 = paragraph.line_count(area.width).try_into().unwrap();
    let max_scroll_state = num_lines_when_rendered.saturating_sub(area.height);
    app.object_info_scroll_state = app.object_info_scroll_state.clamp(0, max_scroll_state);
    frame.render_widget(
        paragraph
            .clone()
            .block(object_info)
            .scroll((app.object_info_scroll_state, 0)),
        area,
    );
}

fn render_search(frame: &mut Frame, app: &mut App, area: Rect) {
    let search_block = Block::new()
        .title("Fuzzy search (type '/')")
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
        .title(
            app.h5_file_path
                .to_str()
                .unwrap_or("unknown file")
                .to_string(),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    match &app.tree {
        Some(_) => match &app.filtered_tree {
            Some(filtered_tree) => {
                let filtered_items = [filtered_tree.into_tree_item()];
                let tree_widget = WidgetTreeRoot::new(&filtered_items)
                    .expect("all item identifiers are unique")
                    .style(STYLE_DEFAULT_TEXT)
                    .highlight_style(STYLE_HIGHLIGHT)
                    .block(tree_block);

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
        },
        None => {
            frame.render_widget(
                Paragraph::new("Loading tree...")
                    .centered()
                    .block(tree_block),
                area,
            );
        }
    }
}

fn render_logger(frame: &mut Frame, area: Rect) {
    let logger_widget = tui_logger::TuiLoggerWidget::default()
        .block(
            Block::bordered()
                .title("Logs (toggle with '?')")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(tui_logger::TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false);
    frame.render_widget(logger_widget, area);
}
