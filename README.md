<div align="center">
  <h1>cfg</h1>
  <p>
    <strong>Context-free grammar tools.</strong>
  </p>
  <p>

[![crates.io][crates.io shield]][crates.io link]
[![Documentation][docs.rs badge]][docs.rs link]
![Rust CI][github ci badge]
[![rustc 1.0+]][Rust 1.0]
[![serde_derive: rustc 1.31+]][Rust 1.31]
<br />
[![Dependency Status][deps.rs status]][deps.rs link]
[![Download Status][shields.io download count]][crates.io link]

  </p>
</div>

[crates.io shield]: https://img.shields.io/crates/v/cfg?label=latest
[crates.io link]: https://crates.io/crates/cfg
[docs.rs badge]: https://docs.rs/cfg/badge.svg?version=0.5.0
[docs.rs link]: https://docs.rs/cfg/0.5.0/bit_vec/
[github ci badge]: https://github.com/contain-rs/linked-hash-map/workflows/Rust/badge.svg?branch=master
[rustc 1.0+]: https://img.shields.io/badge/rustc-1.0%2B-blue.svg
[serde_derive: rustc 1.31+]: https://img.shields.io/badge/serde_derive-rustc_1.31+-lightgray.svg
[Rust 1.0]: https://blog.rust-lang.org/2015/05/15/Rust-1.0.html
[Rust 1.31]: https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html
[deps.rs status]: https://deps.rs/crate/cfg/0.5.0/status.svg
[deps.rs link]: https://deps.rs/crate/cfg/0.5.0
[shields.io download count]: https://img.shields.io/crates/d/cfg.svg

Rust library for manipulating context-free grammars.
[You can check the documentation here](https://docs.rs/cfg/).

## Analyzing and modifying grammars

The following features are implemented thus far:

* cycle detection and elimination,
* useless rule detection and elimination,
* grammar binarization,
* nulling rule elimination for binarized grammars,
* FIRST and FOLLOW set computation,
* minimal distance computation,
* unused symbol removal.

## Building grammars

`cfg` includes an interface that simplifies grammar construction.

### Generating symbols

The easiest way of generating symbols is with the `sym` method. The library is unaware
of the start symbol.

```rust
let mut grammar: Cfg = Cfg::new();
let (start, expr, identifier, number,
     plus, multiply, power, l_paren, r_paren, digit) = grammar.sym();
```

### Building grammar rules

Rules have a LHS symbol and zero or more RHS symbols.

```rust
grammar.rule(start).rhs([expr])
                   .rhs([identifier, l_paren, expr, r_paren]);
```

### Building sequence rules

Sequence rules have a LHS symbol, a RHS symbol, a range of repetitions, and
optional separation. Aside from separation, they closely resemble regular
expression repetitions.

```rust
grammar.sequence(number).inclusive(1, None).rhs(digit);
```

### Building precedenced rules

Precedenced rules are the most convenient way to describe operators. Once
built, they are immediately rewritten into basic grammar rules, and unique
symbols are generated. Operator associativity can be set to `Right` or
`Group`. It's `Left` by default.

```rust
use cfg::precedence::Associativity::{Right, Group};

grammar.precedenced_rule(expr)
           .rhs([number])
           .rhs([identifier])
           .associativity(Group)
           .rhs([l_paren, expr, r_paren])
       .lower_precedence()
           .associativity(Right)
           .rhs([expr, power, expr])
       .lower_precedence()
           .rhs([expr, multiply, expr])
       .lower_precedence()
           .rhs([expr, plus, expr]);
```

## Using a custom grammar representation

Your grammar type has to implement several traits:

* `RuleContainer`
* `ContextFree`
* `ContextFreeRef`
* `ContextFreeMut`

## License

Dual-licensed for compatibility with the Rust project.

Licensed under the Apache License Version 2.0:
http://www.apache.org/licenses/LICENSE-2.0, or the MIT license:
http://opensource.org/licenses/MIT, at your option.
