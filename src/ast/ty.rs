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

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Deref;

use amplify::confinement::Confined;
use amplify::{confinement, Wrapper};
use strict_encoding::constants::*;
use strict_encoding::{
    FieldName, Primitive, Sizing, StrictDecode, StrictDumb, StrictEncode, Variant, STRICT_TYPES_LIB,
};

use super::id::HashId;
use crate::ast::Iter;

/// Glue for constructing ASTs.
pub trait TypeRef:
    HashId + Clone + StrictEncode + StrictDecode + StrictDumb + Eq + Debug + Sized
{
    fn as_ty(&self) -> Option<&Ty<Self>> { None }
    fn type_refs(&self) -> Iter<Self> { Iter::from(self) }

    fn is_compound(&self) -> bool { false }
    fn is_byte(&self) -> bool { false }
    fn is_unicode_char(&self) -> bool { false }
    fn is_ascii_char(&self) -> bool { false }
}

pub trait PrimitiveRef: TypeRef {
    fn byte() -> Self;
    fn ascii_char() -> Self;
    fn unicode_char() -> Self;
}

impl TypeRef for KeyTy {}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = repr, into_u8, try_from_u8)]
#[display(lowercase)]
#[repr(u8)]
pub enum Cls {
    #[strict_type(dumb)]
    Primitive = 0,
    Unicode = 1,
    AsciiStr = 2,
    Enum = 3,
    Union = 4,
    Struct = 5,
    Tuple = 6,
    Array = 7,
    List = 8,
    Set = 9,
    Map = 10,
}

impl Cls {
    pub const ALL: [Cls; 11] = [
        Cls::Primitive,
        Cls::Unicode,
        Cls::AsciiStr,
        Cls::Enum,
        Cls::Union,
        Cls::Struct,
        Cls::Tuple,
        Cls::Array,
        Cls::List,
        Cls::Set,
        Cls::Map,
    ];
}

impl<Ref: TypeRef> Ty<Ref> {
    pub const fn cls(&self) -> Cls {
        match self {
            Ty::Primitive(_) => Cls::Primitive,
            Ty::Enum(_) => Cls::Enum,
            Ty::Union(_) => Cls::Union,
            Ty::Struct(_) => Cls::Struct,
            Ty::Tuple(_) => Cls::Tuple,
            Ty::Array(_, _) => Cls::Array,
            Ty::UnicodeChar => Cls::Unicode,
            Ty::List(_, _) => Cls::List,
            Ty::Set(_, _) => Cls::Set,
            Ty::Map(_, _, _) => Cls::Map,
        }
    }
}

