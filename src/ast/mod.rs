// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Association, Switzerland.
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
    pub fn try_into_key_ty(self) -> Result<KeyTy, TyInner> {
        Ok(match self.0 {
            TyInner::Primitive(code) => KeyTy::Primitive(code),
            TyInner::Enum(vars) => KeyTy::Enum(vars),
            TyInner::Array(ty, len) => KeyTy::Array(ty, len),
            TyInner::Ascii(sizing) => KeyTy::Ascii(sizing),
            TyInner::Unicode(sizing) => KeyTy::Unicode(sizing),
            me @ TyInner::Union(_)
            | me @ TyInner::Struct(_)
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
    Array(Box<TyInner>, u16),
    Ascii(Sizing),
    Unicode(Sizing),
    List(Box<TyInner>, Sizing),
    Set(Box<TyInner>, Sizing),
    Map(KeyTy, Box<TyInner>, Sizing),
}

impl TyInner {
    pub const fn unit() -> TyInner { TyInner::Primitive(UNIT) }
    pub const fn byte() -> TyInner { TyInner::Primitive(BYTE) }
    pub const fn char() -> TyInner { TyInner::Primitive(CHAR) }

    pub const fn u8() -> TyInner { TyInner::Primitive(U8) }
    pub const fn u16() -> TyInner { TyInner::Primitive(U16) }
    pub const fn u24() -> TyInner { TyInner::Primitive(U24) }
    pub const fn u32() -> TyInner { TyInner::Primitive(U32) }
    pub const fn u64() -> TyInner { TyInner::Primitive(U64) }
    pub const fn u128() -> TyInner { TyInner::Primitive(U128) }
    pub const fn u256() -> TyInner { TyInner::Primitive(U256) }
    pub const fn u512() -> TyInner { TyInner::Primitive(U512) }
    pub const fn u1024() -> TyInner { TyInner::Primitive(U1024) }

    pub const fn i8() -> TyInner { TyInner::Primitive(I8) }
    pub const fn i16() -> TyInner { TyInner::Primitive(I16) }
    pub const fn i24() -> TyInner { TyInner::Primitive(I24) }
    pub const fn i32() -> TyInner { TyInner::Primitive(I32) }
    pub const fn i64() -> TyInner { TyInner::Primitive(I64) }
    pub const fn i128() -> TyInner { TyInner::Primitive(I128) }
    pub const fn i256() -> TyInner { TyInner::Primitive(I256) }
    pub const fn i512() -> TyInner { TyInner::Primitive(I512) }
    pub const fn i1024() -> TyInner { TyInner::Primitive(I1024) }

    pub const fn f16b() -> TyInner { TyInner::Primitive(F16B) }
    pub const fn f16() -> TyInner { TyInner::Primitive(F16) }
    pub const fn f32() -> TyInner { TyInner::Primitive(F32) }
    pub const fn f64() -> TyInner { TyInner::Primitive(F64) }
    pub const fn f80() -> TyInner { TyInner::Primitive(F80) }
    pub const fn f128() -> TyInner { TyInner::Primitive(F128) }
    pub const fn f256() -> TyInner { TyInner::Primitive(F256) }

    pub fn enumerate(variants: Variants) -> TyInner { TyInner::Enum(variants) }

    pub fn byte_array(len: u16) -> TyInner {
        TyInner::Array(Box::new(TyInner::Primitive(BYTE)), len)
    }

    pub fn bytes() -> TyInner { TyInner::List(Box::new(TyInner::Primitive(BYTE)), Sizing::U16) }
    pub fn list(ty: TyInner, sizing: Sizing) -> TyInner { TyInner::List(Box::new(ty), sizing) }
    pub fn set(ty: TyInner, sizing: Sizing) -> TyInner { TyInner::Set(Box::new(ty), sizing) }
    pub fn map(key: KeyTy, val: TyInner, sizing: Sizing) -> TyInner {
        TyInner::Map(key, Box::new(val), sizing)
    }

    pub fn option(ty: TyInner) -> TyInner {
        TyInner::Union(alternatives![
            "None" => 0 => TyInner::unit(),
            "Some" => 1 => ty
        ])
    }

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
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum KeyTy {
    Primitive(u8),
    Enum(Variants),
    Array(Box<TyInner>, u16),
    Ascii(Sizing),
    Unicode(Sizing),
    Bytes(Sizing),
}

pub type Alternatives = Confined<BTreeMap<&'static str, Alternative>, 1, { u8::MAX as usize }>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Alternative {
    pub id: u8,
    pub ty: Box<TyInner>,
}

impl Alternative {
    pub fn new(id: u8, ty: TyInner) -> Alternative {
        Alternative {
            id,
            ty: Box::new(ty),
        }
    }
}

pub type Fields = Confined<BTreeMap<&'static str, Box<TyInner>>, 1, { u8::MAX as usize }>;

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
