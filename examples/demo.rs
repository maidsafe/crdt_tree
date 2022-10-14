// Copyright (c) 2022, MaidSafe.
// All rights reserved.
//
// This SAFE Network Software is licensed under the BSD-3-Clause license.
// Please see the LICENSE file for more details.
extern crate crdts;

use crdt_tree::{OpMove, Tree, TreeId, TreeMeta, TreeReplica};
use crdts::Actor;
use rand::Rng;
use std::collections::HashMap;
use std::env;

// define some concrete types to instantiate our Tree data structures with.
type TypeId = u64;
type TypeMeta<'a> = &'static str;
type TypeActor = u64;

// A simple main func to kickoff a demo or print-help.
fn main() {
    let args: Vec<String> = env::args().collect();

    let demo = if args.len() > 1 { &args[1] } else { "" };

    match demo {
        "demo_concurrent_moves" => demo_concurrent_moves(),
        "demo_concurrent_moves_cycle" => demo_concurrent_moves_cycle(),
        "demo_truncate_log" => demo_truncate_log(),
        "demo_walk_deep_tree" => demo_walk_deep_tree(),
        "demo_move_to_trash" => demo_move_to_trash(),

        _ => print_help(),
    }
}

// Demo: Concurrent moves test from the paper.
// See paper for diagram.
//
// Tests what happens when two peers move the same tree node to a different
// location at the same time.  Upon applying eachother's ops, they must converge
// to a common location.
fn demo_concurrent_moves() {
    let mut r1: TreeReplica<TypeId, TypeMeta, TypeActor> = TreeReplica::new(new_id());
    let mut r2: TreeReplica<TypeId, TypeMeta, TypeActor> = TreeReplica::new(new_id());

    let ids: HashMap<&str, TypeId> = [
        ("root", new_id()),
        ("a", new_id()),
        ("b", new_id()),
        ("c", new_id()),
    ]
    .iter()
    .cloned()
    .collect();

    // Setup initial tree state.
    let ops = r1.opmoves(vec![
        (0, "root", ids["root"]),
        (ids["root"], "a", ids["a"]),
        (ids["root"], "b", ids["b"]),
        (ids["root"], "c", ids["c"]),
    ]);

    r1.apply_ops_byref(&ops);
    r2.apply_ops_byref(&ops);

    println!("Initial tree state on both replicas");
    print_tree(r1.tree(), &ids["root"]);

    // replica_1 moves /root/a to /root/b
    let repl1_ops = vec![r1.opmove(ids["b"], "a", ids["a"])];

    // replica_2 "simultaneously" moves /root/a to /root/c
    let repl2_ops = vec![r2.opmove(ids["c"], "a", ids["a"])];

    // replica_1 applies his op, then merges op from replica_2
    r1.apply_ops_byref(&repl1_ops);
    println!("\nreplica_1 tree after move");
    print_tree(r1.tree(), &ids["root"]);
    r1.apply_ops_byref(&repl2_ops);

    // replica_2 applies his op, then merges op from replica_1
    r2.apply_ops_byref(&repl2_ops);
    println!("\nreplica_2 tree after move");
    print_tree(r2.tree(), &ids["root"]);
    r2.apply_ops_byref(&repl1_ops);

    // expected result: state is the same on both replicas
    // and final path is /root/c/a because last-writer-wins
    // and replica_2's op has a later timestamp.
    //    if r1.state.is_equal(&r2.state) {
    if r1.state() == r2.state() {
        println!("\nreplica_1 state matches replica_2 state after each merges other's change.  conflict resolved!");
        print_replica_trees(&r1, &r2, &ids["root"]);
    } else {
        println!("\nwarning: replica_1 state does not match replica_2 state after merge");
        print_replica_trees(&r1, &r2, &ids["root"]);
        println!("-- replica_1 state --");
        println!("{:#?}", r1.state());
        println!("\n-- replica_2 state --");
        println!("{:#?}", r2.state());
    }
}