impl KeyTy {
    pub const fn cls(&self) -> Cls {
        match self {
            KeyTy::Primitive(_) => Cls::Primitive,
            KeyTy::Enum(_) => Cls::Enum,
            KeyTy::Array(_) => Cls::Array,
            KeyTy::UnicodeStr(_) => Cls::Unicode,
            KeyTy::AsciiStr(_) => Cls::AsciiStr,
            KeyTy::Bytes(_) => Cls::List,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub enum Ty<Ref: TypeRef> {
    #[strict_type(tag = 0)]
    #[from]
    Primitive(Primitive),

    /// We use separate type since unlike primitive it has variable length.
    /// While unicode character can be expressed as a composite type, it will be very verbose
    /// expression (union with 256 variants), so instead we built it in.
    #[strict_type(tag = 1, rename = "unicode", dumb)]
    UnicodeChar,

    #[strict_type(tag = 3)]
    #[from]
    Enum(EnumVariants),

    #[strict_type(tag = 4)]
    #[from]
    Union(UnionVariants<Ref>),

    #[strict_type(tag = 5)]
    #[from]
    Tuple(UnnamedFields<Ref>),

    #[strict_type(tag = 6)]
    #[from]
    Struct(NamedFields<Ref>),

    #[strict_type(tag = 7)]
    Array(Ref, u16),

    #[strict_type(tag = 8)]
    List(Ref, Sizing),

    #[strict_type(tag = 9)]
    Set(Ref, Sizing),

    #[strict_type(tag = 10)]
    Map(KeyTy, Ref, Sizing),
}

impl<Ref: TypeRef> Ty<Ref> {
    pub const UNIT: Ty<Ref> = Ty::Primitive(UNIT);
    pub const BYTE: Ty<Ref> = Ty::Primitive(BYTE);

    pub const U8: Ty<Ref> = Ty::Primitive(U8);
    pub const U16: Ty<Ref> = Ty::Primitive(U16);
    pub const U24: Ty<Ref> = Ty::Primitive(U24);
    pub const U32: Ty<Ref> = Ty::Primitive(U32);
    pub const U64: Ty<Ref> = Ty::Primitive(U64);
    pub const U128: Ty<Ref> = Ty::Primitive(U128);
    pub const U256: Ty<Ref> = Ty::Primitive(U256);
    pub const U512: Ty<Ref> = Ty::Primitive(U512);
    pub const U1024: Ty<Ref> = Ty::Primitive(U1024);

    pub const I8: Ty<Ref> = Ty::Primitive(I8);
    pub const I16: Ty<Ref> = Ty::Primitive(I16);
    pub const I24: Ty<Ref> = Ty::Primitive(I24);
    pub const I32: Ty<Ref> = Ty::Primitive(I32);
    pub const I64: Ty<Ref> = Ty::Primitive(I64);
    pub const I128: Ty<Ref> = Ty::Primitive(I128);
    pub const I256: Ty<Ref> = Ty::Primitive(I256);
    pub const I512: Ty<Ref> = Ty::Primitive(I512);
    pub const I1024: Ty<Ref> = Ty::Primitive(I1024);

    pub const F16B: Ty<Ref> = Ty::Primitive(F16B);
    pub const F16: Ty<Ref> = Ty::Primitive(F16);
    pub const F32: Ty<Ref> = Ty::Primitive(F32);
    pub const F64: Ty<Ref> = Ty::Primitive(F64);
    pub const F80: Ty<Ref> = Ty::Primitive(F80);
    pub const F128: Ty<Ref> = Ty::Primitive(F128);
    pub const F256: Ty<Ref> = Ty::Primitive(F256);

    pub const UNICODE: Ty<Ref> = Ty::UnicodeChar;

    pub fn enumerate(variants: EnumVariants) -> Self { Ty::Enum(variants) }
    pub fn union(variants: UnionVariants<Ref>) -> Self { Ty::Union(variants) }
    pub fn struc(fields: NamedFields<Ref>) -> Self { Ty::Struct(fields) }
    pub fn tuple(fields: UnnamedFields<Ref>) -> Self { Ty::Tuple(fields) }

    pub fn list(ty: Ref, sizing: Sizing) -> Self { Ty::List(ty, sizing) }
    pub fn set(ty: Ref, sizing: Sizing) -> Self { Ty::Set(ty, sizing) }
    pub fn map(key: KeyTy, val: Ref, sizing: Sizing) -> Self { Ty::Map(key, val, sizing) }

    pub fn ascii_char() -> Self { Ty::Enum(variants!(32..=127)) }

    pub fn is_compound(&self) -> bool {
        match self {
            Ty::Tuple(fields) if fields.len() > 1 => true,
            Ty::Struct(fields) if fields.len() > 1 => true,
            Ty::Enum(_) | Ty::Union(_) => true,
            _ => false,
        }
    }
    pub fn is_primitive(&self) -> bool { matches!(self, Ty::Primitive(_) | Ty::UnicodeChar) }
    pub fn is_collection(&self) -> bool {
        matches!(self, Ty::Array(..) | Ty::List(..) | Ty::Set(..) | Ty::Map(..))
    }
    pub fn is_option(&self) -> bool {
        matches!(self,
            Ty::Union(variants) if variants.len() == 2
            && variants.first_key_value().unwrap().0 == &Variant { name: fname!("none"), tag: 0 }
            && variants.last_key_value().unwrap().0 == &Variant { name: fname!("some"), tag: 1 }
        )
    }
}

impl<Ref: TypeRef> Display for Ty<Ref>
where Ref: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Primitive(prim) => Display::fmt(prim, f),
            Ty::Enum(_) if self.is_ascii_char() => f.write_str("Ascii"),
            Ty::Enum(vars) => Display::fmt(vars, f),
            Ty::Union(fields) if self.is_option() => {
                let variant = fields.get(&Variant::some()).expect("optional");
                Display::fmt(variant, f)?;
                f.write_str("?")
            }
            Ty::Union(fields) => Display::fmt(fields, f),
            Ty::Struct(fields) => Display::fmt(fields, f),
            Ty::Tuple(fields) => Display::fmt(fields, f),
            Ty::Array(ty, len) => {
                f.write_str("[")?;
                Display::fmt(ty, f)?;
                write!(f, " ^ {len}]")
            }
            Ty::UnicodeChar => f.write_str("Unicode"),
            Ty::List(ty, sizing) => {
                f.write_str("[")?;
                Display::fmt(ty, f)?;
                write!(f, "{sizing}]")
            }
            Ty::Set(ty, sizing) => {
                f.write_str("{")?;
                Display::fmt(ty, f)?;
                write!(f, "{sizing}}}")
            }
            Ty::Map(key, ty, sizing) => {
                write!(f, "{{{key} ->{sizing} ")?;
                Display::fmt(ty, f)?;
                f.write_str("}")
            }
        }
    }
}

