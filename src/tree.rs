// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Implements Tree, a set of triples representing current tree structure.
//!
//! For usage/examples, see:
//!   examples/tree.rs
//!   test/tree.rs
//!
//! This code aims to be an accurate implementation of the
//! tree crdt described in:
//!
//! "A highly-available move operation for replicated trees
//! and distributed filesystems" [1] by Martin Klepmann, et al.
//!
//! [1] https://martin.kleppmann.com/papers/move-op.pdf
//!
//! For clarity, data structures in this implementation are named
//! the same as in the paper (State, Tree) or close to
//! (OpMove --> Move, LogOpMove --> LogOp).  Some are not explicitly
//! named in the paper, such as TreeId, TreeMeta, TreeNode, Clock.

use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;

use super::{TreeId, TreeMeta, TreeNode};

/// From the paper:
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
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree<ID: TreeId, TM: TreeMeta> {
    triples: HashMap<ID, TreeNode<ID, TM>>, // tree_nodes, indexed by child_id.
    children: HashMap<ID, HashMap<ID, bool>>, // parent_id => [child_id => true].  optimization.
}

impl<ID: TreeId, TM: TreeMeta> Tree<ID, TM> {
    /// create a new Tree instance
    pub fn new() -> Self {
        Self {
            triples: HashMap::<ID, TreeNode<ID, TM>>::new(), // tree_nodes, indexed by child_id.
            children: HashMap::<ID, HashMap<ID, bool>>::new(), // parent_id => [child_id => true].  optimization.
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
            n.insert(child_id.to_owned(), true);
        } else {
            let mut h: HashMap<ID, bool> = HashMap::new();
            h.insert(child_id.to_owned(), true);
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
            list.keys().cloned().collect()
        } else {
            Vec::<ID>::default()
        }
    }

    /// walks tree and calls Fn f for each node.
    /// not used by crdt algo.
    /// 
    /// walk uses a non-recursive algorithm, so calling
    /// it on a deep tree will not cause stack overflow.
    pub fn walk<F>(&self, parent_id: &ID, f: &F)
    where
        F: Fn(&Self, &ID, usize),
    {
        let mut stack: Vec::<ID> = Vec::new();
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

    /// parent | child
    /// --------------
    /// 1        2
    /// 1        3
    /// 3        5
    /// 2        6
    /// 6        8

    ///                  1
    ///               2     3
    ///             6         5
    ///           8
    ///
    /// is 2 ancestor of 8?  yes.
    /// is 2 ancestor of 5?   no.

    /// determines if ancestor_id is an ancestor of node_id in tree.
    /// returns bool
    pub fn is_ancestor(&self, child_id: &ID, ancestor_id: &ID) -> bool {
        let mut target_id = child_id;
        while let Some(n) = self.find(&target_id) {
            if n.parent_id() == ancestor_id {
                return true;
            }
            target_id = n.parent_id();
        }
        false
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
