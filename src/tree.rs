use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct TreeNode {
    id: String,
    text: String,
    children: Vec<TreeNode>,
}

impl TreeNode {
    /// Create a new node with the given children
    ///
    /// # Errors
    /// Returns an error if any children have duplicate IDs
    pub fn new(
        id: impl Into<String>,
        text: impl Into<String>,
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

        Ok(Self {
            id: id.into(),
            text: text.into(),
            children,
        })
    }

    /// Get a reference to this node's ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get a reference to this node's text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get a reference to this node's children
    pub fn children(&self) -> &[TreeNode] {
        &self.children
    }

    fn ismatch(haystack: &str, needle: &str) -> bool {
        if needle.is_empty() {
            return true;
        }
        let matcher = SkimMatcherV2::default();
        matcher.fuzzy_match(haystack, needle).is_some()
    }

    /// Filter this node and its children based on a search query
    /// Returns None if neither this node nor any children match
    pub fn filter(&self, query: &str) -> Option<TreeNode> {
        let i_match = Self::ismatch(&self.text, query);

        let matching_children: Vec<_> = self
            .children
            .iter()
            .filter_map(|child| child.filter(query))
            .collect();

        if i_match || !matching_children.is_empty() {
            Some(TreeNode {
                id: self.id.clone(),
                text: self.text.clone(),
                children: matching_children,
            })
        } else {
            None
        }
    }

    /// Convert this TreeNode into a tui_tree_widget::TreeItem
    pub fn into_tree_item(&self) -> tui_tree_widget::TreeItem<String> {
        let children: Vec<_> = self
            .children
            .iter()
            .map(|child| child.into_tree_item())
            .collect();

        tui_tree_widget::TreeItem::new(self.id.clone(), self.text.clone(), children)
            .expect("Already checked for duplicate IDs")
    }

    pub fn contains_path(&self, path: &[String]) -> bool {
        if path.is_empty() {
            return true;
        }

        self.children
            .iter()
            .any(|child| child.id == path[0] && child.contains_path(&path[1..]))
    }
}
