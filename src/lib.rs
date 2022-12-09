// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Project.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.

#[macro_use]
extern crate amplify;

#[macro_use]
mod macros;

mod util;
pub mod primitive;
pub mod ast;
pub mod dtl;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_crate as serde;
mod path;

use std::ops::Deref;

pub use ast::{FieldName, Ident, KeyTy, Translate, Ty, TyId, TypeRef};
pub use dtl::TypeLib;
pub use path::{Path, PathError, Step, TyIter};
pub use util::TypeName;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StenType {
    pub name: &'static str,
    pub ty: Box<Ty<StenType>>,
}

impl Deref for StenType {
    type Target = Ty<StenType>;

    fn deref(&self) -> &Self::Target { self.ty.deref() }
}

impl StenType {
    pub fn unit() -> StenType {
        StenType {
            name: "()",
            ty: Box::new(Ty::UNIT),
        }
    }

    pub fn byte() -> StenType {
        StenType {
            name: "Byte",
            ty: Box::new(Ty::BYTE),
        }
    }

    pub fn new(name: &'static str, ty: Ty<StenType>) -> StenType {
        StenType {
            name,
            ty: Box::new(ty),
        }
    }
}

/// A type which can be deterministically represented in terms of
/// strict encoding schema.
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
