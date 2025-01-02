use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use std::collections::HashSet;

type NodeId = String;

pub const STYLE_DARKGRAY: Style = Style::new().fg(Color::Gray);
const STYLE_MATCH: Style = Style::new().fg(Color::Magenta);

#[derive(Debug, Clone)]
pub struct TreeNode {
    id: NodeId,
    text: String,
    children: Vec<TreeNode>,
    recursive_num_children: usize,
    matching_indices: Option<Vec<usize>>,
}

impl TreeNode {
    pub fn new(
        id: impl Into<String>,
        text: String,
        children: Vec<TreeNode>,
    ) -> std::io::Result<Self> {
        // Check for duplicate IDs among children
        let ids: HashSet<_> = children.iter().map(|child| &child.id).collect();

        if ids.len() != children.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Children contain duplicate IDs",
            ));
        }

        let recursive_num_children: usize = children
            .iter()
            .map(|child| child.recursive_num_children)
            .sum::<usize>()
            + children.len();

        Ok(Self {
            id: id.into(),
            text,
            children,
            recursive_num_children,
            matching_indices: None,
        })
    }

    /// Get a reference to this node's ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get a reference to this node's children
    pub fn children(&self) -> &[TreeNode] {
        &self.children
    }

    fn ismatch(haystack: &str, needle: &str) -> Option<Vec<usize>> {
        let matcher = SkimMatcherV2::default();
        matcher
            .fuzzy_indices(haystack, needle)
            .map(|(_, indices)| indices)
    }

    /// Filter this node and its children based on a search query
    /// Returns None if neither this node nor any children match
    pub fn filter(&self, query: &str) -> Option<TreeNode> {
        let indices = Self::ismatch(&self.text, query);
        let i_match = indices.is_some();

        let matching_children: Vec<_> = self
            .children
            .iter()
            .filter_map(|child| child.filter(query))
            .collect();

        let recursive_num_children: usize = matching_children
            .iter()
            .map(|child| child.recursive_num_children)
            .sum::<usize>()
            + matching_children.len();

        if i_match || !matching_children.is_empty() {
            Some(TreeNode {
                id: self.id.clone(),
                text: self.text.clone(),
                children: matching_children,
                recursive_num_children,
                matching_indices: indices,
            })
        } else {
            None
        }
    }

    /// Convert this TreeNode into a tui_tree_widget::TreeItem
    pub fn into_tree_item(&self) -> tui_tree_widget::TreeItem<NodeId> {
        let children: Vec<_> = self
            .children
            .iter()
            .map(|child| child.into_tree_item())
            .collect();

        let mut formatted_text = if let Some(indices) = &self.matching_indices {
            let mut spans = self
                .text
                .chars()
                .map(|c| Span::raw(c.to_string()))
                .collect::<Vec<_>>();
            for index in indices {
                spans[*index].style = STYLE_MATCH;
            }
            Line::from(spans)
        } else {
            Line::from(self.text.clone())
        };

        if self.recursive_num_children > 0 {
            formatted_text.push_span(Span::styled(
                format!(" ({})", self.recursive_num_children),
                STYLE_DARKGRAY,
            ));
        }

        tui_tree_widget::TreeItem::new(self.id.clone(), formatted_text, children)
            .expect("Already checked for duplicate IDs")
    }

    pub fn contains_path(&self, path: &[NodeId]) -> bool {
        if path.is_empty() {
            return true;
        }

        self.children
            .iter()
            .any(|child| child.id == path[0] && child.contains_path(&path[1..]))
    }
}
