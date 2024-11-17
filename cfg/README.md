<div align="center">
  <h1>cfg</h1>
  <p>
    <strong>Context-free grammar tools.</strong>
  </p>
  <p>

[![crates.io][crates.io shield]][crates.io link]
[![Documentation][docs.rs badge]][docs.rs link]
![Rust CI][github ci badge]
![MSRV][rustc 1.80+]
<br />
<br />
[![Dependency Status][deps.rs status]][deps.rs link]
[![Download Status][shields.io download count]][crates.io link]

  </p>
</div>

[crates.io shield]: https://img.shields.io/crates/v/cfg?label=latest
[crates.io link]: https://crates.io/crates/cfg
[docs.rs badge]: https://docs.rs/cfg/badge.svg?version=0.9.0
[docs.rs link]: https://docs.rs/cfg/0.9.0/cfg/
[github ci badge]: https://github.com/pczarn/cfg/workflows/CI/badge.svg?branch=master
[rustc 1.80+]: https://img.shields.io/badge/rustc-1.80%2B-blue.svg
[deps.rs status]: https://deps.rs/crate/cfg/0.9.0/status.svg
[deps.rs link]: https://deps.rs/crate/cfg/0.9.0
[shields.io download count]: https://img.shields.io/crates/d/cfg.svg

Rust library for manipulating context-free grammars.
[You can check the documentation here](https://docs.rs/cfg/latest/cfg/).

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
cfg = "0.9"
```

If you want grammar serialization support with `miniserde`, include the feature like this:

```toml
[dependencies]
cfg = { version = "0.9", features = ["serialize"] }
```

If you want weighted generation support, include the feature like this:

```toml
[dependencies]
cfg = { version = "0.9", features = ["weighted-generation"] }
```

If you want LL(1) classification support, include the feature like this:

```toml
[dependencies]
cfg = { version = "0.9", features = ["ll"] }
```

## Analyzing and modifying grammars

The following features are implemented thus far:

* rich rule building
  * sequence rules,
  * precedenced rules.
* conversions to a shape similar to Chomsky Normal Form
  * grammar binarization,
  * nulling rule elimination for binarized grammars.
* sanity
  * cycle detection and elimination,
  * useless rule detection and elimination,
  * unused symbol removal.
* analysis for LR(1), LL(1) and others
  * FIRST and FOLLOW set computation,
  * minimal distance computation,
  * LL(1) classification.
* tools for probabilistic grammars
  * generation for PCFGs + negative zero-width lookahead.

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

Example BNF:
```bnf
start ::= expr | identifier l_paren expr r_paren
```

With our library:
```rust
grammar.rule(start).rhs([expr])
                   .rhs([identifier, l_paren, expr, r_paren]);
```

### Building sequence rules

Sequence rules have a LHS symbol, a RHS symbol, a range of repetitions, and
optional separation. Aside from separation, they closely resemble regular
expression repetitions.

Example BNF:
```bnf
number ::= digit+
```

With our library:
```rust
SequencesToProductions::new(&mut grammar).sequence(number).inclusive(1, None).rhs(digit);
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

We've removed the option to plug in your custom grammar type through traits. You should find
it easy to fork the library and make your own types.

## License

Dual-licensed for compatibility with the Rust project.

Licensed under the Apache License Version 2.0:
http://www.apache.org/licenses/LICENSE-2.0, or the MIT license:
http://opensource.org/licenses/MIT, at your option.
