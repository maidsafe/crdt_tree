// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Implements LogOpMove, a log entry used by State
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

use super::{Clock, OpMove, TreeId, TreeMeta, TreeNode};
use crdts::Actor;

/// From the paper:
/// ----
/// In order to correctly apply move operations, a replica needs
/// to maintain not only the current state of the tree, but also
/// an operation log.  The log is a list of LogMove records in
/// descending timestamp order.  LogMove t oldp p m c is similar
/// to Move t p m c; the difference is that LogMove has an additional
/// field oldp of type ('n x 'm) option.  This option type means
/// the field can either take the value None or a pair of a node ID
/// and a metadata field.
///
/// When a replica applies a Move operation to its tree it
/// also records a corresponding LogMove operation in its log.
/// The t, p, m, and c fields are taken directly from the Move
/// record while the oldp field is filled in based on the
/// state of the tree before the move.  If c did not exist
/// in the tree, oldp is set to None. Else oldp records the
/// previous parent metadata of c: if there exist p' and m'
/// such that (p', m', c') E tree, then oldp is set to Some(p', m').
/// The get_parent() function implements this.
/// ----
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogOpMove<ID: TreeId, TM: TreeMeta, A: Actor> {
    // an operation that is being logged.
    op: OpMove<ID, TM, A>,

    /// parent and metadata prior to application of op.
    /// None if op.child_id did not previously exist in tree.
    oldp: Option<TreeNode<ID, TM>>,
}

impl<ID: TreeId, TM: TreeMeta, A: Actor> LogOpMove<ID, TM, A> {
    /// create a new instance of LogOpMove
    pub fn new(op: OpMove<ID, TM, A>, oldp: Option<TreeNode<ID, TM>>) -> LogOpMove<ID, TM, A> {
        LogOpMove { op, oldp }
    }

    /// returns timestamp reference
    pub fn timestamp(&self) -> &Clock<A> {
        self.op.timestamp()
    }

    /// returns parent_id reference
    pub fn parent_id(&self) -> &ID {
        self.op.parent_id()
    }

    /// returns metadata reference
    pub fn metadata(&self) -> &TM {
        self.op.metadata()
    }

    /// returns child_id reference
    pub fn child_id(&self) -> &ID {
        &self.op.child_id()
    }

    /// returns oldp reference
    pub fn oldp(&self) -> &Option<TreeNode<ID, TM>> {
        &self.oldp
    }

    /// converts LogOpMove into an OpMove
    pub fn op_into(self) -> OpMove<ID, TM, A> {
        self.op
    }
}
