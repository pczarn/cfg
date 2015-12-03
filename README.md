## cfg • ![Build status](https://api.travis-ci.org/pczarn/cfg.png?branch=master) ![Latest version](https://img.shields.io/crates/v/cfg.png)

Rust library for manipulating context-free grammars.
[You can check the documentation here](http://pczarn.github.io/cfg/).

## Analyzing and modifying grammars

The following features are implemented thus far:

* cycle detection and elimination,
* useless rule detection and elimination,
* grammar binarization,
* nulling rule elimination for binarized grammars,
* FIRST and FOLLOW set computation.

## Building grammars

`cfg` includes an interface that simplifies grammar construction.

### Generating symbols

The easiest way of generating symbols is with the `sym` method. The start symbol
isn't generated, because it's constant for all grammars.

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
grammar.sequence(number).rhs(digit, 1..);
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

* `SymbolSource`
* `RuleContainer`
* `ContextFree`
* `ContextFreeRef`
* `ContextFreeMut`

## License

Dual-licensed for compatibility with the Rust project.

Licensed under the Apache License Version 2.0:
http://www.apache.org/licenses/LICENSE-2.0, or the MIT license:
http://opensource.org/licenses/MIT, at your option.
