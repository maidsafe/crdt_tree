// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Implements TreeNode, ie a node that is stored in a Tree.
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

use super::{TreeId, TreeMeta};

/// Represents a Node in a Tree.
///
/// Logically, each Node consists of a triple (parent_id, metadata, child_id).
/// However, in this implementation, the child_id is stored as the
/// key in Tree::triples HashMap<ID, TreeNode>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode<ID: TreeId, TM: TreeMeta> {
    parent_id: ID,
    metadata: TM,
}

impl<ID: TreeId, TM: TreeMeta> TreeNode<ID, TM> {
    // parent_id: ID,
    // metadata: TM,
    // note: child_id is stored only as a map key in tree.

    /// creates a new TreeNode instance
    pub fn new(parent_id: ID, metadata: TM) -> Self {
        Self {
            parent_id,
            metadata,
        }
    }

    /// returns parent_id reference
    pub fn parent_id(&self) -> &ID {
        &self.parent_id
    }

    /// returns metadata reference
    pub fn metadata(&self) -> &TM {
        &self.metadata
    }
}
