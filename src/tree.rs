use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub type NodeId = String;

#[derive(Debug, Clone)]
pub struct TreeNode {
    id: NodeId,
    text: String,
    children: Vec<TreeNode>,
    recursive_num_children: usize,
    matching_indices: Vec<usize>,
}

impl TreeNode {
    pub fn new(id: impl Into<NodeId>, text: impl Into<String>, children: Vec<TreeNode>) -> Self {
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
            matching_indices: vec![],
        }
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
    pub fn filter(&self, query: &str) -> Option<TreeNode> {
        let indices = Self::ismatch(&self.text, query);
        let i_match = indices.is_some();

        let matching_children: Vec<_> = self
            .children
            .iter()
            .filter_map(|child| child.filter(query))
            .collect();

        if i_match || !matching_children.is_empty() {
            let mut node = TreeNode::new(self.id.clone(), self.text.clone(), matching_children);

            node.matching_indices = indices.unwrap_or(vec![]);

            Some(node)
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
