use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use std::collections::HashSet;

pub type NodeId = String;

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

    /// Get a reference to this node's text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get a reference to this node's children
    pub fn children(&self) -> &[TreeNode] {
        &self.children
    }

    /// Get a reference to this node's recursive number of children
    pub fn recursive_num_children(&self) -> usize {
        self.recursive_num_children
    }

    /// Get a reference to this node's matching indices
    pub fn matching_indices(&self) -> Option<&Vec<usize>> {
        self.matching_indices.as_ref()
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

    pub fn contains_path(&self, path: &[NodeId]) -> bool {
        if path.is_empty() {
            return true;
        }

        self.children
            .iter()
            .any(|child| child.id == path[0] && child.contains_path(&path[1..]))
    }
}
