// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

//! Implements Tree Conflict-Free Replicated Data Type (CRDT).
//!
//! For usage/examples, see:
//!   examples/demo.rs
//!   tests/tree.rs
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

mod tree;
pub use self::tree::Tree;

mod state;
pub use self::state::State;

mod clock;
pub use self::clock::Clock;

mod opmove;
pub use self::opmove::OpMove;

mod logopmove;
pub use self::logopmove::LogOpMove;

mod treeid;
pub use self::treeid::TreeId;

mod treemeta;
pub use self::treemeta::TreeMeta;

mod treenode;
pub use self::treenode::TreeNode;

mod treereplica;
pub use self::treereplica::TreeReplica;
