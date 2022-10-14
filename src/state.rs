// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ordering, PartialEq};

use super::{Clock, LogOpMove, OpMove, Tree, TreeId, TreeMeta, TreeNode};
use crdts::{Actor, CmRDT};
use log::warn;

/// Holds Tree CRDT state and implements the core algorithm.
///
/// `State` is not tied to any actor/peer and should be equal on any
/// two replicas where each has applied the same operations.
///
/// `State` may be instantiated to manipulate a CRDT Tree or
/// alternatively the higher level `TreeReplica` may be used.
///
/// For usage/examples, see:
///   tests/tree.rs
///
/// This code aims to be an accurate implementation of the
/// tree crdt algorithm described in:
///
/// "A highly-available move operation for replicated trees
/// and distributed filesystems" [1] by Martin Klepmann, et al.
///
/// [1] https://martin.kleppmann.com/papers/move-op.pdf
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State<ID: TreeId, TM: TreeMeta, A: Actor> {
    // a list of `LogMove` in descending timestamp order.
    log_op_list: Vec<LogOpMove<ID, TM, A>>,

    // a tree structure, ie a set of (parent, meta, child) triples
    // that represent the current state of the tree.
    tree: Tree<ID, TM>,
}

impl<ID: TreeId, TM: TreeMeta, A: Actor> State<ID, TM, A> {
    /// create a new State
    pub fn new() -> Self {
        Self {
            log_op_list: Vec::<LogOpMove<ID, TM, A>>::default(),
            tree: Tree::<ID, TM>::new(),
        }
    }

    /// returns tree reference
    #[inline]
    pub fn tree(&self) -> &Tree<ID, TM> {
        &self.tree
    }

    /// returns mutable Tree reference
    ///
    /// Warning: this is dangerous.  Normally the `Tree` should
    /// not be mutated directly.
    ///
    /// See the demo_move_to_trash in examples/demo.rs for a
    /// use-case, only after log truncation has been performed.    
    #[inline]
    pub fn tree_mut(&mut self) -> &mut Tree<ID, TM> {
        &mut self.tree
    }

    /// returns log reference
    #[inline]
    pub fn log(&self) -> &Vec<LogOpMove<ID, TM, A>> {
        &self.log_op_list
    }

    /// add_log_entry
    pub fn add_log_entry(&mut self, entry: LogOpMove<ID, TM, A>) {
        // add at beginning of array
        self.log_op_list.insert(0, entry);
    }

    /// removes log entries before a given timestamp.
    /// not part of crdt-tree algo.
    pub fn truncate_log_before(&mut self, timestamp: &Clock<A>) -> bool {
        // newest entries are at start of list, so to find
        // oldest entries we iterate from the end towards start.
        let len = self.log_op_list.len();
        let mut last_idx: usize = len - 1;
        for (i, v) in self.log_op_list.iter().rev().enumerate() {
            if v.timestamp() < timestamp {
                last_idx = len - 1 - i;
            } else {
                break;
            }
        }

        loop {
            let idx = self.log_op_list.len() - 1;
            if idx < last_idx {
                break;
            }
            self.log_op_list.remove(idx);
        }

        last_idx + 1 < len
    }

    /// The do_op function performs the actual work of applying
    /// a move operation.
    ///
    /// This function takes as argument a pair consisting of a
    /// Move operation and the current tree and it returns a pair
    /// consisting of a LogMove operation (which will be added to the log) and
    /// an updated tree.
    pub fn do_op(&mut self, op: OpMove<ID, TM, A>) -> LogOpMove<ID, TM, A> {
        // When a replica applies a `Move` op to its tree, it also records
        // a corresponding `LogMove` op in its log.  The t, p, m, and c
        // fields are taken directly from the `Move` record, while the `oldp`
        // field is filled in based on the state of the tree before the move.
        // If c did not exist in the tree, `oldp` is set to None.  Otherwise
        // `oldp` records the previous parent and metadata of c.
        let oldp = self.tree.find(op.child_id()).cloned();

        // ensures no cycles are introduced.  If the node c
        // is being moved, and c is an ancestor of the new parent
        // newp, then the tree is returned unmodified, ie the operation
        // is ignored.
        // Similarly, the operation is also ignored if c == newp
        if op.child_id() == op.parent_id() || self.tree.is_ancestor(op.parent_id(), op.child_id()) {
            return LogOpMove::new(op, oldp);
        }

        // Otherwise, the tree is updated by removing c from
        // its existing parent, if any, and adding the new
        // parent-child relationship (newp, m, c) to the tree.
        self.tree.rm_child(op.child_id());
        let tt = TreeNode::new(op.parent_id().to_owned(), op.metadata().to_owned());
        self.tree.add_node(op.child_id().to_owned(), tt);
        LogOpMove::new(op, oldp)
    }

