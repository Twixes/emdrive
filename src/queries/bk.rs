use crate::utils::vec_eq;
use std::{collections::HashMap, hash::Hash};

use super::Distancable;

/// A BK tree search result containing positions found.
#[derive(Debug)]
pub struct SearchResult<'a, P>
where
    P: Eq,
{
    exact: Vec<&'a P>,
    close: Vec<&'a P>,
}

impl<'a, P> SearchResult<'a, P>
where
    P: Eq,
{
    fn new() -> Self {
        SearchResult {
            exact: vec![],
            close: vec![],
        }
    }
}

impl<'a, P> PartialEq for SearchResult<'a, P>
where
    P: Eq + Ord,
{
    fn eq(&self, other: &Self) -> bool {
        vec_eq(&self.exact, &other.exact) && vec_eq(&self.close, &other.close)
    }
}

/// A BK tree, starting from `root`.
#[derive(Debug)]
pub struct Tree<P, D>
where
    P: Distancable<D>,
    D: num::Num + Eq + Ord + Copy + Hash,
{
    root: Option<TreeNode<P, D>>,
}

impl<P, D> Tree<P, D>
where
    P: Distancable<D>,
    D: num::Num + Eq + Ord + Copy + Hash,
{
    pub fn new() -> Self {
        Tree { root: None }
    }

    pub fn add(&mut self, position: P) {
        if let Some(root) = &mut self.root {
            root.add(position);
        } else {
            self.root = Some(TreeNode::new(position));
        }
    }

    pub fn search(&self, position: &P, radius: D) -> SearchResult<P> {
        return if let Some(root) = &self.root {
            root.search(&position, radius)
        } else {
            SearchResult::new()
        };
    }
}

impl<P, D> Default for Tree<P, D>
where
    P: Distancable<D>,
    D: num::Num + Eq + Ord + Copy + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A BK tree node.
#[derive(Debug)]
struct TreeNode<P, D>
where
    P: Distancable<D>,
    D: num::Num + Eq + Ord + Copy + Hash,
{
    position: P,
    children: HashMap<D, TreeNode<P, D>>,
}

impl<P, D> TreeNode<P, D>
where
    P: Distancable<D>,
    D: num::Num + Eq + Ord + Copy + Hash,
{
    fn new(position: P) -> Self {
        TreeNode {
            position,
            children: HashMap::new(),
        }
    }

    fn add(&mut self, position: P) {
        let distance = self.position.distance(&position);
        if let Some(child) = self.children.get_mut(&distance) {
            child.add(position);
        } else {
            self.children.insert(distance, TreeNode::new(position));
        }
    }

    fn search(&self, position: &P, radius: D) -> SearchResult<P> {
        let mut result = SearchResult {
            exact: vec![],
            close: vec![],
        };

        let current_distance = self.position.distance(&position);
        if current_distance == D::zero() {
            result.exact.push(&self.position);
        } else if current_distance <= radius {
            result.close.push(&self.position);
        }

        for child in self.children.values() {
            let mut sub_result = child.search(position, radius);
            result.exact.append(&mut sub_result.exact);
            result.close.append(&mut sub_result.close);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test implementation of Distancable to create an i32-based BK tree (with absolute difference as metric)
    impl Distancable<i32> for i32 {
        fn distance(&self, other: &Self) -> i32 {
            (self - other).abs()
        }
    }

    #[test]
    fn bk_tree_add_and_search_work() {
        let mut tree: Tree<i32, i32> = Tree::new();
        tree.add(2);
        tree.add(1000);
        tree.add(5);
        tree.add(-1);

        assert_eq!(
            tree.search(&3, 1),
            SearchResult {
                exact: vec![],
                close: vec![&2]
            }
        );
        assert_eq!(
            tree.search(&3, 2),
            SearchResult {
                exact: vec![],
                close: vec![&2, &5]
            }
        );
        assert_eq!(
            tree.search(&2, 1),
            SearchResult {
                exact: vec![&2],
                close: vec![]
            }
        );
        assert_eq!(
            tree.search(&999, 1000),
            SearchResult {
                exact: vec![],
                close: vec![&2, &5, &-1, &1000]
            }
        );
    }
}