// Demo: cycle test from the paper
//
// Tests what happen when two peers independently perform operations that would
// introduce a cycle when combined.
//
// Upon applying eachother's ops, they must converge to a common location without
// any cycles.
fn demo_concurrent_moves_cycle() {
    let mut r1: TreeReplica<TypeId, TypeMeta, TypeActor> = TreeReplica::new(new_id());
    let mut r2: TreeReplica<TypeId, TypeMeta, TypeActor> = TreeReplica::new(new_id());

    let ids: HashMap<&str, TypeId> = [
        ("root", new_id()),
        ("a", new_id()),
        ("b", new_id()),
        ("c", new_id()),
    ]
    .iter()
    .cloned()
    .collect();

    // Setup initial tree state.
    let ops = r1.opmoves(vec![
        (0, "root", ids["root"]),
        (ids["root"], "a", ids["a"]),
        (ids["root"], "b", ids["b"]),
        (ids["a"], "c", ids["c"]),
    ]);

    r1.apply_ops_byref(&ops);
    r2.apply_ops_byref(&ops);

    println!("Initial tree state on both replicas");
    print_tree(r1.tree(), &ids["root"]);

    // replica_1 moves /root/b to /root/a,  creating /root/a/b
    let repl1_ops = r1.opmoves(vec![(ids["a"], "b", ids["b"])]);

    // replica_2 "simultaneously" moves /root/a to /root/b, creating /root/b/a
    let repl2_ops = r2.opmoves(vec![(ids["b"], "a", ids["a"])]);

    // replica_1 applies his op, then merges op from replica_2
    r1.apply_ops_byref(&repl1_ops);
    println!("\nreplica_1 tree after move");
    print_tree(r1.tree(), &ids["root"]);
    r1.apply_ops_byref(&repl2_ops);

    // replica_2 applies his op, then merges op from replica_1
    r2.apply_ops_byref(&repl2_ops);
    println!("\nreplica_2 tree after move");
    print_tree(r2.tree(), &ids["root"]);
    r2.apply_ops_byref(&repl1_ops);

    // expected result: state is the same on both replicas
    // and final path is /root/b/a because last-writer-wins
    // and replica_2's op has a later timestamp.
    if r1.state() == r2.state() {
        println!("\nreplica_1 state matches replica_2 state after each merges other's change.  conflict resolved!");
        print_replica_trees(&r1, &r2, &ids["root"]);
    } else {
        println!("\nwarning: replica_1 state does not match replica_2 state after merge");
        print_replica_trees(&r1, &r2, &ids["root"]);
        println!("-- replica_1 state --");
        println!("{:#?}", r1.state());
        println!("\n-- replica_2 state --");
        println!("{:#?}", r2.state());
    }
}

// Demo: Walk a deep tree
//
// This demonstrates creation of a deep tree which we then walk in depth-first
// fashion.
//
// This particular tree contains 2^6-1 nodes and is up to 6 levels deep.
fn demo_walk_deep_tree() {
    let mut r1: TreeReplica<TypeId, TypeMeta, TypeActor> = TreeReplica::new(new_id());

    let ids: HashMap<&str, TypeId> = [("root", new_id())].iter().cloned().collect();

    // Generate initial tree state.
    println!("generating ops...");
    let mut ops = vec![(0, "root", ids["root"])];
    mktree_ops(&mut ops, &mut r1, ids["root"], 2, 6); //  <-- max 6 levels deep.

    println!("applying ops...");
    let ops_len = ops.len();
    r1.apply_ops_byref(&r1.opmoves(ops));

    println!("walking tree...");
    r1.tree().walk(&ids["root"], |tree, node_id, depth| {
        if true {
            let meta = match tree.find(node_id) {
                Some(tn) => format!("{:?}", tn.metadata()),
                None => format!("{:?}", node_id),
            };
            println!("{:indent$}{}", "", meta, indent = depth);
        }
    });

    println!("\nnodes in tree: {}", ops_len);
}

