use crate::app::{App, SelectionMode};
use crate::hist_plot;
use crate::tree;

use num_traits::clamp;

use ratatui::layout::Margin;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::{
    layout::{Constraint, Flex, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    Frame,
};
use tui_big_text::{BigText, PixelSize};
use tui_logger;
use tui_tree_widget::Tree as WidgetTreeRoot;
use tui_tree_widget::TreeItem as WidgetTreeItem;

pub const MAGENTA_R: u8 = 0xfc;
pub const MAGENTA_G: u8 = 0x4c;
pub const MAGENTA_B: u8 = 0xb4;

enum Styles {
    TreeItemHighlight,
    DefaultText,
    SearchCharMatch,
    Magenta,
    BorderHighlight,
    BorderDefault,
    LogBorder,
}

fn get_style(style: Styles, mode: SelectionMode) -> Style {
    let s = match style {
        Styles::TreeItemHighlight => Style::new().bg(Color::DarkGray),
        Styles::DefaultText => Style::new().fg(Color::White),
        Styles::SearchCharMatch => Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        Styles::Magenta => Style::new().fg(Color::Rgb(MAGENTA_R, MAGENTA_G, MAGENTA_B)),
        Styles::BorderHighlight => Style::default().fg(Color::Red),
        Styles::BorderDefault => Style::default().fg(Color::White),
        Styles::LogBorder => Style::default().fg(Color::Blue),
    };

    if let SelectionMode::HelpScreen = mode {
        s.add_modifier(Modifier::DIM)
    } else {
        s
    }
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Min(0)]).split(frame.area());

    let left_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(chunks[0]);

    render_search(frame, app, left_layout[1]);
    render_tree(frame, app, left_layout[0]);

    let right_layout =
        Layout::vertical([Constraint::Percentage(85), Constraint::Min(0)]).split(chunks[1]);

    let object_info_area = if app.show_logs {
        right_layout[0]
    } else {
        chunks[1]
    };

    if app.show_logs {
        render_logger(frame, app, right_layout[1]);
    }
    render_object_info(frame, app, object_info_area);

    if let SelectionMode::HelpScreen = app.mode {
        render_help_screen(frame, app, popup_area(frame.area(), 80, 80));
    }
    app.set_last_object_info_area(object_info_area);
    app.set_last_tree_area(left_layout[0]);
    app.set_last_search_query_area(left_layout[1]);
}
impl<IdT> tree::TreeNode<IdT>
where
    IdT: Eq + std::hash::Hash + Clone + std::fmt::Debug,
{
    pub fn into_tree_item(&self, mode: SelectionMode) -> WidgetTreeItem<'_, IdT> {
        let children: Vec<_> = self
            .children()
            .iter()
            .map(|child| child.into_tree_item(mode))
            .collect();

        let matching_indices = self.matching_indices();
        let mut formatted_text = Line::from(
            self.text()
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if matching_indices.contains(&i) {
                        Span::styled(c.to_string(), get_style(Styles::SearchCharMatch, mode))
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
                get_style(Styles::Magenta, mode),
            ));
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
                .style(get_style(Styles::DefaultText, app.mode).add_modifier(Modifier::DIM)),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if let SelectionMode::ObjectInfoInspecting = app.mode {
            get_style(Styles::BorderHighlight, app.mode)
        } else {
            get_style(Styles::BorderDefault, app.mode)
        });

    frame.render_widget(&object_info, area);

    let layout = Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area.inner(Margin::new(1, 1)));
    let table_area = layout[0];
    let chart_area = layout[1];

    let mut rows = vec![];

    let key_col_width = 20;
    let table_widths = [Constraint::Min(key_col_width), Constraint::Percentage(100)];
    let data_col_width = std::cmp::max(area.width.saturating_sub(key_col_width + 3), 2); // 3 for borders

    let mut histogram_data: Option<_> = None;

    let selected = app.tree_state.selected().to_vec();
    if !selected.is_empty() {
        // selected is of form: ["/group1", "/group1/dataset1"]
        let info = app.get_text_for(&selected);
        if let Some((info, hist_data_opt)) = info {
            histogram_data = hist_data_opt;
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
                            Cell::from(
                                Text::from(sub_row_k.to_owned())
                                    .style(get_style(Styles::DefaultText, app.mode)),
                            ),
                            Cell::from(
                                Text::from(subrow.to_owned())
                                    .style(get_style(Styles::Magenta, app.mode)),
                            ),
                        ]));
                    });
            });
        }
    }

    let num_lines_when_rendered: u16 = rows.len().try_into().unwrap();
    let max_scroll_state = num_lines_when_rendered.saturating_sub(table_area.height);
    app.object_info_scroll_state = app.object_info_scroll_state.clamp(0, max_scroll_state);

    let table = Table::new(rows, table_widths);
    let mut table_scroll = TableState::new().with_offset(app.object_info_scroll_state.into());
    frame.render_stateful_widget(table, table_area, &mut table_scroll);

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(max_scroll_state.into())
        .viewport_content_length((table_area.height).into())
        .position(app.object_info_scroll_state.into());

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .track_symbol(None)
            .end_symbol(None),
        table_area,
        &mut scrollbar_state,
    );

    if let Some(hist_data) = histogram_data {
        let histogram_widget =
            hist_plot::histogram_widget(&hist_data, chart_area.height, chart_area.width);
        frame.render_widget(histogram_widget, chart_area);
    }
}

