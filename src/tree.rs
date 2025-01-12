use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use log::*;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode<IdT>
where
    IdT: Eq + Hash + Clone + std::fmt::Debug,
{
    id: IdT,
    text: String,
    children: Vec<TreeNode<IdT>>,
    recursive_num_children: usize,
    matching_indices: Vec<usize>,
    pub is_direct_match: bool,
}

impl<IdT> TreeNode<IdT>
where
    IdT: Eq + Hash + Clone + std::fmt::Debug,
{
    pub fn new(id: impl Into<IdT>, text: impl Into<String>, children: Vec<TreeNode<IdT>>) -> Self {
        Self::new_with_indices(id, text, children, vec![], true)
    }

    pub fn new_with_indices(
        id: impl Into<IdT>,
        text: impl Into<String>,
        children: Vec<TreeNode<IdT>>,
        indices: Vec<usize>,
        is_direct_match: bool,
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
            is_direct_match,
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
                i_match,
            ))
        } else {
            None
        }
    }

    pub fn path_to_first_match(&self) -> Vec<IdT> {
        fn path_to_first_match_helper<IdT: Clone + Eq + Hash + std::fmt::Debug>(
            node: &TreeNode<IdT>,
            path: &mut Vec<IdT>,
        ) -> bool {
            path.push(node.id.clone());
            if node.is_direct_match {
                return true;
            }

            if node
                .children()
                .iter()
                .any(|child| path_to_first_match_helper(child, path))
            {
                return true;
            } else {
                path.pop();
                return false;
            }
        }

        let mut path = vec![];
        let match_found = path_to_first_match_helper(self, &mut path);
        if match_found {
            path
        } else {
            vec![]
        }
    }

    pub fn get_selected_node(&self, path: &[IdT]) -> Option<&TreeNode<IdT>> {
        if path.is_empty() {
            return None;
        } else if self.id == path[0] {
            let path_for_children = &path[1..];
            if path_for_children.is_empty() {
                return Some(self);
            }

            return self
                .children
                .iter()
                .find_map(|child| child.get_selected_node(path_for_children));
        }

        return None;
    }
}

mod tests {
    use super::TreeNode;

    #[test]
    fn test_path_to_first_match() {
        let tree = TreeNode::<i32>::new_with_indices(
            0,
            "root",
            vec![
                TreeNode::new_with_indices(1, "child1", vec![], vec![], true),
                TreeNode::new_with_indices(2, "child2", vec![], vec![], true),
            ],
            vec![],
            false,
        );
        assert_eq!(tree.path_to_first_match(), vec![0, 1]);
    }

    #[test]
    fn test_path_to_first_match_no_match() {
        let tree = TreeNode::<i32>::new_with_indices(0, "root", vec![], vec![], false);
        assert_eq!(tree.path_to_first_match(), vec![]);
    }

    #[test]
    fn test_path_to_first_match_nested() {
        let tree = TreeNode::<i32>::new_with_indices(
            0,
            "root",
            vec![
                TreeNode::new_with_indices(
                    1,
                    "child1",
                    vec![TreeNode::new_with_indices(
                        2,
                        "child2",
                        vec![],
                        vec![],
                        false,
                    )],
                    vec![],
                    false,
                ),
                TreeNode::new_with_indices(3, "child3", vec![], vec![], true),
            ],
            vec![],
            false,
        );
        assert_eq!(tree.path_to_first_match(), vec![0, 3]);
    }

    #[test]
    fn test_get_selected_node() {
        let tree = TreeNode::<i32>::new(
            0,
            "root",
            vec![
                TreeNode::new(1, "child1", vec![]),
                TreeNode::new(2, "child2", vec![]),
            ],
        );
        assert_eq!(tree.get_selected_node(&vec![0]), Some(&tree));
        assert_eq!(tree.get_selected_node(&vec![0, 1]), Some(&tree.children[0]));
        assert_eq!(tree.get_selected_node(&vec![0, 2]), Some(&tree.children[1]));
        assert_eq!(tree.get_selected_node(&vec![0, 1, 5]), None);
        assert_eq!(tree.get_selected_node(&vec![]), None);
        assert_eq!(tree.get_selected_node(&vec![0, 5]), None);
    }
}
