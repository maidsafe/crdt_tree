// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};

use super::{Clock, OpMove, TreeId, TreeMeta, TreeNode};
use crdts::Actor;

/// Implements `LogOpMove`, a log entry used by `State`
///
/// From the paper[1]:
/// ----
/// In order to correctly apply move operations, a replica needs
/// to maintain not only the current state of the tree, but also
/// an operation log.  The log is a list of `LogMove` records in
/// descending timestamp order.  `LogMove` t oldp p m c is similar
/// to Move t p m c; the difference is that `LogMove` has an additional
/// field oldp of type ('n x 'm) option.  This option type means
/// the field can either take the value None or a pair of a node ID
/// and a metadata field.
///
/// When a replica applies a `Move` operation to its tree it
/// also records a corresponding LogMove operation in its log.
/// The t, p, m, and c fields are taken directly from the Move
/// record while the oldp field is filled in based on the
/// state of the tree before the move.  If c did not exist
/// in the tree, oldp is set to None. Else oldp records the
/// previous parent metadata of c: if there exist p' and m'
/// such that (p', m', c') E tree, then `oldp` is set to `Some(p', m')`.
/// The `get_parent()` function implements this.
/// ----
/// [1] <https://martin.kleppmann.com/papers/move-op.pdf>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogOpMove<ID: TreeId, TM: TreeMeta, A: Actor> {
    // an operation that is being logged.
    op: OpMove<ID, TM, A>,

    /// parent and metadata prior to application of op.
    /// None if `op.child_id` did not previously exist in tree.
    oldp: Option<TreeNode<ID, TM>>,
}

impl<ID: TreeId, TM: TreeMeta, A: Actor> LogOpMove<ID, TM, A> {
    /// create a new instance of `LogOpMove`
    pub fn new(op: OpMove<ID, TM, A>, oldp: Option<TreeNode<ID, TM>>) -> LogOpMove<ID, TM, A> {
        LogOpMove { op, oldp }
    }

    /// returns `timestamp` reference
    #[inline]
    pub fn timestamp(&self) -> &Clock<A> {
        self.op.timestamp()
    }

    /// returns `parent_id` reference
    #[inline]
    pub fn parent_id(&self) -> &ID {
        self.op.parent_id()
    }

    /// returns `metadata` reference
    #[inline]
    pub fn metadata(&self) -> &TM {
        self.op.metadata()
    }

    /// returns `child_id` reference
    #[inline]
    pub fn child_id(&self) -> &ID {
        self.op.child_id()
    }

    /// returns oldp reference
    #[inline]
    pub fn oldp(&self) -> &Option<TreeNode<ID, TM>> {
        &self.oldp
    }

    /// converts `LogOpMove` into an `OpMove`
    #[inline]
    pub fn op_into(self) -> OpMove<ID, TM, A> {
        self.op
    }
}