fn render_search(frame: &mut Frame, app: &mut App, area: Rect) {
    let search_block = Block::new()
        .title("Fuzzy search (type '/')")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if let SelectionMode::SearchQueryEditing = app.mode {
            get_style(Styles::BorderHighlight, app.mode)
        } else {
            get_style(Styles::BorderDefault, app.mode)
        });

    let (search_query_text, mut search_query_cursor_pos) = app.search_query_and_cursor();

    let view_width = area.width - 2;
    let min_offset = search_query_cursor_pos.saturating_sub(view_width);
    let max_offset = min_offset.max(search_query_cursor_pos.saturating_sub(view_width / 2));
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
        .border_style(if let SelectionMode::TreeBrowsing = app.mode {
            get_style(Styles::BorderHighlight, app.mode)
        } else {
            get_style(Styles::BorderDefault, app.mode)
        });

    match &app.tree {
        Some(_) => match &app.filtered_tree {
            Some(filtered_tree) => {
                let filtered_items = [filtered_tree.into_tree_item(app.mode)];
                let tree_widget = WidgetTreeRoot::new(&filtered_items)
                    .expect("all item identifiers are unique")
                    .style(get_style(Styles::DefaultText, app.mode))
                    .highlight_style(get_style(Styles::TreeItemHighlight, app.mode))
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

fn render_logger(frame: &mut Frame, app: &App, area: Rect) {
    let logger_widget = tui_logger::TuiLoggerWidget::default()
        .block(
            Block::bordered()
                .title("Logs (toggle with 'L')")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(get_style(Styles::LogBorder, app.mode)),
        )
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(tui_logger::TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false);
    frame.render_widget(logger_widget, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn render_help_screen(frame: &mut Frame, app: &App, area: Rect) {
    let help_block = Block::new()
        .title("Help")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    frame.render_widget(Clear, area);
    frame.render_widget(help_block, area);

    let [title_area, text_area] =
        Layout::vertical([Constraint::Min(20), Constraint::Percentage(100)])
            .areas(area.inner(Margin::new(3, 3)));

    let [_, main_title_area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Min(80),
        Constraint::Fill(1),
    ])
    .areas(title_area);
    let [main_title_area, version_area] =
        Layout::vertical([Constraint::Length(8), Constraint::Length(10)]).areas(main_title_area);

    let big_text = BigText::builder()
        .pixel_size(PixelSize::Full)
        .lines(vec![Line::styled(
            "h5inspect",
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        )])
        .centered()
        .build();

    frame.render_widget(big_text, main_title_area);

    let big_text_version = BigText::builder()
        .pixel_size(PixelSize::Sextant)
        .lines(vec![Line::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        )])
        .right_aligned()
        .build();
    frame.render_widget(big_text_version, version_area);

    const KEY_BINDING_TITLE_STYLE: Style =
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    const KEY_BINDING_STYLE: Style = Style::new().fg(Color::Yellow);
    const DEFAULT_TEXT_STYLE: Style = Style::new().fg(Color::White);
    let help_text = Text::from(vec![
        Line::from("h5inspect - Simple TUI to inspect h5 files"),
        Line::from(""),
        Line::from("Key bindings:").style(KEY_BINDING_TITLE_STYLE),
        Line::from("  Up/Down/Left/Right:").style(KEY_BINDING_STYLE),
        Line::from("    Navigate tree").style(DEFAULT_TEXT_STYLE),
        Line::from("  Enter:").style(KEY_BINDING_STYLE),
        Line::from("    Expand/collapse group").style(DEFAULT_TEXT_STYLE),
        Line::from("  /:").style(KEY_BINDING_STYLE),
        Line::from("    Start editing search query").style(DEFAULT_TEXT_STYLE),
        Line::from("  Esc:").style(KEY_BINDING_STYLE),
        Line::from("    Stop editing search query").style(DEFAULT_TEXT_STYLE),
        Line::from("  Backspace:").style(KEY_BINDING_STYLE),
        Line::from("    Delete last character in search query").style(DEFAULT_TEXT_STYLE),
        Line::from("  Ctrl+U:").style(KEY_BINDING_STYLE),
        Line::from("    Clear search query").style(DEFAULT_TEXT_STYLE),
        Line::from("  n:").style(KEY_BINDING_STYLE),
        Line::from("    Jump to next match in tree").style(DEFAULT_TEXT_STYLE),
        Line::from("  N:").style(KEY_BINDING_STYLE),
        Line::from("    Jump to previous match in tree").style(DEFAULT_TEXT_STYLE),
        Line::from("  i:").style(KEY_BINDING_STYLE),
        Line::from("    Inspect object info (if any)").style(DEFAULT_TEXT_STYLE),
        Line::from("  o:").style(KEY_BINDING_STYLE),
        Line::from("    Stop inspecting object info").style(DEFAULT_TEXT_STYLE),
        Line::from("  ?:").style(KEY_BINDING_STYLE),
        Line::from("    Toggle this help screen").style(DEFAULT_TEXT_STYLE),
        Line::from("  l:").style(KEY_BINDING_STYLE),
        Line::from("    Toggle log view").style(DEFAULT_TEXT_STYLE),
        Line::from("  q:").style(KEY_BINDING_STYLE),
        Line::from("    Quit").style(DEFAULT_TEXT_STYLE),
        Line::from(""),
        Line::from("Press Esc/q/? to close this help screen.").style(DEFAULT_TEXT_STYLE),
    ]);
    let help_paragraph = Paragraph::new(help_text)
        .style(get_style(Styles::DefaultText, app.mode).remove_modifier(Modifier::DIM))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(help_paragraph, text_area);
}
