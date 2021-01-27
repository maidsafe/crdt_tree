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
