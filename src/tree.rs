use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct TreeNode<IdT>
where
    IdT: Eq + Hash + Clone,
{
    id: IdT,
    text: String,
    children: Vec<TreeNode<IdT>>,
    recursive_num_children: usize,
    matching_indices: Vec<usize>,
}

impl<IdT> TreeNode<IdT>
where
    IdT: Eq + Hash + Clone,
{
    pub fn new(id: impl Into<IdT>, text: impl Into<String>, children: Vec<TreeNode<IdT>>) -> Self {
        Self::new_with_indices(id, text, children, vec![])
    }

    fn new_with_indices(
        id: impl Into<IdT>,
        text: impl Into<String>,
        children: Vec<TreeNode<IdT>>,
        indices: Vec<usize>,
    ) -> Self {
        let recursive_num_children: usize = children
            .iter()
            .map(|child| child.recursive_num_children)
            .sum::<usize>()
            + children.len();

        Self {
            id: id.into(),
            text: text.into(),
            children,
            recursive_num_children,
            matching_indices: indices,
        }
    }

    /// Get a reference to this node's ID
    pub fn id(&self) -> IdT {
        self.id.clone()
    }

    /// Get a reference to this node's text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get a reference to this node's children
    pub fn children(&self) -> &[TreeNode<IdT>] {
        &self.children
    }

    /// Get a reference to this node's recursive number of children
    pub fn recursive_num_children(&self) -> usize {
        self.recursive_num_children
    }

    /// Get a reference to this node's matching indices
    pub fn matching_indices(&self) -> &Vec<usize> {
        &self.matching_indices
    }

    fn ismatch(haystack: &str, needle: &str) -> Option<Vec<usize>> {
        let matcher = SkimMatcherV2::default();
        matcher
            .fuzzy_indices(haystack, needle)
            .map(|(_, indices)| indices)
    }

    /// Filter this node and its children based on a search query
    /// Returns None if neither this node nor any children match
    pub fn filter(&self, query: &str) -> Option<TreeNode<IdT>> {
        let indices = Self::ismatch(&self.text, query);
        let i_match = indices.is_some();

        let matching_children: Vec<_> = self
            .children
            .iter()
            .filter_map(|child| child.filter(query))
            .collect();

        if i_match || !matching_children.is_empty() {
            Some(TreeNode::new_with_indices(
                self.id.clone(),
                self.text.clone(),
                matching_children,
                indices.unwrap_or(vec![]),
            ))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn contains_path(&self, path: &[IdT]) -> bool {
        if path.is_empty() {
            return true;
        }

        self.children
            .iter()
            .any(|child| child.id == path[0] && child.contains_path(&path[1..]))
    }
}
