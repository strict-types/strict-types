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

use std::io;

use amplify::confinement::MediumVec;
use amplify::num::u24;
use amplify::{Wrapper, WriteCounter};

use super::inner::TyInner;
use super::Ty;
use crate::ast::ty::{RecursiveRef, SubTy};
use crate::ast::{Field, Fields, TypeRef, Variants};
use crate::dtl::InlineRef;
use crate::primitive::Primitive;
use crate::util::Sizing;
use crate::{FieldName, KeyTy, StenType};

pub enum DecodeError {}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Cls {
    Primitive = 0,
    Enum = 1,
    Union = 2,
    Struct = 3,
    Array = 4,
    Unicode = 5,
    List = 6,
    Set = 7,
    Map = 8,
}

impl Cls {
    pub const ALL: [Cls; 9] = [
        Cls::Primitive,
        Cls::Enum,
        Cls::Union,
        Cls::Struct,
        Cls::Array,
        Cls::Unicode,
        Cls::List,
        Cls::Set,
        Cls::Map,
    ];
}

impl TryFrom<u8> for Cls {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        for cls in Cls::ALL {
            if cls as u8 == value {
                return Ok(cls);
            }
        }
        return Err(value);
    }
}

impl<Ref: TypeRef> TyInner<Ref> {
    pub const fn cls(&self) -> Cls {
        match self {
            TyInner::Primitive(_) => Cls::Primitive,
            TyInner::Enum(_) => Cls::Enum,
            TyInner::Union(_) => Cls::Union,
            TyInner::Struct(_) => Cls::Struct,
            TyInner::Array(_, _) => Cls::Array,
            TyInner::Unicode(_) => Cls::Unicode,
            TyInner::List(_, _) => Cls::List,
            TyInner::Set(_, _) => Cls::Set,
            TyInner::Map(_, _, _) => Cls::Map,
        }
    }
}

impl KeyTy {
    pub const fn cls(&self) -> Cls {
        match self {
            KeyTy::Primitive(_) => Cls::Primitive,
            KeyTy::Enum(_) => Cls::Enum,
            KeyTy::Array(_) => Cls::Array,
            KeyTy::Unicode(_) => Cls::Unicode,
            KeyTy::Bytes(_) => Cls::List,
        }
    }
}

impl<Ref: TypeRef + Encode> Ty<Ref> {
    pub fn from_serialized(ast_data: MediumVec<u8>) -> Result<Self, DecodeError> { todo!() }

    pub fn serialized_len(&self) -> usize {
        let mut counter = WriteCounter::default();
        self.encode(&mut counter).expect("counter doesn't error");
        counter.count
    }

    pub fn to_serialized(&self) -> MediumVec<u8> {
        let len = self.serialized_len();
        debug_assert!(
            len > u24::MAX.into_usize(),
            "Ty type guarantees on the data size are broken"
        );
        let mut ast_data = Vec::with_capacity(len);
        self.encode(&mut ast_data).expect("memory writers do not error");
        MediumVec::try_from(ast_data).expect("Ty type guarantees on the data size are broken")
    }
}

pub trait Encode {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error>;
}

impl<Ref: TypeRef + Encode> Encode for Ty<Ref> {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write_all(&[self.cls() as u8])?;
        match self.as_inner() {
            TyInner::Primitive(prim) => prim.encode(writer),
            TyInner::Enum(vars) => vars.encode(writer),
            TyInner::Union(fields) => fields.encode(writer),
            TyInner::Struct(fields) => fields.encode(writer),
            TyInner::Array(ty, len) => {
                writer.write_all(&len.to_le_bytes())?;
                ty.encode(writer)
            }
            TyInner::Unicode(sizing) => sizing.encode(writer),
            TyInner::List(ty, sizing) => {
                ty.encode(writer)?;
                sizing.encode(writer)
            }
            TyInner::Set(ty, sizing) => {
                ty.encode(writer)?;
                sizing.encode(writer)
            }
            TyInner::Map(key, ty, sizing) => {
                key.encode(writer)?;
                ty.encode(writer)?;
                sizing.encode(writer)
            }
        }
    }
}

impl Encode for Primitive {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write_all(&[self.into_code()])
    }
}

impl Encode for StenType {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.as_ty().encode(writer)
    }
}

impl Encode for SubTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.as_ty().encode(writer)
    }
}

impl Encode for InlineRef {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self {
            InlineRef::Name(name) => name.encode(writer),
            InlineRef::Inline(ty) => ty.encode(writer),
        }
    }
}

impl Encode for KeyTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write_all(&[self.cls() as u8])?;
        match self {
            KeyTy::Primitive(prim) => prim.encode(writer),
            KeyTy::Enum(vars) => vars.encode(writer),
            KeyTy::Array(len) => writer.write_all(&len.to_le_bytes()),
            KeyTy::Unicode(sizing) => sizing.encode(writer),
            KeyTy::Bytes(sizing) => sizing.encode(writer),
        }
    }
}

impl Encode for Variants {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write_all(&[self.len_u8()])?;
        for (field, val) in self.as_inner() {
            field.encode(writer)?;
            writer.write_all(&[*val])?;
        }
        Ok(())
    }
}

impl Encode for Field {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        if let Some(name) = &self.name {
            name.encode(writer)?;
        }
        writer.write_all(&[self.ord])
    }
}

impl<Ref: TypeRef + Encode, const OP: bool> Encode for Fields<Ref, OP> {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write(&[self.len_u8()])?;
        for (name, ty) in self {
            name.encode(writer)?;
            ty.encode(writer)?;
        }
        Ok(())
    }
}

impl Encode for FieldName {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write_all(self.as_bytes())
    }
}

impl Encode for Sizing {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write_all(&self.min.to_le_bytes())?;
        writer.write_all(&self.max.to_le_bytes())
    }
}