/// Demonstrates log truncation
///
/// This requires that causally stable threshold tracking is enabled in `TreeReplica`
fn demo_truncate_log() {
    let mut replicas: Vec<TreeReplica<TypeId, TypeMeta, TypeActor>> = Vec::new();
    let num_replicas = 5;

    // start some replicas.
    for _i in 0..num_replicas {
        // pass true flag to enable causally stable threshold tracking
        let r: TreeReplica<TypeId, TypeMeta, TypeActor> = TreeReplica::new(new_id());
        replicas.push(r);
    }

    let root_id = new_id();

    // Generate initial tree state.
    let mut opmoves = vec![replicas[0].opmove(0, "root", root_id)];

    println!("generating move operations...");

    // generate some initial ops from all replicas.
    for r in replicas.iter_mut() {
        let finaldepth = rand::thread_rng().gen_range(3, 6);
        let mut ops = vec![];
        mktree_ops(&mut ops, r, root_id, 2, finaldepth);
        opmoves.extend(r.opmoves(ops));
    }

    // apply all ops to all replicas
    println!(
        "applying {} operations to all {} replicas...\n",
        opmoves.len(),
        replicas.len()
    );
    apply_ops_to_replicas(&mut replicas, &opmoves);

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Stat {
        pub replica: TypeActor,
        pub ops_before_truncate: usize,
        pub ops_after_truncate: usize,
    }

    let mut stats: Vec<Stat> = Vec::new();
    for r in replicas.iter_mut() {
        println!("truncating log of replica {}...", r.id());
        println!(
            "causally stable threshold: {:?}\n",
            r.causally_stable_threshold()
        );
        let ops_b4 = r.state().log().len();
        r.truncate_log();
        let ops_after = r.state().log().len();
        stats.push(Stat {
            replica: *r.id(),
            ops_before_truncate: ops_b4,
            ops_after_truncate: ops_after,
        });
    }

    println!("-- Stats -- ");
    println!("\n{:#?}", stats);
}

/// Demonstrates moving items to a Trash node outside the nominal root and then
/// emptying the trash after the log is truncated.
///
/// This requires that causally stable threshold tracking is enabled in `TreeReplica`
fn demo_move_to_trash() {
    // pass true flag to enable causally stable threshold tracking
    let mut r1: TreeReplica<TypeId, TypeMeta, TypeActor> = TreeReplica::new(new_id());
    let mut r2: TreeReplica<TypeId, TypeMeta, TypeActor> = TreeReplica::new(new_id());

    let ids: HashMap<&str, TypeId> = [
        ("forest", new_id()),
        ("trash", new_id()),
        ("root", new_id()),
        ("home", new_id()),
        ("bob", new_id()),
        ("project", new_id()),
    ]
    .iter()
    .cloned()
    .collect();

    // Generate initial tree state.
    //
    // - forest
    //   - trash
    //   - root
    //     - home
    //       - bob
    //         - project
    let mut ops = vec![
        (ids["forest"], "root", ids["root"]),
        (ids["forest"], "trash", ids["trash"]),
        (ids["root"], "home", ids["home"]),
        (ids["home"], "bob", ids["bob"]),
        (ids["bob"], "project", ids["project"]),
    ];

    // add some nodes under project
    mktree_ops(&mut ops, &mut r1, ids["project"], 2, 3);
    let opmoves = r1.opmoves(ops);
    r1.apply_ops_byref(&opmoves);
    r2.apply_ops_byref(&opmoves);

    println!("Initial tree");
    print_tree(r1.tree(), &ids["forest"]);

    // move project to trash
    let ops = vec![r1.opmove(ids["trash"], "project", ids["project"])];
    r1.apply_ops_byref(&ops);
    r2.apply_ops_byref(&ops);

    println!("\nAfter project moved to trash (deleted) on both replicas");
    print_tree(r1.tree(), &ids["forest"]);

    // Initially, trashed nodes must be retained because a concurrent move
    // operation may move them back out of the trash.
    //
    // Once the operation that moved a node to the trash is causally
    // stable, we know that no future operations will refer to this node,
    // and so the trashed node and its descendants can be discarded.
    //
    // note:  change r1.opmoves() to r2.opmoves() above to
    //        make the causally stable threshold less than the trash operation
    //        timestamp, which will cause this test to fail, ie hit the
    //        "trash should not be emptied" condition.
    let result = r2.causally_stable_threshold();
    match result {
        Some(cst) if cst < ops[0].timestamp() => {
            println!(
                "\ncausally stable threshold:\n{:#?}\n\ntrash operation:\n{:#?}",
                cst,
                ops[0].timestamp()
            );
            panic!("!error: causally stable threshold is less than trash operation timestamp");
        }
        None => panic!("!error: causally stable threshold not found"),
        _ => {}
    }

    // empty trash
    r1.tree_mut().rm_subtree(&ids["trash"], false);
    println!("\nDelete op is now causally stable, so we can empty trash:");
    print_tree(r1.tree(), &ids["forest"]);
}