impl<Ref: TypeRef> Ty<Ref> {
    pub fn ty_at(&self, pos: u8) -> Option<&Ref> {
        match self {
            Ty::Union(fields) => fields.ty_by_tag(pos),
            Ty::Struct(fields) => fields.ty_by_pos(pos),
            Ty::Tuple(fields) => fields.ty_by_pos(pos),
            Ty::Array(ty, _) | Ty::List(ty, _) | Ty::Set(ty, _) | Ty::Map(_, ty, _) if pos > 0 => {
                Some(ty)
            }
            _ => None,
        }
    }

    pub fn is_byte(&self) -> bool { matches!(self, x if x == &Ty::BYTE || x == &Ty::U8) }
    pub fn is_unicode_char(&self) -> bool { matches!(self, x if x == &Ty::UNICODE) }
    pub fn is_ascii_char(&self) -> bool { matches!(self, x if x == &Ty::ascii_char()) }

    pub fn try_to_key(&self) -> Result<KeyTy, &Ty<Ref>> {
        Ok(match self {
            Ty::Primitive(code) => KeyTy::Primitive(*code),
            Ty::Enum(vars) => KeyTy::Enum(vars.clone()),
            Ty::Array(ty, len) if ty.is_byte() => KeyTy::Array(*len),
            Ty::Array(ty, len) if ty.is_unicode_char() => {
                KeyTy::UnicodeStr(Sizing::fixed(*len as u64))
            }
            Ty::List(ty, sizing) if ty.is_byte() => KeyTy::Bytes(*sizing),
            Ty::List(ty, sizing) if ty.is_ascii_subset() => KeyTy::Bytes(*sizing),
            Ty::List(ty, sizing) if ty.is_unicode_char() => KeyTy::UnicodeStr(*sizing),
            Ty::List(ty, sizing) if ty.is_ascii_char() => KeyTy::AsciiStr(*sizing),
            Ty::UnicodeChar => KeyTy::UnicodeStr(Sizing::ONE),
            Ty::Union(_)
            | Ty::Struct(_)
            | Ty::Tuple(_)
            | Ty::Array(_, _)
            | Ty::List(_, _)
            | Ty::Set(_, _)
            | Ty::Map(_, _, _) => return Err(self),
        })
    }
}

/// Lexicographically sortable types which may serve as map keys.
///
/// The type is always guaranteed to fit strict encoding AST serialization
/// bounds since it doesn't has a dynamically-sized types.
#[derive(Clone, PartialEq, Eq, Debug, Display, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom, dumb = { KeyTy::Array(1) })]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[display(inner)]
pub enum KeyTy {
    #[strict_type(tag = 0)]
    #[from]
    Primitive(Primitive),

    #[strict_type(tag = 3)]
    #[display("({0})")]
    #[from]
    Enum(EnumVariants),

    /// Fixed-size byte array
    #[strict_type(tag = 7)]
    #[display("[Byte ^ {0}]")]
    #[from]
    Array(u16),

    #[strict_type(tag = 0x10, rename = "unicode")]
    #[display("[Unicode{0}]")]
    UnicodeStr(Sizing),

    #[strict_type(tag = 0x11, rename = "ascii")]
    #[display("[Std.Ascii{0}]")]
    AsciiStr(Sizing),

