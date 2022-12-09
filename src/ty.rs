// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use amplify::confinement::Confined;

use crate::alternatives;
use crate::primitive::constants::*;
use crate::primitive::NumInfo;
use crate::util::{Size, Sizing};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ty {
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

impl Ty {
    pub fn unit() -> Ty { Ty::Primitive(UNIT) }
    pub fn byte() -> Ty { Ty::Primitive(BYTE) }
    pub fn char() -> Ty { Ty::Primitive(CHAR) }

    pub fn u8() -> Ty { Ty::Primitive(U8) }
    pub fn u16() -> Ty { Ty::Primitive(U16) }
    pub fn u24() -> Ty { Ty::Primitive(U24) }
    pub fn u32() -> Ty { Ty::Primitive(U32) }
    pub fn u64() -> Ty { Ty::Primitive(U64) }
    pub fn u128() -> Ty { Ty::Primitive(U128) }
    pub fn u256() -> Ty { Ty::Primitive(U256) }
    pub fn u512() -> Ty { Ty::Primitive(U512) }
    pub fn u1024() -> Ty { Ty::Primitive(U1024) }

    pub fn i8() -> Ty { Ty::Primitive(I8) }
    pub fn i16() -> Ty { Ty::Primitive(I16) }
    pub fn i24() -> Ty { Ty::Primitive(I24) }
    pub fn i32() -> Ty { Ty::Primitive(I32) }
    pub fn i64() -> Ty { Ty::Primitive(I64) }
    pub fn i128() -> Ty { Ty::Primitive(I128) }
    pub fn i256() -> Ty { Ty::Primitive(I256) }
    pub fn i512() -> Ty { Ty::Primitive(I512) }
    pub fn i1024() -> Ty { Ty::Primitive(I1024) }

    pub fn f16b() -> Ty { Ty::Primitive(F16B) }
    pub fn f16() -> Ty { Ty::Primitive(F16) }
    pub fn f32() -> Ty { Ty::Primitive(F32) }
    pub fn f64() -> Ty { Ty::Primitive(F64) }
    pub fn f80() -> Ty { Ty::Primitive(F80) }
    pub fn f128() -> Ty { Ty::Primitive(F128) }
    pub fn f256() -> Ty { Ty::Primitive(F256) }

    pub fn enumerate(variants: Variants) -> Ty { Ty::Enum(variants) }

    pub fn byte_array(len: u16) -> Ty { Ty::Array(Box::new(Ty::Primitive(BYTE)), len) }

    pub fn bytes() -> Ty { Ty::List(Box::new(Ty::Primitive(BYTE)), Sizing::U16) }
    pub fn list(ty: Ty, sizing: Sizing) -> Ty { Ty::List(Box::new(ty), sizing) }
    pub fn set(ty: Ty, sizing: Sizing) -> Ty { Ty::Set(Box::new(ty), sizing) }
    pub fn map(key: KeyTy, val: Ty, sizing: Sizing) -> Ty { Ty::Map(key, Box::new(val), sizing) }

    pub fn option(ty: Ty) -> Ty {
        Ty::Union(alternatives![
            "None" => 0 => Ty::unit(),
            "Some" => 1 => ty
        ])
    }

    pub fn try_into_ty(self) -> Result<KeyTy, Ty> {
        Ok(match self {
            Ty::Primitive(code) => KeyTy::Primitive(code),
            Ty::Enum(vars) => KeyTy::Enum(vars),
            Ty::Array(ty, len) => KeyTy::Array(ty, len),
            Ty::Ascii(sizing) => KeyTy::Ascii(sizing),
            Ty::Unicode(sizing) => KeyTy::Unicode(sizing),
            me @ Ty::Union(_)
            | me @ Ty::Struct(_)
            | me @ Ty::List(_, _)
            | me @ Ty::Set(_, _)
            | me @ Ty::Map(_, _, _) => return Err(me),
        })
    }

    pub fn size(&self) -> Size {
        match self {
            Ty::Primitive(UNIT) | Ty::Primitive(BYTE) | Ty::Primitive(CHAR) => Size::Fixed(1),
            Ty::Primitive(F16B) => Size::Fixed(2),
            Ty::Primitive(code) => Size::Fixed(NumInfo::from_code(*code).size()),
            Ty::Union(fields) => {
                fields.values().map(|alt| alt.ty.size()).max().unwrap_or(Size::Fixed(0))
            }
            Ty::Struct(fields) => fields.values().map(|ty| ty.size()).sum(),
            Ty::Enum(_) => Size::Fixed(1),
            Ty::Array(_, len) => Size::Fixed(*len),
            Ty::Unicode(..) | Ty::Ascii(..) | Ty::List(..) | Ty::Set(..) | Ty::Map(..) => {
                Size::Variable
            }
        }
    }
}

/// Lexicographically sortable types which may serve as map keys.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum KeyTy {
    Primitive(u8),
    Enum(Variants),
    Array(Box<Ty>, u16),
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
