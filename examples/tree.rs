// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

extern crate crdts;

use crdt_tree::{Clock, OpMove, State, Tree, TreeId, TreeMeta};
use crdts::Actor;
use rand::Rng;
use std::collections::HashMap;
use std::env;
use log::debug;

#[derive(Debug)]
struct Replica<ID: TreeId, TM: TreeMeta, A: Actor> {
    id: A,                   // Actor representing this replica.  (globally unique id).
    state: State<ID, TM, A>, // Tree state
    time: Clock<A>,          // Lamport Clock for this replica/tree.

    // These are both for tracking causally stable threshold.
    //  (needed for truncating logs, emptying trash)
    latest_time_by_replica: HashMap<A, Clock<A>>,
    track_causally_stable_threshold: bool,
}

impl<ID: TreeId, TM: TreeMeta, A: Actor + std::fmt::Debug> Replica<ID, TM, A> {
    pub fn new(id: A) -> Self {
        Self {
            id: id.clone(),
            state: State::new(),
            time: Clock::<A>::new(id, None),
            latest_time_by_replica: HashMap::<A, Clock<A>>::new(),
            track_causally_stable_threshold: false,
        }
    }

    pub fn track_causally_stable_threshold(&mut self, flag: bool) {
        self.track_causally_stable_threshold = flag;
    }
    #[inline]
    pub fn id(&self) -> &A {
        &self.id
    }

    pub fn apply_ops_noref(&mut self, ops: Vec<OpMove<ID, TM, A>>) {
        for op in ops.clone() {
            self.time = self.time.merge(op.timestamp());

            // store latest timestamp for this actor.
            // This is only needed for calculation of
            // causally_stable_threshold.  If that is not
            // required, it needn't execute.
            if self.track_causally_stable_threshold {
                let id = op.timestamp().actor_id();
                match self.latest_time_by_replica.get(id) {
                   Some(latest) if (latest <= op.timestamp()) => {
                                        debug!("Clock not increased, current timestamp {:?}, provided is {:?}, dropping op!", latest, op.timestamp());
                                    }
                    _ => {
                        self.latest_time_by_replica
                            .insert(op.timestamp().actor_id().clone(), op.timestamp().clone());
                    }
                };
            }

            self.state.apply_op(op);
        }
    }

    #[inline]
    pub fn state(&self) -> &State<ID, TM, A> {
        &self.state
    }

    #[inline]
    pub fn tree(&self) -> &Tree<ID, TM> {
        self.state.tree()
    }

    #[inline]
    pub fn tree_mut(&mut self) -> &mut Tree<ID, TM> {
        self.state.tree_mut()
    }

    pub fn apply_ops(&mut self, ops: &Vec<OpMove<ID, TM, A>>) {
        self.apply_ops_noref(ops.clone())
    }

    /*
        // applies ops from a log.  useful for log replay.
        fn apply_log_ops(&mut self, log_ops: &Vec<LogOpMove<TM, A>>) {
            let mut ops: Vec::<OpMove<TM, A>> = Vec::new();
            for log_op in log_ops {
                ops.push(OpMove::from_log_op_move(log_op));
            }
            self.apply_ops(&ops);
        }
    */

    pub fn causally_stable_threshold(&self) -> Option<&Clock<A>> {
        // The minimum of latest timestamp from each replica
        // is the causally stable threshold.

        let mut v: Vec<&Clock<A>> = self.latest_time_by_replica.values().collect();
        v.sort_unstable_by(|a, b| a.cmp(b));
        v.pop() 
    }

    pub fn truncate_log(&mut self) -> bool {
        let result = self.causally_stable_threshold();
        match result.cloned() {
            Some(t) => self.state.truncate_log_before(&t),
            None => false,
        }
    }

    pub fn tick(&mut self) -> Clock<A> {
        self.time.tick()
    }
}

type TypeId = u64;
type TypeMeta<'a> = &'a str;
type TypeActor = u64;

// Returns operations representing a depth-first tree,
// with 2 children for each parent.
fn mktree_ops(
    ops: &mut Vec<OpMove<TypeId, TypeMeta, TypeActor>>,
    r: &mut Replica<TypeId, TypeMeta, TypeActor>,
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
        ops.push(OpMove::new(r.tick(), parent_id, name, child_id));
        mktree_ops(ops, r, child_id, depth + 1, max_depth);
    }
}