    #[strict_type(tag = 0x12)]
    #[display("[Byte{0}]")]
    Bytes(Sizing),
}

impl KeyTy {
    pub const U8: KeyTy = KeyTy::Primitive(U8);
    pub const BYTE: KeyTy = KeyTy::Primitive(BYTE);

    pub fn to_ty<Ref: PrimitiveRef>(&self) -> Ty<Ref> { self.clone().into_ty() }

    pub fn into_ty<Ref: PrimitiveRef>(self) -> Ty<Ref> {
        match self {
            KeyTy::Primitive(prim) => Ty::<Ref>::Primitive(prim),
            KeyTy::Enum(variants) => Ty::<Ref>::Enum(variants),
            KeyTy::Array(len) => Ty::<Ref>::Array(Ref::byte(), len),
            KeyTy::UnicodeStr(sizing) if sizing == Sizing::ONE => Ty::<Ref>::UnicodeChar,
            KeyTy::UnicodeStr(sizing) if sizing.is_fixed() && sizing.min <= u16::MAX as u64 => {
                Ty::<Ref>::Array(Ref::unicode_char(), sizing.min as u16)
            }
            KeyTy::UnicodeStr(sizing) => Ty::<Ref>::List(Ref::unicode_char(), sizing),
            KeyTy::AsciiStr(sizing) if sizing == Sizing::ONE => Ty::ascii_char(),
            KeyTy::AsciiStr(sizing) if sizing.is_fixed() && sizing.min <= u16::MAX as u64 => {
                Ty::<Ref>::Array(Ref::ascii_char(), sizing.min as u16)
            }
            KeyTy::AsciiStr(sizing) => Ty::<Ref>::List(Ref::ascii_char(), sizing),
            KeyTy::Bytes(sizing) if sizing == Sizing::ONE => Ty::BYTE,
            KeyTy::Bytes(sizing) if sizing.is_fixed() && sizing.min <= u16::MAX as u64 => {
                Ty::<Ref>::Array(Ref::byte(), sizing.min as u16)
            }
            KeyTy::Bytes(sizing) => Ty::<Ref>::List(Ref::byte(), sizing),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Field<Ref: TypeRef> {
    pub name: FieldName,
    pub ty: Ref,
}

impl<Ref: TypeRef> Display for Field<Ref>
where Ref: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)?;
        f.write_str(" ")?;
        Display::fmt(&self.ty, f)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, dumb = fields!("dumb" => Ref::strict_dumb()))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct NamedFields<Ref: TypeRef>(Confined<Vec<Field<Ref>>, 1, { u8::MAX as usize }>);

impl<Ref: TypeRef> Wrapper for NamedFields<Ref> {
    type Inner = Confined<Vec<Field<Ref>>, 1, { u8::MAX as usize }>;

    fn from_inner(inner: Self::Inner) -> Self { Self(inner) }

    fn as_inner(&self) -> &Self::Inner { &self.0 }

    fn into_inner(self) -> Self::Inner { self.0 }
}

impl<Ref: TypeRef> Deref for NamedFields<Ref> {
    type Target = Confined<Vec<Field<Ref>>, 1, { u8::MAX as usize }>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<Ref: TypeRef> TryFrom<Vec<Field<Ref>>> for NamedFields<Ref> {
    type Error = confinement::Error;

    fn try_from(inner: Vec<Field<Ref>>) -> Result<Self, Self::Error> {
        Confined::try_from(inner).map(NamedFields::from)
    }
}

impl<Ref: TypeRef> IntoIterator for NamedFields<Ref> {
    type Item = Field<Ref>;
    type IntoIter = std::vec::IntoIter<Field<Ref>>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a, Ref: TypeRef> IntoIterator for &'a NamedFields<Ref> {
    type Item = &'a Field<Ref>;
    type IntoIter = std::slice::Iter<'a, Field<Ref>>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl<Ref: TypeRef> NamedFields<Ref> {
    pub fn into_inner(self) -> Vec<Field<Ref>> { self.0.into_inner() }

