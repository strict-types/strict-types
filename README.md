# Strict types AST and typelib implementation

![Build](https://github.com/strict-types/strict-types/workflows/Build/badge.svg)
![Tests](https://github.com/strict-types/strict-types/workflows/Tests/badge.svg)
![Lints](https://github.com/strict-types/strict-types/workflows/Lints/badge.svg)
[![codecov](https://codecov.io/gh/strict-types/strict-types/branch/master/graph/badge.svg)](https://codecov.io/gh/strict-types/strict-types)

[![crates.io](https://img.shields.io/crates/v/strict_types)](https://crates.io/crates/strict_types)
[![Docs](https://docs.rs/strict_types/badge.svg)](https://docs.rs/strict_types)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Apache-2 licensed](https://img.shields.io/crates/l/strict_types)](./LICENSE)

#### Protobufs for functional programming

This is a set of libraries for working with abstract syntax trees and libraries
of [strict types] &ndash; type system made with category theory which ensures
provable properties and bounds for the in-memory and serialized type
representation.

Strict types is a formal notation for defining and serializing
[generalized algebraic data types (GADT)][gadt] in a deterministic
and confined way. It is developed with [type theory] in mind.

Strict Types are:

* __schema-based__ (with the schema being strict encoding notation),
* __semantic__, i.e. defines types not just as they are layed out in memory,
  but also depending on their meaning,
* __deterministic__, i.e. produces the same result for a given type,
* __portabile__, i.e. can run on ahy hardware architecture and OS, including
  low-performant embedded systems,
* __confined__, i.e. provides guarantees and static analysis on a maximum size
  of the typed data,
* __formally verifiabile__.

To learn more about strict encoding [read the spec](https://strict-types.org).

Strict types works with type definitions. It allows:

- static analysis of data types, like
    * defining semantic type ids;
    * specifying exact memory layout;
    * type equivalence in terms of semantics and memory layout;
    * size of serialized data
- composing types into type libraries;
- versioning type libraries basing on the semantic types;

The library allows to generate & compile strict type libraries (STL) from rust
types implementing `StrictEncoding` trait -- and ensures that the
deserialization with `StrictDecode` follows the same memory and semantic layout.

## Strict Types Library

The library is able to reflect on itself, producing replica of its rust data
types in strict types system.

Strict types library id:
`stl:9KALDYR8Nyjq4FdMW6kYoL7vdkWnqPqNuFnmE9qHpNjZ#lagoon-rodent-option`

Import this lib by putting in the file header
`import lagoon_rodent_option_9KALDYR8Nyjq4FdMW6kYoL7vdkWnqPqNuFnmE9qHpNjZ as StrictTypes`

Source code can be found in [`stl/StrictTypes.sty`] file.

## Contributing

[CONTRIBUTING.md](../CONTRIBUTING.md)

## License

The libraries are distributed on the terms of [Apache 2.0 license](LICENSE).

[strict types]: https://strict-types.org

[gadt]: https://en.wikipedia.org/wiki/Algebraic_data_type

[type theory]: https://en.wikipedia.org/wiki/Type_theory
