// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
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
mod serialize;
mod write;
mod check;

#[cfg(test)]
pub(crate) mod test;

pub use ast::{FieldName, KeyTy, Translate, Ty, TyId, TypeRef};
pub use check::CheckedWriter;
pub use dtl::{Dependency, EmbeddedTy, LibAlias, LibName, LibTy, TypeLib, TypeLibId, TypeSystem};
pub use serialize::{Cls, Decode, DecodeError, Deserialize, Encode, Serialize};
pub use util::{Ident, SemVer, TypeName, Urn};
pub use write::{StenWrite, StructWriter, Writer};

/// Type information which can be automatically derived out of -- or provided by a rust type via
/// implementing [`StenSchema`] trait.
///
/// The type contains a recursive information about all nested types, and thus can operate without
/// any type library.
///
/// This form of type information can not be serialized and is only an in-memory representation.
///
/// In order to perform type serialization the type has to be [`Translate`]ed into either
/// [`TypeLib`] or [`TypeSystem`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StenType {
    /// Type name which should match rust type name in most of the cases
    pub name: &'static str,
    /// Type structure abstract syntax tree
    pub ty: Box<Ty<StenType>>,
}

impl StenSchema for StenType {
    const STEN_TYPE_NAME: &'static str = "StenType";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields! {
            "name" => Ident::sten_type(),
            "ty" => Ty::<StenType>::sten_type(),
        })
    }
}

impl std::ops::Deref for StenType {
    type Target = Ty<StenType>;

    fn deref(&self) -> &Self::Target { self.ty.deref() }
}

impl StenType {
    pub fn unit() -> StenType {
        StenType {
            name: "",
            ty: Box::new(Ty::UNIT),
        }
    }

    pub fn byte() -> StenType {
        StenType {
            name: "Byte",
            ty: Box::new(Ty::BYTE),
        }
    }

    pub fn ascii() -> StenType {
        StenType {
            name: "Ascii",
            ty: Box::new(Ty::<StenType>::ascii_char()),
        }
    }

    pub fn new(name: &'static str, ty: Ty<StenType>) -> StenType {
        StenType {
            name,
            ty: Box::new(ty),
        }
    }
}

/// A type which can be deterministically represented in terms of strict encoding schema.
pub trait StenSchema {
    /// Strict encoding type name.
    const STEN_TYPE_NAME: &'static str;

    /// Returns [`StenType`] representation of this structure
    fn sten_type() -> StenType {
        StenType {
            name: Self::STEN_TYPE_NAME,
            ty: Box::new(Self::sten_ty()),
        }
    }

    /// Returns AST representing strict encoding of the data.
    fn sten_ty() -> Ty<StenType>;
}