    pub fn ty_by_pos(&self, pos: u8) -> Option<&Ref> { self.0.get(pos as usize).map(|f| &f.ty) }
    pub fn ty_by_name(&self, name: &FieldName) -> Option<&Ref> {
        self.0.iter().find(|f| &f.name == name).map(|f| &f.ty)
    }
}

impl<Ref: TypeRef> Display for NamedFields<Ref>
where Ref: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.len();
        let mut iter = self.iter();
        let last = iter.next_back();
        for field in iter {
            Display::fmt(field, f)?;
            if len >= 3 {
                f.write_str("\n                       , ")?;
            } else {
                f.write_str(", ")?;
            }
        }
        if let Some(field) = last {
            Display::fmt(field, f)?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, dumb = fields!(Ref::strict_dumb()))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct UnnamedFields<Ref: TypeRef>(Confined<Vec<Ref>, 1, { u8::MAX as usize }>);

impl<Ref: TypeRef> Wrapper for UnnamedFields<Ref> {
    type Inner = Confined<Vec<Ref>, 1, { u8::MAX as usize }>;

    fn from_inner(inner: Self::Inner) -> Self { Self(inner) }

    fn as_inner(&self) -> &Self::Inner { &self.0 }

    fn into_inner(self) -> Self::Inner { self.0 }
}

impl<Ref: TypeRef> Deref for UnnamedFields<Ref> {
    type Target = Confined<Vec<Ref>, 1, { u8::MAX as usize }>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<Ref: TypeRef> TryFrom<Vec<Ref>> for UnnamedFields<Ref> {
    type Error = confinement::Error;

    fn try_from(inner: Vec<Ref>) -> Result<Self, Self::Error> {
        Confined::try_from(inner).map(UnnamedFields::from)
    }
}

impl<Ref: TypeRef> IntoIterator for UnnamedFields<Ref> {
    type Item = Ref;
    type IntoIter = std::vec::IntoIter<Ref>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a, Ref: TypeRef> IntoIterator for &'a UnnamedFields<Ref> {
    type Item = &'a Ref;
    type IntoIter = std::slice::Iter<'a, Ref>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl<Ref: TypeRef> UnnamedFields<Ref> {
    pub fn into_inner(self) -> Vec<Ref> { self.0.into_inner() }

