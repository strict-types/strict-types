# Rust implementation of strict encoding schema (STENS)

Strict encoding is a deterministic schema-base binary serialization format 
for algebraic types (ADT) which provides automatic strong type checking. It is
used in consensus protocols, networking, AluVM and long-term data storage. 

This library provides primitives for describing strict encoding schemata, 
validating and parsing structured data using the schema and for on-the-fly
checking of algebraic data type serialziation/deserialization.

To learn more about strict encoding [read the spec](https://www.strict-encoding.org).

Strict encoding schema works with type definitions. It allows:
- static analysis of data types, like
  * defining semantic type ids;
  * specifying exact memory layout;
  * type equivalence in terms of semantics and memory layout;
  * size of serialized data
- composing types into type libraries;
- versioning type libraries basing on the semantic types;

Current rust implementation additionally allows to build type libraries out of
rust data types which implement `StrictEncoding` trait -- and ensures that the
deserialization with `StrictDecode` follows the same memory and semantic layout.
