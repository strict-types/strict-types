// Strict encoding schema library, implementing validation and parsing of strict encoded data
// against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2019-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2024-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2022-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    //dead_code,
    //missing_docs
)]
#![allow(unused_braces)] // Due to rust compiler bug not understanding proc macro expressions
#![allow(clippy::result_large_err)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[macro_use]
extern crate amplify;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
#[macro_use]
pub extern crate strict_encoding as encoding;
extern crate core;

pub use encoding::derive;
pub use strict_encoding::{
    fname, ident, impl_ident_subtype, impl_ident_type, impl_strict_newtype, impl_strict_struct,
    libname, strict_dumb, tn, vname, DecodeError, DeserializeError, FieldName, Ident,
    InvalidRString, LibName, RString, SerializeError, StrictDecode, StrictDeserialize, StrictDumb,
    StrictEncode, StrictSerialize, StrictType, TypeName, VariantError, VariantName,
};

#[macro_use]
mod macros;
mod util;
pub mod ast;
pub mod typelib;
pub mod typesys;
pub mod value;
pub mod stl;
pub mod layout;

pub use ast::{Cls, PrimitiveRef, SemId, Translate, Ty, TypeRef};
pub use typelib::{
    CompileError, Dependency, LibBuilder, LibRef, SymbolRef, SymbolicLib, TranspileError,
    TranspileRef, TypeLib, TypeLibId,
};
pub use typesys::{SymbolicSys, SystemBuilder, TypeSymbol, TypeSysId, TypeSystem};
pub use util::{parse_args, BuildFragment, PreFragment, SemVer, StlFormat, UnknownFormat, Urn};
pub use value::{decode, ston, typify, KeyStep, Path, PathError, Step, StrictVal};

pub trait CommitConsume {
    fn commit_consume(&mut self, data: impl AsRef<[u8]>);
}

impl CommitConsume for sha2::Sha256 {
    fn commit_consume(&mut self, data: impl AsRef<[u8]>) {
        use sha2::Digest;
        self.update(data)
    }
}