    pub fn ty_by_pos(&self, pos: u8) -> Option<&Ref> { self.0.get(pos as usize) }
}

impl<Ref: TypeRef> Display for UnnamedFields<Ref>
where Ref: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        let last = iter.next_back();
        for ty in iter {
            Display::fmt(ty, f)?;
            f.write_str(", ")?;
        }
        if let Some(ty) = last {
            Display::fmt(ty, f)?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[derive(StrictDumb, StrictType)]
#[strict_type(lib = STRICT_TYPES_LIB, dumb = variants!("dumb" => Ref::strict_dumb()))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct UnionVariants<Ref: TypeRef>(Confined<BTreeMap<Variant, Ref>, 1, { u8::MAX as usize }>);

impl<Ref: TypeRef> Wrapper for UnionVariants<Ref> {
    type Inner = Confined<BTreeMap<Variant, Ref>, 1, { u8::MAX as usize }>;

    fn from_inner(inner: Self::Inner) -> Self { Self(inner) }

    fn as_inner(&self) -> &Self::Inner { &self.0 }

    fn into_inner(self) -> Self::Inner { self.0 }
}

impl<Ref: TypeRef> Deref for UnionVariants<Ref> {
    type Target = Confined<BTreeMap<Variant, Ref>, 1, { u8::MAX as usize }>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<Ref: TypeRef> TryFrom<BTreeMap<Variant, Ref>> for UnionVariants<Ref> {
    type Error = confinement::Error;

    fn try_from(inner: BTreeMap<Variant, Ref>) -> Result<Self, Self::Error> {
        Confined::try_from(inner).map(UnionVariants::from)
    }
}

impl<Ref: TypeRef> IntoIterator for UnionVariants<Ref> {
    type Item = (Variant, Ref);
    type IntoIter = std::collections::btree_map::IntoIter<Variant, Ref>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a, Ref: TypeRef> IntoIterator for &'a UnionVariants<Ref> {
    type Item = (&'a Variant, &'a Ref);
    type IntoIter = std::collections::btree_map::Iter<'a, Variant, Ref>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl<Ref: TypeRef> UnionVariants<Ref> {
    pub fn into_inner(self) -> BTreeMap<Variant, Ref> { self.0.into_inner() }

    pub fn into_keys(self) -> std::collections::btree_map::IntoKeys<Variant, Ref> {
        self.0.into_inner().into_keys()
    }

    pub fn into_values(self) -> std::collections::btree_map::IntoValues<Variant, Ref> {
        self.0.into_inner().into_values()
    }

    pub fn has_tag(&self, tag: u8) -> bool { self.0.keys().find(|v| v.tag == tag).is_some() }
    pub fn by_name(&self, name: &FieldName) -> Option<(&Variant, &Ref)> {
        self.0.iter().find(|(v, _)| &v.name == name)
    }
    pub fn ty_by_name(&self, name: &FieldName) -> Option<&Ref> {
        self.0.iter().find(|(v, _)| &v.name == name).map(|(_, ty)| ty)
    }
    pub fn ty_by_tag(&self, tag: u8) -> Option<&Ref> {
        self.0.iter().find(|(v, _)| v.tag == tag).map(|(_, ty)| ty)
    }
    pub fn tag_by_name(&self, name: &FieldName) -> Option<u8> {
        self.0.keys().find(|v| &v.name == name).map(|v| v.tag)
    }
    pub fn name_by_tag(&self, tag: u8) -> Option<&FieldName> {
        self.0.keys().find(|v| v.tag == tag).map(|v| &v.name)
    }
}

impl<Ref: TypeRef> Display for UnionVariants<Ref>
where Ref: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        let last = iter.next_back();
        let mut last_tag = 0u8;
        for (variant, ty) in iter {
            write!(f, "{variant}")?;
            if last_tag != variant.tag {
                last_tag = variant.tag;
                write!(f, ":{last_tag} ")?;
            } else {
                f.write_str(" ")?;
            }
            last_tag += 1;
            if ty.is_compound() {
                f.write_str("(")?;
                Display::fmt(ty, f)?;
                f.write_str(")")?;
            } else {
                Display::fmt(ty, f)?;
            }
            write!(f, "\n                       | ")?;
        }
        if let Some((variant, ty)) = last {
            write!(f, "{variant}")?;
            if last_tag != variant.tag {
                last_tag = variant.tag;
                write!(f, ":{last_tag} ")?;
            } else {
                f.write_str(" ")?;
            }
            if ty.is_compound() {
                f.write_str("(")?;
                Display::fmt(ty, f)?;
                f.write_str(")")?;
            } else {
                Display::fmt(ty, f)?;
            }
        }
        Ok(())
    }
}

#[derive(Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[wrapper(Deref)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, dumb = variants!("dumb"))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct EnumVariants(Confined<BTreeSet<Variant>, 1, { u8::MAX as usize }>);

impl TryFrom<BTreeSet<Variant>> for EnumVariants {
    type Error = confinement::Error;

    fn try_from(inner: BTreeSet<Variant>) -> Result<Self, Self::Error> {
        Confined::try_from(inner).map(EnumVariants::from)
    }
}

impl IntoIterator for EnumVariants {
    type Item = Variant;
    type IntoIter = std::collections::btree_set::IntoIter<Variant>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a> IntoIterator for &'a EnumVariants {
    type Item = &'a Variant;
    type IntoIter = std::collections::btree_set::Iter<'a, Variant>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl EnumVariants {
    pub fn into_inner(self) -> BTreeSet<Variant> { self.0.into_inner() }

    pub fn tag_by_name(&self, name: &FieldName) -> Option<u8> {
        self.0.iter().find(|v| &v.name == name).map(|v| v.tag)
    }
    pub fn name_by_tag(&self, tag: u8) -> Option<&FieldName> {
        self.0.iter().find(|v| v.tag == tag).map(|v| &v.name)
    }
    pub fn has_tag(&self, tag: u8) -> bool { self.0.iter().find(|v| v.tag == tag).is_some() }
}

impl Display for EnumVariants {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        if let Some(variant) = iter.next() {
            write!(f, "{variant:#}")?;
        }
        let mut chunk_size = None;
        loop {
            for variant in iter.by_ref().take(chunk_size.unwrap_or(3)) {
                write!(f, " | {variant:#}")?;
            }
            chunk_size = Some(4);
            if iter.len() == 0 {
                break;
            }
            write!(f, "\n                      ")?;
        }
        writeln!(f)
    }
}