fn print_help() {
    let buf = "
Usage: tree <demo>

<demo> can be any of:
  demo_concurrent_moves
  demo_concurrent_moves_cycle
  demo_truncate_log
  demo_walk_deep_tree
  demo_move_to_trash

";
    println!("{}", buf);
}

// Returns op tuples representing a depth-first tree,
// with 2 children for each parent.
fn mktree_ops(
    ops: &mut Vec<(TypeId, TypeMeta, TypeActor)>,
    r: &mut TreeReplica<TypeId, TypeMeta, TypeActor>,
    parent_id: u64,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth {
        return;
    }

    for i in 0..2 {
        let name = if i == 0 { "a" } else { "b" };
        let child_id = new_id();
        ops.push((parent_id, name, child_id));
        mktree_ops(ops, r, child_id, depth + 1, max_depth);
    }
}

// applies each operation in ops to each replica in replicas.
fn apply_ops_to_replicas<ID, TM, A>(
    replicas: &mut [TreeReplica<ID, TM, A>],
    ops: &[OpMove<ID, TM, A>],
) where
    ID: TreeId,
    A: Actor + std::fmt::Debug,
    TM: TreeMeta,
{
    for r in replicas.iter_mut() {
        r.apply_ops_byref(ops);
    }
}

// note: in practice a UUID (at least 128 bits should be used)
fn new_id() -> TypeId {
    rand::random::<TypeId>()
}

// print a treenode, recursively
fn print_treenode<ID, TM>(tree: &Tree<ID, TM>, node_id: &ID, depth: usize, with_id: bool)
where
    ID: TreeId + std::fmt::Debug,
    TM: TreeMeta + std::fmt::Debug,
{
    let result = tree.find(node_id);
    let meta = match result {
        Some(tn) => format!("{:?}", tn.metadata()),
        None if depth == 0 => "forest".to_string(),
        None => {
            panic!("tree node {:?} not found", node_id);
        }
    };
    println!("{:indent$}{}", "", meta, indent = depth * 2);

    for c in tree.children(node_id) {
        print_treenode(tree, &c, depth + 1, with_id);
    }
}

// print a tree.
fn print_tree<ID, TM>(tree: &Tree<ID, TM>, root: &ID)
where
    ID: TreeId + std::fmt::Debug,
    TM: TreeMeta + std::fmt::Debug,
{
    print_treenode(tree, root, 0, false);
}

// print trees for two replicas
fn print_replica_trees<ID, TM, A>(
    repl1: &TreeReplica<ID, TM, A>,
    repl2: &TreeReplica<ID, TM, A>,
    root: &ID,
) where
    ID: TreeId + std::fmt::Debug,
    A: Actor + std::fmt::Debug,
    TM: TreeMeta + std::fmt::Debug,
{
    println!("\n--replica_1 --");
    print_tree(repl1.tree(), root);
    println!("\n--replica_2 --");
    print_tree(repl2.tree(), root);
    println!();
}
