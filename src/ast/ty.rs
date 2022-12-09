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

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;

use amplify::confinement::Confined;

use crate::alternatives;
use crate::primitive::constants::*;
use crate::primitive::NumInfo;
use crate::util::{Size, Sizing};

/// Provides guarantees that the type information fits maximum type size
/// requirements, i.e. the serialized AST does not exceed `u24::MAX` bytes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ty(TyInner);

impl Deref for Ty {
    type Target = TyInner;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Ty {
    pub const UNIT: Ty = Ty(TyInner::Primitive(UNIT));
    pub const BYTE: Ty = Ty(TyInner::Primitive(BYTE));
    pub const CHAR: Ty = Ty(TyInner::Primitive(CHAR));

    pub const U8: Ty = Ty(TyInner::Primitive(U8));
    pub const U16: Ty = Ty(TyInner::Primitive(U16));
    pub const U24: Ty = Ty(TyInner::Primitive(U24));
    pub const U32: Ty = Ty(TyInner::Primitive(U32));
    pub const U64: Ty = Ty(TyInner::Primitive(U64));
    pub const U128: Ty = Ty(TyInner::Primitive(U128));
    pub const U256: Ty = Ty(TyInner::Primitive(U256));
    pub const U512: Ty = Ty(TyInner::Primitive(U512));
    pub const U1024: Ty = Ty(TyInner::Primitive(U1024));

    pub const I8: Ty = Ty(TyInner::Primitive(I8));
    pub const I16: Ty = Ty(TyInner::Primitive(I16));
    pub const I24: Ty = Ty(TyInner::Primitive(I24));
    pub const I32: Ty = Ty(TyInner::Primitive(I32));
    pub const I64: Ty = Ty(TyInner::Primitive(I64));
    pub const I128: Ty = Ty(TyInner::Primitive(I128));
    pub const I256: Ty = Ty(TyInner::Primitive(I256));
    pub const I512: Ty = Ty(TyInner::Primitive(I512));
    pub const I1024: Ty = Ty(TyInner::Primitive(I1024));

    pub const F16B: Ty = Ty(TyInner::Primitive(F16B));
    pub const F16: Ty = Ty(TyInner::Primitive(F16));
    pub const F32: Ty = Ty(TyInner::Primitive(F32));
    pub const F64: Ty = Ty(TyInner::Primitive(F64));
    pub const F80: Ty = Ty(TyInner::Primitive(F80));
    pub const F128: Ty = Ty(TyInner::Primitive(F128));
    pub const F256: Ty = Ty(TyInner::Primitive(F256));

    pub fn enumerate(variants: Variants) -> Self { Ty(TyInner::Enum(variants)) }

    pub fn union(alternatives: Alternatives) -> Self { Ty(TyInner::Union(alternatives)) }

    pub fn option(ty: Ty) -> Self {
        Ty(TyInner::Union(alternatives![
            "None" => 0 => Ty::UNIT,
            "Some" => 1 => ty
        ]))
    }

    pub fn byte_array(len: u16) -> Self { Ty(TyInner::Array(Box::new(Ty::BYTE), len)) }

    pub fn bytes() -> Self { Ty(TyInner::List(Box::new(Ty::BYTE), Sizing::U16)) }
    pub fn list(ty: Ty, sizing: Sizing) -> Self { Ty(TyInner::List(Box::new(ty), sizing)) }
    pub fn set(ty: Ty, sizing: Sizing) -> Self { Ty(TyInner::Set(Box::new(ty), sizing)) }
    pub fn map(key: KeyTy, val: Ty, sizing: Sizing) -> Self {
        Ty(TyInner::Map(key, Box::new(val), sizing))
    }
}

impl Ty {
    pub fn try_into_key_ty(self) -> Result<KeyTy, TyInner> {
        Ok(match self.0 {
            TyInner::Primitive(code) => KeyTy::Primitive(code),
            TyInner::Enum(vars) => KeyTy::Enum(vars),
            TyInner::Array(ty, len) if *ty == Ty::BYTE => KeyTy::Array(len),
            TyInner::Ascii(sizing) => KeyTy::Ascii(sizing),
            TyInner::Unicode(sizing) => KeyTy::Unicode(sizing),
            me @ TyInner::Union(_)
            | me @ TyInner::Struct(_)
            | me @ TyInner::Array(_, _)
            | me @ TyInner::List(_, _)
            | me @ TyInner::Set(_, _)
            | me @ TyInner::Map(_, _, _) => return Err(me),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TyInner {
    Primitive(u8),
    Enum(Variants),
    Union(Alternatives),
    Struct(Fields),
    Array(Box<Ty>, u16),
    Ascii(Sizing),
    Unicode(Sizing),
    List(Box<Ty>, Sizing),
    Set(Box<Ty>, Sizing),
    Map(KeyTy, Box<Ty>, Sizing),
}

impl TyInner {
    pub fn size(&self) -> Size {
        match self {
            TyInner::Primitive(UNIT) | TyInner::Primitive(BYTE) | TyInner::Primitive(CHAR) => {
                Size::Fixed(1)
            }
            TyInner::Primitive(F16B) => Size::Fixed(2),
            TyInner::Primitive(code) => Size::Fixed(NumInfo::from_code(*code).size()),
            TyInner::Union(fields) => {
                fields.values().map(|alt| alt.ty.size()).max().unwrap_or(Size::Fixed(0))
            }
            TyInner::Struct(fields) => fields.values().map(|ty| ty.size()).sum(),
            TyInner::Enum(_) => Size::Fixed(1),
            TyInner::Array(_, len) => Size::Fixed(*len),
            TyInner::Unicode(..)
            | TyInner::Ascii(..)
            | TyInner::List(..)
            | TyInner::Set(..)
            | TyInner::Map(..) => Size::Variable,
        }
    }
}

/// Lexicographically sortable types which may serve as map keys.
///
/// The type is always guaranteed to fit strict encoding AST serialization
/// bounds since it doesn't has a dynamically-sized types.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum KeyTy {
    Primitive(u8),
    Enum(Variants),
    /// Fixed-size byte array
    Array(u16),
    Ascii(Sizing),
    Unicode(Sizing),
    Bytes(Sizing),
}

pub type Alternatives = Confined<BTreeMap<&'static str, Alternative>, 1, { u8::MAX as usize }>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Alternative {
    pub id: u8,
    pub ty: Box<Ty>,
}

impl Alternative {
    pub fn new(id: u8, ty: Ty) -> Alternative {
        Alternative {
            id,
            ty: Box::new(ty),
        }
    }
}

pub type Fields = Confined<BTreeMap<&'static str, Box<Ty>>, 1, { u8::MAX as usize }>;

pub type Variants = Confined<BTreeSet<Variant>, 1, { u8::MAX as usize }>;

#[derive(Copy, Clone, Eq, Debug)]
pub struct Variant {
    pub name: &'static str,
    pub value: u8,
}

impl Variant {
    pub fn new(name: &'static str, value: u8) -> Variant { Variant { name, value } }
}

impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool { self.name == other.name || self.value == other.value }
}

impl PartialOrd for Variant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Variant {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        self.value.cmp(&other.value)
    }
}