fn apply_ops_to_replicas<ID, TM, A>(
    replicas: &mut Vec<Replica<ID, TM, A>>,
    ops: &Vec<OpMove<ID, TM, A>>,
) where
    ID: TreeId,
    A: Actor + std::fmt::Debug,
    TM: TreeMeta,
{
    for r in replicas.iter_mut() {
        r.apply_ops(ops);
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
    let result = tree.find(&node_id);
    let meta = match result {
        Some(tn) => format!("{:?}", tn.metadata()),
        None if depth == 0 => "forest".to_string(),
        None => {
            panic!("tree node {:?} not found", node_id);
        }
    };
    println!("{:indent$}{}", "", meta, indent = depth * 2);

    for c in tree.children(&node_id) {
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

fn print_replica_trees<ID, TM, A>(repl1: &Replica<ID, TM, A>, repl2: &Replica<ID, TM, A>, root: &ID)
where
    ID: TreeId + std::fmt::Debug,
    A: Actor + std::fmt::Debug,
    TM: TreeMeta + std::fmt::Debug,
{
    println!("\n--replica_1 --");
    print_tree(repl1.tree(), root);
    println!("\n--replica_2 --");
    print_tree(repl2.tree(), root);
    println!("");
}

// See paper for diagram.
fn test_concurrent_moves() {
    let mut r1: Replica<TypeId, TypeMeta, TypeActor> = Replica::new(new_id());
    let mut r2: Replica<TypeId, TypeMeta, TypeActor> = Replica::new(new_id());

    let ids: HashMap<&str, TypeId> = [
        ("root", 0),
        ("a", new_id()),
        ("b", new_id()),
        ("c", new_id()),
    ]
    .iter()
    .cloned()
    .collect();

    // Setup initial tree state.
    let ops = vec![
        OpMove::new(r1.tick(), 0, "root", ids["root"]),
        OpMove::new(r1.tick(), ids["root"], "a", ids["a"]),
        OpMove::new(r1.tick(), ids["root"], "b", ids["b"]),
        OpMove::new(r1.tick(), ids["root"], "c", ids["c"]),
    ];

    r1.apply_ops(&ops);
    r2.apply_ops(&ops);

    println!("Initial tree state on both replicas");
    print_tree(r1.tree(), &ids["root"]);

    // replica_1 moves /root/a to /root/b
    let repl1_ops = vec![OpMove::new(r1.tick(), ids["b"], "a", ids["a"])];

    // replica_2 "simultaneously" moves /root/a to /root/c
    let repl2_ops = vec![OpMove::new(r2.tick(), ids["c"], "a", ids["a"])];

    // replica_1 applies his op, then merges op from replica_2
    r1.apply_ops(&repl1_ops);
    println!("\nreplica_1 tree after move");
    print_tree(r1.tree(), &ids["root"]);
    r1.apply_ops(&repl2_ops);

    // replica_2 applies his op, then merges op from replica_1
    r2.apply_ops(&repl2_ops);
    println!("\nreplica_2 tree after move");
    print_tree(r2.tree(), &ids["root"]);
    r2.apply_ops(&repl1_ops);

    // expected result: state is the same on both replicas
    // and final path is /root/c/a because last-writer-wins
    // and replica_2's op has a later timestamp.
    //    if r1.state.is_equal(&r2.state) {
    if r1.state == r2.state {
        println!("\nreplica_1 state matches replica_2 state after each merges other's change.  conflict resolved!");
        print_replica_trees(&r1, &r2, &ids["root"]);
    } else {
        println!("\nwarning: replica_1 state does not match replica_2 state after merge");
        print_replica_trees(&r1, &r2, &ids["root"]);
        println!("-- replica_1 state --");
        println!("{:#?}", r1.state);
        println!("\n-- replica_2 state --");
        println!("{:#?}", r2.state);
    }
}

fn test_concurrent_moves_cycle() {
    let mut r1: Replica<TypeId, TypeMeta, TypeActor> = Replica::new(new_id());
    let mut r2: Replica<TypeId, TypeMeta, TypeActor> = Replica::new(new_id());

    let ids: HashMap<&str, TypeId> = [
        ("root", 0),
        ("a", new_id()),
        ("b", new_id()),
        ("c", new_id()),
    ]
    .iter()
    .cloned()
    .collect();

    // Setup initial tree state.
    let ops = vec![
        OpMove::new(r1.tick(), 0, "root", ids["root"]),
        OpMove::new(r1.tick(), ids["root"], "a", ids["a"]),
        OpMove::new(r1.tick(), ids["root"], "b", ids["b"]),
        OpMove::new(r1.tick(), ids["a"], "c", ids["c"]),
    ];

    r1.apply_ops(&ops);
    r2.apply_ops(&ops);

    println!("Initial tree state on both replicas");
    print_tree(r1.tree(), &ids["root"]);

    // replica_1 moves /root/b to /root/a
    let repl1_ops = vec![OpMove::new(r1.tick(), ids["a"], "b", ids["b"])];

    // replica_2 "simultaneously" moves /root/a to /root/b
    let repl2_ops = vec![OpMove::new(r2.tick(), ids["b"], "a", ids["a"])];

    // replica_1 applies his op, then merges op from replica_2
    r1.apply_ops(&repl1_ops);
    println!("\nreplica_1 tree after move");
    print_tree(r1.tree(), &ids["root"]);
    r1.apply_ops(&repl2_ops);

    // replica_2 applies his op, then merges op from replica_1
    r2.apply_ops(&repl2_ops);
    println!("\nreplica_2 tree after move");
    print_tree(r2.tree(), &ids["root"]);
    r2.apply_ops(&repl1_ops);

    // expected result: state is the same on both replicas
    // and final path is /root/c/a because last-writer-wins
    // and replica_2's op has a later timestamp.
    if r1.state == r2.state {
        println!("\nreplica_1 state matches replica_2 state after each merges other's change.  conflict resolved!");
        print_replica_trees(&r1, &r2, &ids["root"]);
    } else {
        println!("\nwarning: replica_1 state does not match replica_2 state after merge");
        print_replica_trees(&r1, &r2, &ids["root"]);
        println!("-- replica_1 state --");
        println!("{:#?}", r1.state);
        println!("\n-- replica_2 state --");
        println!("{:#?}", r2.state);
    }
}

fn test_walk_deep_tree() {
    let mut r1: Replica<TypeId, TypeMeta, TypeActor> = Replica::new(new_id());

    let ids: HashMap<&str, TypeId> = [("root", 0)].iter().cloned().collect();

    // Generate initial tree state.
    println!("generating ops...");
    let mut ops = vec![OpMove::new(r1.tick(), 0, "root", ids["root"])];
    mktree_ops(&mut ops, &mut r1, ids["root"], 2, 13);

    println!("applying ops...");
    r1.apply_ops(&ops);

    println!("walking tree...");
    r1.tree().walk(&ids["root"], &|tree, node_id, depth| {
        if false {
            let meta = match tree.find(node_id) {
                Some(tn) => format!("{:?}", tn.metadata()),
                None => format!("{:?}", node_id),
            };
            println!("{:indent$}{}", "", meta, indent = depth);
        }
    });

    println!("\nnodes in tree: {}", ops.len());
}

fn test_truncate_log() {
    let mut replicas: Vec<Replica<TypeId, TypeMeta, TypeActor>> = Vec::new();
    let num_replicas = 5;

    // start some replicas.
    for _i in 0..num_replicas {
        let mut r: Replica<TypeId, TypeMeta, TypeActor> = Replica::new(new_id());
        r.track_causally_stable_threshold(true); // needed for truncation
        replicas.push(r);
    }

    let root_id = new_id();

    // Generate initial tree state.
    let mut ops = vec![OpMove::new(replicas[0].tick(), 0, "root", root_id)];

    println!("generating move operations...");

    // generate some initial ops from all replicas.
    for mut r in replicas.iter_mut() {
        let finaldepth = rand::thread_rng().gen_range(3, 6);
        mktree_ops(&mut ops, &mut r, root_id, 2, finaldepth);
    }

    // apply all ops to all replicas
    println!(
        "applying {} operations to all {} replicas...\n",
        ops.len(),
        replicas.len()
    );
    apply_ops_to_replicas(&mut replicas, &ops);

    #[derive(Debug)]
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

fn test_move_to_trash() {
    let mut r1: Replica<TypeId, TypeMeta, TypeActor> = Replica::new(new_id());
    let mut r2: Replica<TypeId, TypeMeta, TypeActor> = Replica::new(new_id());

    r1.track_causally_stable_threshold(true);
    r2.track_causally_stable_threshold(true);

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
        OpMove::new(r1.tick(), ids["forest"], "root", ids["root"]),
        OpMove::new(r1.tick(), ids["forest"], "trash", ids["trash"]),
        OpMove::new(r1.tick(), ids["root"], "home", ids["home"]),
        OpMove::new(r1.tick(), ids["home"], "bob", ids["bob"]),
        OpMove::new(r1.tick(), ids["bob"], "project", ids["project"]),
    ];

    // add some nodes under project
    mktree_ops(&mut ops, &mut r1, ids["project"], 2, 3);
    r1.apply_ops(&ops);
    r2.apply_ops(&ops);

    println!("Initial tree");
    print_tree(r1.tree(), &ids["forest"]);

    // move project to trash
    let ops = vec![OpMove::new(
        r1.tick(),
        ids["trash"],
        "project",
        ids["project"],
    )];
    r1.apply_ops(&ops);
    r2.apply_ops(&ops);

    println!("\nAfter project moved to trash (deleted) on both replicas");
    print_tree(r1.tree(), &ids["forest"]);

    // Initially, trashed nodes must be retained because a concurrent move
    // operation may move them back out of the trash.
    //
    // Once the operation that moved a node to the trash is causally
    // stable, we know that no future operations will refer to this node,
    // and so the trashed node and its descendants can be discarded.
    //
    // note:  change r1.tick() to r2.tick() for any of the above operations to
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
Usage: tree <test>

<test> can be any of:
  test_concurrent_moves
  test_concurrent_moves_cycle
  test_truncate_log
  test_walk_deep_tree
  test_move_to_trash

";
    println!("{}", buf);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let test = if args.len() > 1 { &args[1] } else { "" };

    match test {
        "test_concurrent_moves" => test_concurrent_moves(),
        "test_concurrent_moves_cycle" => test_concurrent_moves_cycle(),
        "test_truncate_log" => test_truncate_log(),
        "test_walk_deep_tree" => test_walk_deep_tree(),
        "test_move_to_trash" => test_move_to_trash(),

        _ => print_help(),
    }
}
