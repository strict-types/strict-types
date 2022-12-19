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

#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    dead_code,
    //missing_docs
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[macro_use]
extern crate amplify;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_crate as serde;

#[macro_use]
mod macros;

mod util;
pub mod primitive;
pub mod ast;
pub mod dtl;
mod encoding;

pub use ast::{FieldName, KeyTy, SemId, Translate, Ty, TypeRef};
pub use dtl::{Dependency, EmbeddedRef, LibAlias, LibName, LibRef, TypeLib, TypeLibId, TypeSystem};
#[cfg(test)]
pub(crate) use encoding::test;
pub use encoding::{
    CheckedWriter, Cls, Decode, DecodeError, Deserialize, Encode, Serialize, StenWrite,
    StructWriter, Writer,
};
pub use util::{Ident, SemVer, TypeName, Urn};

// TODO: Check guarantees on type and lib sizing

/// Type information which can be automatically derived out of -- or provided by a rust type via
/// implementing [`StenSchema`] trait.
///
/// The type contains a recursive information about all nested types, and thus can operate without
/// any type library.
///
/// The type has to be [`Translate`]ed into [`TypeLib`] or [`TypeSystem`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StenType {
    /// Type name which should match rust type name in most of the cases
    pub name: Option<TypeName>,
    /// Type structure abstract syntax tree
    pub ty: Box<Ty<StenType>>,
}

impl StenSchema for StenType {
    const STEN_TYPE_NAME: &'static str = "StenType";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields! {
            "name" => <Option<TypeName>>::sten_type(),
            "ty" => Ty::<StenType>::sten_type(),
        })
    }
}

impl StenType {
    pub fn byte() -> StenType { StenType::unnamed(Ty::BYTE) }
    pub fn ascii_char() -> StenType { StenType::named("Ascii", Ty::<StenType>::ascii_char()) }

    pub fn unnamed(ty: Ty<StenType>) -> StenType {
        StenType {
            name: None,
            ty: Box::new(ty),
        }
    }

    pub fn named(name: &'static str, ty: Ty<StenType>) -> StenType {
        StenType {
            name: Some(tn!(name)),
            ty: Box::new(ty),
        }
    }
}

/// A type which can be deterministically represented in terms of strict encoding schema.
pub trait StenSchema {
    /// Strict encoding type name.
    const STEN_TYPE_NAME: &'static str;

    /// Returns [`StenType`] representation of this structure
    fn sten_type() -> StenType { StenType::named(Self::STEN_TYPE_NAME, Self::sten_ty()) }

    /// Returns AST representing strict encoding of the data.
    fn sten_ty() -> Ty<StenType>;
}
