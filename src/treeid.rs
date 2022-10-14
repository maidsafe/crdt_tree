// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

use std::hash::Hash;

/// `TreeId` trait. `TreeId` are unique identifiers for each node in a tree.
pub trait TreeId: Eq + Clone + Hash {}
impl<ID: Eq + Clone + Hash> TreeId for ID {}
