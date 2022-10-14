// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.

/// tests for crdt-tree
use crdt_tree::{Clock, OpMove, State};

// Define some "real" types for use in the tests.
type TypeId = u8;
type TypeActor = u8;
type TypeMetaStr<'a> = &'a str;

// helper: generate a new random id
fn new_id() -> TypeId {
    rand::random::<TypeId>()
}

// helper: generate a new random actor
fn new_actor() -> TypeActor {
    rand::random::<TypeActor>()
}

// Tests case 1 in the paper.  Concurrent moves of the same node.
//
// Initial State:
// root
//  - A
//  - B
//  - C
//
// Replica 1 moves A to be a child of B, while concurrently
// replica 2 moves the same node A to be a child of C.
// a child of B.  This could potentially result in A being
// duplicated under B and C, or A having 2 parents, B and C.
//
// The only valid result is for one operation
// to succeed and the other to be ignored, but both replica's
// must pick the same success case.
//
// See paper for diagram.
#[test]
fn concurrent_moves_lww() {
    let mut r1: State<TypeId, TypeMetaStr, TypeActor> = State::new();
    let mut r2: State<TypeId, TypeMetaStr, TypeActor> = State::new();

    let (r1_id, r2_id) = (new_actor(), new_actor());
    let mut r1t = Clock::<TypeActor>::new(r1_id, None);
    let mut r2t = Clock::<TypeActor>::new(r2_id, None);

    let (root_id, a_id, b_id, c_id) = (new_id(), new_id(), new_id(), new_id());

    // Create ops for initial tree state.
    let ops = vec![
        OpMove::new(r1t.tick(), 0, "root", root_id),
        OpMove::new(r1t.tick(), root_id, "a", a_id),
        OpMove::new(r1t.tick(), root_id, "b", b_id),
        OpMove::new(r1t.tick(), root_id, "c", c_id),
    ];

    // Apply initial ops to both replicas
    for op in ops {
        r1.apply_op(op.clone());
        r2.apply_op(op);
    }

    // replica_1 moves /root/a to /root/b
    let r1_op = OpMove::new(r1t.tick(), b_id, "a", a_id);
    // replica_2 "simultaneously" moves /root/a to /root/c
    let r2_op = OpMove::new(r2t.tick(), c_id, "a", a_id);

    // apply both ops to r1
    r1.apply_op(r1_op.clone());
    r1.apply_op(r2_op.clone());

    // apply both ops to r2
    r2.apply_op(r2_op);
    r2.apply_op(r1_op);

    assert_eq!(r1, r2);
}

// Tests case 2 in the paper.  Moving a node to be a descendant of itself.
//
// Initial State:
// root
//  - A
//    - C
//  - B
//
// Initially, nodes A and B are siblings.  Replica 1 moves B
// to be a child of A, while concurrently replica 2 moves A to be
// a child of B.  This could potentially result in a cyle, or
// duplication.  The only valid result is for one operation
// to succeed and the other to be ignored, but both replica's
// must pick the same success case.
//
// See paper for diagram.
#[test]
fn concurrent_moves_cycle() {
    let mut r1: State<TypeId, TypeMetaStr, TypeActor> = State::new();
    let mut r2: State<TypeId, TypeMetaStr, TypeActor> = State::new();

    let (r1_id, r2_id) = (new_actor(), new_actor());
    let mut r1t = Clock::<TypeActor>::new(r1_id, None);
    let mut r2t = Clock::<TypeActor>::new(r2_id, None);

    let (root_id, a_id, b_id, c_id) = (new_id(), new_id(), new_id(), new_id());

    // Create ops for initial tree state.
    let ops = vec![
        OpMove::new(r1t.tick(), 0, "root", root_id),
        OpMove::new(r1t.tick(), root_id, "a", a_id),
        OpMove::new(r1t.tick(), root_id, "b", b_id),
        OpMove::new(r1t.tick(), a_id, "c", c_id),
    ];

    // Apply initial ops to both replicas
    for op in ops {
        r1.apply_op(op.clone());
        r2.apply_op(op);
    }

    // replica_1 moves /root/b to /root/a
    let r1_op = OpMove::new(r1t.tick(), a_id, "b", b_id);
    // replica_2 "simultaneously" moves /root/a to /root/b
    let r2_op = OpMove::new(r2t.tick(), b_id, "a", a_id);

    // apply both ops to r1
    r1.apply_op(r1_op.clone());
    r1.apply_op(r2_op.clone());

    // apply both ops to r2
    r2.apply_op(r2_op);
    r2.apply_op(r1_op);

    assert_eq!(r1, r2);
}
