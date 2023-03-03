// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 UBIDECO Institute
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Strict values: schema-less representation of strict types. The module includes:
//! - [`path`]: path accessors/introspects into strict values;
//! - [STON][ston]: strict type object notation, a JSON-like representation of strict types;
//! - [`reify`]: conversion between strict encoding and strict values;
//! - [`typify`]: checks of strict values against strict type schema;
//! - [`convert`]: conversion between strict values and other text representations (JSON, YAML,
//!   TOML, etc).

#[macro_use]
mod val;
pub mod path;
pub mod ston;
pub mod typify;
pub mod reify;
pub mod convert;

pub use val::{EnumTag, StrictNum, StrictVal};
