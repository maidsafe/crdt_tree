// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;

use super::{TreeId, TreeMeta, TreeNode};

/// Implements `Tree`, a set of triples representing current tree structure.
///
/// Normally this `Tree` struct should not be instantiated directly.
/// Instead instantiate `State` (lower-level) or `TreeReplica` (higher-level)
/// and invoke operations on them.
///
/// From the paper[1]:
/// ----
/// We can represent the tree as a set of (parent, meta, child)
/// triples, denoted in Isabelle/HOL as (’n × ’m × ’n) set. When
/// we have (p, m, c) ∈ tree, that means c is a child of p in the tree,
/// with associated metadata m. Given a tree, we can construct
/// a new tree’ in which the child c is moved to a new parent p,
/// with associated metadata m, as follows:
///
/// tree’ = {(p’, m’, c’) ∈ tree. c’ != c} ∪ {(p, m, c)}
///
/// That is, we remove any existing parent-child relationship
/// for c from the set tree, and then add {(p, m, c)} to represent
/// the new parent-child relationship.
/// ----
/// [1] https://martin.kleppmann.com/papers/move-op.pdf
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree<ID: TreeId, TM: TreeMeta> {
    triples: HashMap<ID, TreeNode<ID, TM>>, // tree_nodes, indexed by child_id.
    children: HashMap<ID, HashSet<ID>>,     // parent_id => [child_id].  index/optimization.
}

impl<ID: TreeId, TM: TreeMeta> Tree<ID, TM> {
    /// create a new Tree instance
    pub fn new() -> Self {
        Self {
            triples: HashMap::<ID, TreeNode<ID, TM>>::new(), // tree_nodes, indexed by child_id.
            children: HashMap::<ID, HashSet<ID>>::new(), // parent_id => [child_id].  index/optimization.
        }
    }

    /// helper for removing a triple based on child_id
    pub fn rm_child(&mut self, child_id: &ID) {
        let result = self.triples.get(child_id);
        if let Some(t) = result {
            if let Some(map) = self.children.get_mut(t.parent_id()) {
                map.remove(child_id);
                // cleanup parent entry if empty.
                if map.is_empty() {
                    self.children.remove(t.parent_id());
                }
            }
            self.triples.remove(child_id);
        }
    }

    /// removes a subtree.  useful for emptying trash.
    /// not used by crdt algo.
    pub fn rm_subtree(&mut self, parent_id: &ID, include_parent: bool) {
        for c in self.children(parent_id) {
            self.rm_subtree(&c, false);
            self.rm_child(&c);
        }
        if include_parent {
            self.rm_child(parent_id)
        }
    }

    /// adds a node to the tree
    pub fn add_node(&mut self, child_id: ID, tt: TreeNode<ID, TM>) {
        if let Some(n) = self.children.get_mut(tt.parent_id()) {
            n.insert(child_id.to_owned());
        } else {
            let mut h: HashSet<ID> = HashSet::new();
            h.insert(child_id.to_owned());
            self.children.insert(tt.parent_id().to_owned(), h);
        }
        self.triples.insert(child_id, tt);
    }

    /// returns matching node, or None.
    pub fn find(&self, child_id: &ID) -> Option<&TreeNode<ID, TM>> {
        self.triples.get(child_id)
    }

    /// returns children (IDs) of a given parent node.
    /// useful for walking tree.
    /// not used by crdt algo.
    pub fn children(&self, parent_id: &ID) -> Vec<ID> {
        if let Some(list) = self.children.get(parent_id) {
            list.iter().cloned().collect()
        } else {
            Vec::<ID>::default()
        }
    }

    /// walks tree and calls FnMut f for each node.
    /// not used by crdt algo.
    ///
    /// walk uses a non-recursive algorithm, so calling
    /// it on a deep tree will not cause stack overflow.
    pub fn walk<F>(&self, parent_id: &ID, mut f: F)
    where
        F: FnMut(&Self, &ID, usize),
    {
        let mut stack: Vec<ID> = Vec::new();
        stack.push(parent_id.clone());
        while !stack.is_empty() {
            if let Some(next) = stack.pop() {
                f(self, &next, stack.len());
                for child in self.children(&next) {
                    stack.push(child)
                }
            }
        }
    }

    /// returns true if ancestor_id is an ancestor of child_id in tree.
    ///
    /// ```text
    /// parent | child
    /// --------------
    /// 1        2
    /// 1        3
    /// 3        5
    /// 2        6
    /// 6        8
    ///
    ///                  1
    ///               2     3
    ///             6         5
    ///           8
    ///
    /// is 2 ancestor of 8?  yes.
    /// is 2 ancestor of 5?   no.
    /// ```
    pub fn is_ancestor(&self, child_id: &ID, ancestor_id: &ID) -> bool {
        let mut target_id = child_id;
        while let Some(n) = self.find(target_id) {
            if n.parent_id() == ancestor_id {
                return true;
            }
            target_id = n.parent_id();
        }
        false
    }

    /// Total number of nodes (triples) in the tree
    pub fn num_nodes(&self) -> usize {
        self.triples.len()
    }
}

/// Implement `IntoIterator` for `Tree`.  This is useful for
/// walking all Nodes in tree without knowing a starting point.
impl<ID: TreeId, TM: TreeMeta> IntoIterator for Tree<ID, TM> {
    type Item = (ID, TreeNode<ID, TM>);
    type IntoIter = std::collections::hash_map::IntoIter<ID, TreeNode<ID, TM>>;

    fn into_iter(self) -> Self::IntoIter {
        self.triples.into_iter()
    }
}

impl<ID: TreeId + Debug, TM: TreeMeta + Debug> fmt::Display for Tree<ID, TM> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print_tree(f)
    }
}

impl<ID: TreeId + Debug, TM: TreeMeta + Debug> Tree<ID, TM> {
    // print a treenode, recursively
    fn print_treenode(
        &self,
        f: &mut fmt::Formatter<'_>,
        node_id: &ID,
        depth: usize,
    ) -> fmt::Result {
        let findresult = self.find(node_id);
        let meta = match findresult {
            Some(tn) => format!("{:?} [{:?}]", node_id, tn.metadata()),
            None => format!("{:?}", node_id),
        };
        let mut result = writeln!(f, "{:indent$}{}", "", meta, indent = depth * 2);

        for c in self.children(node_id) {
            result = self.print_treenode(f, &c, depth + 1);
            if result.is_err() {
                break;
            }
        }
        result
    }

    // print a tree.
    fn print_tree(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut r: fmt::Result = Ok(());

        let mut seen: HashSet<ID> = Default::default();

        // We iterate through all triples to find the top-level nodes,
        // i.e. those without any parent (or metadata), then print sub-tree
        // for each one.
        // PERF: This is a slow way to find top-level nodes.  We could
        //       consider keeping a list of them as tree is modified
        for treenode in self.triples.values() {
            let p = treenode.parent_id();
            if self.triples.get(p).is_none() && !seen.contains(p) {
                seen.insert(p.clone());
                r = self.print_treenode(f, p, 0);
                if r.is_err() {
                    break;
                }
            }
        }
        r
    }
}
