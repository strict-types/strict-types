// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 Ubideco Project
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

#![feature(associated_type_defaults)]
#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    //dead_code,
    //missing_docs
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[macro_use]
extern crate amplify;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_crate as serde;
extern crate core;

#[macro_use]
mod macros;
#[macro_use]
mod ident;
mod util;
pub mod primitive;
pub mod ast;
pub mod typelib;
//pub mod typesys;
pub mod encoding;

pub use ast::{Cls, KeyTy, SemId, Ty, TypeRef};
pub use encoding::*;
pub use ident::{FieldName, Ident, InvalidIdent, LibName, TypeName};
// #[cfg(test)]
// pub(crate) use encoding::test;
pub use typelib::{Dependency, LibAlias, LibRef, TypeLib, TypeLibId};
//pub use typesys::{EmbeddedRef, TypeSystem};
pub use util::{SemVer, Urn};

const STEN_LIB: &'static str = "StEn";