    /// undo_op
    pub fn undo_op(&mut self, log: &LogOpMove<ID, TM, A>) {
        self.tree.rm_child(log.child_id());

        if let Some(oldp) = log.oldp() {
            let tn = TreeNode::new(oldp.parent_id().to_owned(), oldp.metadata().to_owned());
            self.tree.add_node(log.child_id().to_owned(), tn);
        }
    }

    /// redo_op uses do_op to perform an operation
    /// again and recomputes the `LogMove` record (which
    /// might have changed due to the effect of the new operation)
    pub fn redo_op(&mut self, log: LogOpMove<ID, TM, A>) {
        let op = OpMove::from(log);
        let logop2 = self.do_op(op);

        self.add_log_entry(logop2);
    }

    /// See general description of apply/undo/redo above.
    ///
    /// The apply_op func takes two arguments:
    /// a `Move` operation to apply and the current replica
    /// state; and it returns the new replica state.
    /// The constrains `t::{linorder} in the type signature
    /// indicates that timestamps `t are instance if linorder
    /// type class, and they can therefore be compared with the
    /// < operator during a linear (or total) order.
    pub fn apply_op(&mut self, op1: OpMove<ID, TM, A>) {
        if self.log_op_list.is_empty() {
            let op2 = self.do_op(op1);
            self.log_op_list = vec![op2];
        } else {
            match op1.timestamp().cmp(self.log_op_list[0].timestamp()) {
                Ordering::Equal => {
                    // This case should never happen in normal operation
                    // because it is requirement/invariant that all
                    // timestamps are unique.  However, uniqueness is not
                    // strictly enforced in this impl.
                    // The crdt paper does not even check for this case.
                    // We just treat it as a no-op.
                    warn!("op with timestamp equal to previous op ignored. (not applied).  Every op must have a unique timestamp.");
                }
                Ordering::Less => {
                    let logop = self.log_op_list.remove(0); // take from beginning of array
                    self.undo_op(&logop);
                    self.apply_op(op1);
                    self.redo_op(logop);
                }
                Ordering::Greater => {
                    let op2 = self.do_op(op1);
                    self.add_log_entry(op2);
                }
            }
        }
    }

    /// applies a list of operations and consume them. (no cloning)
    pub fn apply_ops_into(&mut self, ops: Vec<OpMove<ID, TM, A>>) {
        for op in ops {
            self.apply_op(op);
        }
    }

    /// applies a list of operations reference, cloning each op.
    pub fn apply_ops(&mut self, ops: &[OpMove<ID, TM, A>]) {
        self.apply_ops_into(ops.to_vec())
    }
}

impl<ID: TreeId, A: Actor, TM: TreeMeta> Default for State<ID, TM, A> {
    fn default() -> Self {
        Self::new()
    }
}

// to make clippy happy.
type LogOpList<ID, TM, A> = Vec<LogOpMove<ID, TM, A>>;

impl<ID: TreeId, A: Actor, TM: TreeMeta> From<(Vec<LogOpMove<ID, TM, A>>, Tree<ID, TM>)>
    for State<ID, TM, A>
{
    /// creates State from tuple `(Vec<LogOpMove>, Tree)`
    fn from(e: (LogOpList<ID, TM, A>, Tree<ID, TM>)) -> Self {
        Self {
            log_op_list: e.0,
            tree: e.1,
        }
    }
}

impl<ID: TreeId, TM: TreeMeta, A: Actor> CmRDT for State<ID, TM, A> {
    type Op = OpMove<ID, TM, A>;

    /// Apply an operation to a `State` instance.
    fn apply(&mut self, op: Self::Op) {
        self.apply_op(op);
    }
}

/// Implement `IntoIterator` for `State`.  This is useful for
/// walking all Nodes in a tree without knowing a starting point.
impl<ID: TreeId, TM: TreeMeta, A: Actor> IntoIterator for State<ID, TM, A> {
    type Item = (ID, TreeNode<ID, TM>);
    type IntoIter = std::collections::hash_map::IntoIter<ID, TreeNode<ID, TM>>;

    fn into_iter(self) -> Self::IntoIter {
        self.tree.into_iter()
    }
}

// See <root>/tests/tree.rs for tests
