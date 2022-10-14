// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

/// `TreeMeta` represent the app-defined data that an application stores in each node
/// of the tree.
pub trait TreeMeta: Clone {}
impl<TM: Clone> TreeMeta for TM {}
