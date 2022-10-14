// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

extern crate crdts;

use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};

use super::{Clock, LogOpMove, OpMove, State, Tree, TreeId, TreeMeta};
use crdts::Actor;
use log::debug;
use std::collections::HashMap;

/// `TreeReplica` holds tree `State` plus lamport timestamp (actor + counter)
///
/// It can optionally keep track of the latest timestamp for each
/// replica which is needed for calculating the causally stable threshold which
/// is in turn needed for log truncation.
///
/// `TreeReplica` is a higher-level interface to the Tree CRDT and is tied to a
/// particular actor/peer.
///
/// `State` is a lower-level interface to the Tree CRDT and is not tied to any
/// actor/peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeReplica<ID: TreeId, TM: TreeMeta, A: Actor> {
    state: State<ID, TM, A>, // Tree state
    time: Clock<A>,          // Lamport Clock for this replica/tree.

    latest_time_by_replica: HashMap<A, Clock<A>>,
}

impl<ID: TreeId, TM: TreeMeta, A: Actor + std::fmt::Debug> TreeReplica<ID, TM, A> {
    /// returns new TreeReplica
    pub fn new(id: A) -> Self {
        Self {
            state: State::new(),
            time: Clock::<A>::new(id, None),
            latest_time_by_replica: HashMap::<A, Clock<A>>::new(),
        }
    }

    /// Generates an OpMove
    ///
    /// Note that OpMove::timestamp is incremented from TreeReplica::time.
    /// TreeReplica::time is not updated until ::apply_op() is called.
    ///
    /// Therefore, multiple ops generated with this method may share the same
    /// timestamp, and only one can be sucessfully applied.
    ///
    /// To generate multiple ops before calling ::apply_op(), use ::opmoves() instead.
    pub fn opmove(&self, parent_id: ID, metadata: TM, child_id: ID) -> OpMove<ID, TM, A> {
        OpMove::new(self.time.inc(), parent_id, metadata, child_id)
    }

    /// Generates a list of OpMove from a list of tuples (child_id, metadata, parent_id)
    ///
    /// Each OpMove::timestamp will be greater than the previous op in the returned list.
    /// Therefore, these operations can be successfully applied via ::apply_op() without
    /// timestamp collision.
    pub fn opmoves(&self, ops: Vec<(ID, TM, ID)>) -> Vec<OpMove<ID, TM, A>> {
        let mut time = self.time.clone();

        let mut opmoves = vec![];

        for op in ops {
            opmoves.push(OpMove::new(time.tick(), op.0, op.1, op.2));
        }
        opmoves
    }

    /// Returns actor ID for this replica
    #[inline]
    pub fn id(&self) -> &A {
        self.time.actor_id()
    }

    /// Returns the latest lamport time seen by this replica
    #[inline]
    pub fn time(&self) -> &Clock<A> {
        &self.time
    }

    /// Returns Tree State reference
    #[inline]
    pub fn state(&self) -> &State<ID, TM, A> {
        &self.state
    }

    /// Returns Tree reference
    #[inline]
    pub fn tree(&self) -> &Tree<ID, TM> {
        self.state.tree()
    }

    /// Returns mutable Tree reference
    ///
    /// Warning: this is dangerous.  Normally the `Tree` should
    /// not be mutated directly.
    ///
    /// See the demo_move_to_trash in examples/demo.rs for a
    /// use-case, only after log truncation has been performed.
    #[inline]
    pub fn tree_mut(&mut self) -> &mut Tree<ID, TM> {
        self.state.tree_mut()
    }

    /// Applies single operation to `State` and updates our time clock
    ///
    /// Also records latest timestamp for each replica if
    /// track_causally_stable_threshold flag is set.
    pub fn apply_op(&mut self, op: OpMove<ID, TM, A>) {
        self.time = self.time.merge(op.timestamp());

        // store latest timestamp for this actor.
        // This is only needed for calculation of causally_stable_threshold.
        let id = op.timestamp().actor_id();
        match self.latest_time_by_replica.get(id) {
            Some(latest) if (op.timestamp() <= latest) => {
                debug!(
                    "Clock not increased, current timestamp {:?}, provided is {:?}, dropping op!",
                    latest,
                    op.timestamp()
                );
            }
            _ => {
                self.latest_time_by_replica
                    .insert(op.timestamp().actor_id().clone(), op.timestamp().clone());
            }
        };

        self.state.apply_op(op);
    }

    /// Applies list of operations
    pub fn apply_ops(&mut self, ops: Vec<OpMove<ID, TM, A>>) {
        for op in ops {
            self.apply_op(op);
        }
    }

    /// Applies list of operations without taking ownership
    pub fn apply_ops_byref(&mut self, ops: &[OpMove<ID, TM, A>]) {
        self.apply_ops(ops.to_vec())
    }

    /// applies op from a log.  useful for log replay.
    pub fn apply_log_op(&mut self, log_op: LogOpMove<ID, TM, A>) {
        self.apply_op(log_op.into());
    }

    /// applies ops from a log.  useful for log replay.
    pub fn apply_log_ops(&mut self, log_ops: Vec<LogOpMove<ID, TM, A>>) {
        for log_op in log_ops {
            self.apply_log_op(log_op);
        }
    }

    /// returns the causally stable threshold
    pub fn causally_stable_threshold(&self) -> Option<&Clock<A>> {
        // The minimum of latest timestamp from each replica
        // is the causally stable threshold.

        let mut v: Vec<&Clock<A>> = self.latest_time_by_replica.values().collect();
        v.sort();
        v.reverse(); // reverse, so last is lowest.
        v.pop()
    }

    /// truncates log
    pub fn truncate_log(&mut self) -> bool {
        let result = self.causally_stable_threshold();
        match result.cloned() {
            Some(t) => self.state.truncate_log_before(&t),
            None => false,
        }
    }
}
