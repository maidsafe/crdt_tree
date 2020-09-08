// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Implements Tree Conflict-Free Replicated Data Type (CRDT).
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
//! [1] <https://martin.kleppmann.com/papers/move-op.pdf>
//!
//! For clarity, data structures in this implementation are named
//! the same as in the paper (State, Tree) or close to
//! (OpMove --> Move, LogOpMove --> LogOp).  Some are not explicitly
//! named in the paper, such as TreeId, TreeMeta, TreeNode, Clock.
#![deny(missing_docs)]

/// This module contains a Tree.
pub mod tree;

/// This module contains State.
pub mod state;

/// This module contains a Clock.
pub mod clock;

/// This module contains OpMove.
pub mod opmove;

/// This module contains `LogOpMove`.
pub mod logopmove;

/// This module contains `TreeId`.
pub mod treeid;

/// This module contains `TreeMeta`.
pub mod treemeta;

/// This module contains `TreeNode`.
pub mod treenode;

pub use self::{
    clock::Clock, logopmove::LogOpMove, opmove::OpMove, state::State, tree::Tree, treeid::TreeId,
    treemeta::TreeMeta, treenode::TreeNode,
};
