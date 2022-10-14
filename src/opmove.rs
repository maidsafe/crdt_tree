// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};

use super::{Clock, LogOpMove, TreeId, TreeMeta};
use crdts::quickcheck::{Arbitrary, Gen};
use crdts::Actor;
use std::hash::Hash;

/// Implements `OpMove`, the only way to manipulate tree data.
///
/// `OpMove` are applied via `State`::apply_op() or at a higher
/// level via `TreeReplica`::apply_op()
///
/// From the paper[1]:
/// ----
/// We allow the tree to be updated in three ways: by creating
/// a new child of any parent node, by deleting a node, or by
/// moving a node to be a child of a new parent.  However all
/// three types of update can be represented by a move operation.
/// To create a node, we generate a fresh ID for that node, and
/// issue an operation to move this new ID to be created.  We
/// also designate as "trash" some node ID that does not exist
/// in the tree; then we can delete a node by moving it to be
/// a child of the trash.
///
/// Thus, we define one kind of operation: Move t p m c.  A move
/// operation is a 4-tuple consisting of a timestamp t of type 't,
/// a parent node ID p of type 'n, a metadata field m of type 'm,
/// and a child node ID c of type 'n.  Here, 't, 'n, and 'm are
/// type variables that can be replaced with arbitrary types;
/// we only require that node identifiers 'n are globally unique
/// (eg UUIDs); timestamps 't need to be globally unique and
/// totally ordered (eg Lamport timestamps [11]).
///
/// The meaning of an operation Move t p m c is that at time t,
/// the node with ID c is moved to be a child of the parent node
/// with ID p.  The operation does not specify the old location
/// of c; the algorithm simply removes c from wherever it is
/// currently located in the tree, and moves it to p.  If c
/// does not currently exist in the tree, it is created as a child
/// of p.
///
/// The metadata field m in a move operation allows additional
/// information to be associated with the parent-child relationship
/// of p and c.  For example, in a filesystem, the parent and
/// child are the inodes of a directory and a file within it
/// respectively, and the metadata contains the filename of the
/// child.  Thus, a file with inode c can be renamed by performing
/// a Move t p m c, where the new parent directory p is the inode
/// of the existing parent (unchanged), but the metadata m contains
/// the new filename.
///
/// When users want to make changes to the tree on their local
/// replica they generate new Move t p m c operations for these
/// changes, and apply these operations using the algorithm
/// described...
/// ----
/// [1] https://martin.kleppmann.com/papers/move-op.pdf
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpMove<ID: TreeId, TM: TreeMeta, A: Actor> {
    /// lamport clock + actor
    timestamp: Clock<A>,
    /// parent identifier
    parent_id: ID,
    /// metadata.  can be anything.
    metadata: TM,
    /// child identifier
    child_id: ID,
}

impl<ID: TreeId, TM: TreeMeta, A: Actor> OpMove<ID, TM, A> {
    /// create a new OpMove instance
    #[inline]
    pub fn new(timestamp: Clock<A>, parent_id: ID, metadata: TM, child_id: ID) -> Self {
        Self {
            timestamp,
            parent_id,
            metadata,
            child_id,
        }
    }

    /// returns timestamp reference
    #[inline]
    pub fn timestamp(&self) -> &Clock<A> {
        &self.timestamp
    }

    /// returns `parent_id` reference
    #[inline]
    pub fn parent_id(&self) -> &ID {
        &self.parent_id
    }

    /// returns metadata reference
    #[inline]
    pub fn metadata(&self) -> &TM {
        &self.metadata
    }

    /// returns `child_id` reference
    #[inline]
    pub fn child_id(&self) -> &ID {
        &self.child_id
    }
}

impl<ID: TreeId, A: Actor, TM: TreeMeta> From<LogOpMove<ID, TM, A>> for OpMove<ID, TM, A> {
    /// creates `OpMove` from a `LogOpMove`
    fn from(l: LogOpMove<ID, TM, A>) -> Self {
        l.op_into()
    }
}

// For testing with quicktest
impl<ID: TreeId + Arbitrary, A: Actor + Arbitrary, TM: TreeMeta + Arbitrary> Arbitrary
    for OpMove<ID, TM, A>
{
    /// generates an arbitrary (random) OpMove
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self::new(
            Clock::arbitrary(g),
            ID::arbitrary(g),
            TM::arbitrary(g),
            ID::arbitrary(g),
        )
    }
}
