[![Build Status](https://travis-ci.org/tree_crdt/tree_crdt.svg?branch=master)](https://travis-ci.org/tree_crdt/crdt_tree) 
[![crates.io](http://meritbadge.herokuapp.com/crdt_tree)](https://crates.io/crates/crdt_tree)
[![docs.rs](https://docs.rs/crdt_tree/badge.svg)](https://docs.rs/crdt_tree)

# crdt_tree

A Conflict-free Replicated Data Type (CRDT) Tree written in Rust.

| [MaidSafe website](http://maidsafe.net) | [SAFE Network Forum](https://safenetforum.org/) |
|:-------:|:-------:|

## About

This crate aims to be an accurate implementation of the tree crdt algorithm described in the paper: 

[A highly-available move operation for replicated trees and distributed filesystems](https://martin.kleppmann.com/papers/move-op.pdf) by M. Kleppmann, et al.

Please refer to the paper for a description of the algorithm's properties.

For clarity, data structures in this implementation are named the same as in the paper (State, Tree) or close to (OpMove --> Move, LogOpMove --> LogOp). Some are not explicitly named in the paper, such as TreeId,TreeMeta, TreeNode, Clock.

### Additional References

- [CRDT: The Hard Parts](https://martin.kleppmann.com/2020/07/06/crdt-hard-parts-hydra.html)
- [Youtube Video: CRDT: The Hard Parts](https://youtu.be/x7drE24geUw)

## Usage

See [examples/tree.rs](examples/tree.rs) or [tests/tree.rs](tests/tree.rs).

In particular, the Replica struct in examples/tree.rs may be helpful.

## Other Implementations

There is a PHP implementation [here](https://github.com/dan-da/crdt-php).

## License

This Safe Network library is licensed under the BSD-3-Clause license.

See the [LICENSE](LICENSE) file for more details.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
