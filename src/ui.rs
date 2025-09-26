use crate::app::{App, SelectionMode};
use crate::tree;
use num_traits::clamp;
use ratatui::layout::Margin;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{
        Block, BorderType, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    Frame,
};

use tui_logger;
use tui_tree_widget::Tree as WidgetTreeRoot;
use tui_tree_widget::TreeItem as WidgetTreeItem;

use std::hash::Hash;
const STYLE_HIGHLIGHT: Style = Style::new().bg(Color::DarkGray);
const STYLE_DEFAULT_TEXT: Style = Style::new().fg(Color::White);
const STYLE_MATCH: Style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);
const STYLE_MAGENTA: Style = Style::new().fg(Color::Magenta);
const COLOR_BORDER_HIGHLIGHT: Color = Color::Red;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Min(0)]).split(frame.area());

    let left_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(chunks[0]);

    render_search(frame, app, left_layout[1]);
    render_tree(frame, app, left_layout[0]);

    let right_layout =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).split(chunks[1]);

    let object_info_area = if app.show_logs {
        right_layout[0]
    } else {
        chunks[1]
    };

    if app.show_logs {
        render_logger(frame, right_layout[1]);
    }
    render_object_info(frame, app, object_info_area);
    app.set_last_object_info_area(object_info_area);
    app.set_last_tree_area(left_layout[0]);
    app.set_last_search_query_area(left_layout[1]);
}
impl<IdT> tree::TreeNode<IdT>
where
    IdT: Eq + Hash + Clone + std::fmt::Debug,
{
    pub fn into_tree_item(&self) -> WidgetTreeItem<'_, IdT> {
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
            formatted_text.push_span(Span::styled(format!(" ({})", num_children), STYLE_MAGENTA))
        }

        WidgetTreeItem::new(self.id(), formatted_text, children)
            .expect("Already checked for duplicate IDs")
    }
}

fn render_object_info(frame: &mut Frame, app: &mut App, area: Rect) {
    let num_active_tasks = app.get_num_active_data_analysis_tasks();

    let object_info = Block::new()
        .title("Object info")
        .title_bottom(
            Line::from(format!("# background analysis tasks: {}", num_active_tasks))
                .left_aligned()
                .style(STYLE_DEFAULT_TEXT.add_modifier(Modifier::DIM)),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default().fg(if let SelectionMode::ObjectInfoInspecting = app.mode {
                COLOR_BORDER_HIGHLIGHT
            } else {
                Color::White
            }),
        );

    // frame.render_widget(object_info, area);

    let mut rows = vec![];

    let key_col_width = 20;
    let table_widths = [Constraint::Min(key_col_width), Constraint::Percentage(100)];
    let data_col_width = std::cmp::max(area.width.saturating_sub(key_col_width + 3), 2); // 3 for borders

    let selected = app.tree_state.selected().to_vec();
    if !selected.is_empty() {
        // selected is of form: ["/group1", "/group1/dataset1"]
        let info = app.get_text_for(&selected);
        if let Some((info, hist_data_opt)) = info {
            info.iter().for_each(|(k, v)| {
                v.split('\n')
                    .map(|line| {
                        line.chars()
                            .collect::<Vec<_>>()
                            .chunks(data_col_width.into())
                            .map(|chunk| chunk.iter().collect::<String>())
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .enumerate()
                    .for_each(|(i, subrow)| {
                        let sub_row_k = if i == 0 { k } else { "" };
                        rows.push(Row::new([
                            Cell::from(Text::from(sub_row_k.to_owned())),
                            Cell::from(Text::from(subrow.to_owned()).style(STYLE_MAGENTA)),
                        ]));
                    });
            });
        }
    }

    let num_lines_when_rendered: u16 = rows.len().try_into().unwrap();
    let max_scroll_state = num_lines_when_rendered.saturating_sub(area.height - 2);
    app.object_info_scroll_state = app.object_info_scroll_state.clamp(0, max_scroll_state);

    let table = Table::new(rows, table_widths);
    let mut table_scroll = TableState::new().with_offset(app.object_info_scroll_state.into());
    frame.render_stateful_widget(table.block(object_info), area, &mut table_scroll);

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(max_scroll_state.into())
        .viewport_content_length((area.height).into())
        .position(app.object_info_scroll_state.into());

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .track_symbol(None)
            .end_symbol(None),
        area.inner(Margin::new(0, 1)),
        &mut scrollbar_state,
    );
}

fn render_search(frame: &mut Frame, app: &mut App, area: Rect) {
    let search_block = Block::new()
        .title("Fuzzy search (type '/')")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default().fg(if let SelectionMode::SearchQueryEditing = app.mode {
                COLOR_BORDER_HIGHLIGHT
            } else {
                Color::White
            }),
        );

    let (search_query_text, mut search_query_cursor_pos) = app.search_query_and_cursor();

    let view_width = area.width - 2;
    let min_offset = search_query_cursor_pos.saturating_sub(view_width);
    let max_offset = std::cmp::max(
        search_query_cursor_pos.saturating_sub(view_width / 2),
        min_offset,
    );
    app.search_query_view_offset = clamp(app.search_query_view_offset, min_offset, max_offset);
    search_query_cursor_pos = search_query_cursor_pos.saturating_sub(app.search_query_view_offset);
    let visible_search_query: String = search_query_text
        .chars()
        .skip(app.search_query_view_offset.into())
        .take(view_width.into())
        .collect();

    let search_query = Paragraph::new(visible_search_query).block(search_block);
    frame.render_widget(search_query, area);
    match app.mode {
        SelectionMode::SearchQueryEditing => {
            frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                area.x + search_query_cursor_pos as u16 + 1,
                // Move one line down, from the border to the input line
                area.y + 1,
            ))
        }
        _ => {}
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
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default().fg(if let SelectionMode::TreeBrowsing = app.mode {
                COLOR_BORDER_HIGHLIGHT
            } else {
                Color::White
            }),
        );

    match &app.tree {
        Some(_) => match &app.filtered_tree {
            Some(filtered_tree) => {
                let filtered_items = [filtered_tree.into_tree_item()];
                let tree_widget = WidgetTreeRoot::new(&filtered_items)
                    .expect("all item identifiers are unique")
                    .style(STYLE_DEFAULT_TEXT)
                    .highlight_style(STYLE_HIGHLIGHT)
                    .block(tree_block)
                    .experimental_scrollbar(Some(
                        Scrollbar::new(ScrollbarOrientation::VerticalRight)
                            .begin_symbol(None)
                            .track_symbol(None)
                            .end_symbol(None),
                    ));

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
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        )
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(tui_logger::TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false);
    frame.render_widget(logger_widget, area);
}
