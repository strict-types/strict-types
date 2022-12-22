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

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Deref;

use amplify::confinement::Confined;
use amplify::{confinement, Wrapper};

use crate::ast::Iter;
use crate::encoding::StrictEncode;
use crate::primitive::constants::*;
use crate::util::Sizing;
use crate::{Ident, SemId};

/// Glue for constructing ASTs.
pub trait TypeRef: Clone + StrictEncode<Dumb = Self> + Eq + Debug + Sized {
    const TYPE_NAME: &'static str;
    fn id(&self) -> SemId;
    fn is_byte(&self) -> bool { false }
    fn is_unicode_char(&self) -> bool { false }
    fn is_ascii_char(&self) -> bool { false }
}
// TODO: None of the Ref-types implements this, but a lot of implementations on `Ty` are only for
//       RecursiveRef's. Check how this can be improved
pub trait RecursiveRef: TypeRef {
    fn as_ty(&self) -> &Ty<Self>;
    fn into_ty(self) -> Ty<Self>;
    fn iter(&self) -> Iter<Self> { Iter::from(self) }
}

impl TypeRef for SemId {
    const TYPE_NAME: &'static str = "SemId";
    fn id(&self) -> SemId { *self }
}

impl TypeRef for KeyTy {
    const TYPE_NAME: &'static str = "KeyTy";
    fn id(&self) -> SemId { KeyTy::id(self) }
}

pub type FieldName = Ident;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
#[display(lowercase)]
#[repr(u8)]
pub enum Cls {
    Primitive = 0,
    UnicodeChar = 1,
    AsciiStr = 2,
    Enum = 3,
    Union = 4,
    Struct = 5,
    Array = 6,
    List = 7,
    Set = 8,
    Map = 9,
}

impl Cls {
    pub const ALL: [Cls; 10] = [
        Cls::Primitive,
        Cls::UnicodeChar,
        Cls::AsciiStr,
        Cls::Enum,
        Cls::Union,
        Cls::Struct,
        Cls::Array,
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

impl<Ref: TypeRef> Ty<Ref> {
    pub const fn cls(&self) -> Cls {
        match self {
            Ty::Primitive(_) => Cls::Primitive,
            Ty::Enum(_) => Cls::Enum,
            Ty::Union(_) => Cls::Union,
            Ty::Struct(_) => Cls::Struct,
            Ty::Array(_, _) => Cls::Array,
            Ty::UnicodeChar => Cls::UnicodeChar,
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
            KeyTy::UnicodeStr(_) => Cls::UnicodeChar,
            KeyTy::AsciiStr(_) => Cls::AsciiStr,
            KeyTy::Bytes(_) => Cls::List,
        }
    }
}

#[derive(Clone, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Field {
    pub name: Option<FieldName>,
    pub ord: u8,
}

impl Field {
    pub fn named(name: FieldName, value: u8) -> Field {
        Field {
            name: Some(name),
            ord: value,
        }
    }
    pub fn unnamed(value: u8) -> Field {
        Field {
            name: None,
            ord: value,
        }
    }

    pub fn none() -> Field {
        Field {
            name: Some(FieldName::from("None")),
            ord: 0,
        }
    }
    pub fn some() -> Field {
        Field {
            name: Some(FieldName::from("Some")),
            ord: 1,
        }
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        match (&self.name, &other.name) {
            (None, None) => self.ord == other.ord,
            (Some(name1), Some(name2)) => name1 == name2 || self.ord == other.ord,
            _ => false,
        }
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        self.ord.cmp(&other.ord)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}", name)?;
        }
        if f.alternate() {
            if self.name.is_some() {
                f.write_str(" = ")?;
            }
            Display::fmt(&self.ord, f)?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, From)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub enum Ty<Ref: TypeRef> {
    Primitive(Primitive),
    /// We use separate type since unlike primitive it has variable length.
    /// While unicode character can be expressed as a composite type, it will be very verbose
    /// expression (union with 256 variants), so instead we built it in.
    UnicodeChar,
    Enum(Variants),
    Union(Fields<Ref, false>),
    Struct(Fields<Ref, true>),
    Array(Ref, u16),
    List(Ref, Sizing),
    Set(Ref, Sizing),
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

    pub fn enumerate(variants: Variants) -> Self { Ty::Enum(variants) }
    pub fn union(fields: Fields<Ref, false>) -> Self { Ty::Union(fields) }
    pub fn composition(fields: Fields<Ref, true>) -> Self { Ty::Struct(fields) }

    pub fn list(ty: Ref, sizing: Sizing) -> Self { Ty::List(ty, sizing) }
    pub fn set(ty: Ref, sizing: Sizing) -> Self { Ty::Set(ty, sizing) }
    pub fn map(key: KeyTy, val: Ref, sizing: Sizing) -> Self { Ty::Map(key, val, sizing) }

    pub fn ascii_char() -> Self { Ty::Enum(variants!(0..=127)) }

    pub fn is_primitive(&self) -> bool { matches!(self, Ty::Primitive(_) | Ty::UnicodeChar) }
    pub fn is_collection(&self) -> bool {
        matches!(self, Ty::Array(..) | Ty::List(..) | Ty::Set(..) | Ty::Map(..))
    }
    pub fn is_compound(&self) -> bool {
        matches!(self, Ty::Struct(fields)
            if fields.len() > 1
            || fields.keys().next().expect("always at least one field").name.is_some())
            || (matches!(self, Ty::Enum(_) | Ty::Union(_) | Ty::Map(..)) && !self.is_option())
    }
    pub fn is_option(&self) -> bool {
        matches!(self,
            Ty::Union(fields) if fields.len() == 2
            && fields.contains_key(&Field::none())
            && fields.contains_key(&Field::some())
        )
    }

    pub fn into_union_fields(self) -> Option<Fields<Ref, false>> {
        match self {
            Ty::Union(fields) => Some(fields),
            _ => None,
        }
    }

    pub fn into_struct_fields(self) -> Option<Fields<Ref, true>> {
        match self {
            Ty::Struct(fields) => Some(fields),
            _ => None,
        }
    }

    pub fn into_enum_variants(self) -> Option<Variants> {
        match self {
            Ty::Enum(variants) => Some(variants),
            _ => None,
        }
    }

    pub fn as_union_fields(&self) -> Option<&Fields<Ref, false>> {
        match self {
            Ty::Union(ref fields) => Some(fields),
            _ => None,
        }
    }

    pub fn as_struct_fields(&self) -> Option<&Fields<Ref, true>> {
        match self {
            Ty::Struct(ref fields) => Some(fields),
            _ => None,
        }
    }

    pub fn as_enum_variants(&self) -> Option<&Variants> {
        match self {
            Ty::Enum(ref variants) => Some(variants),
            _ => None,
        }
    }
}

impl<Ref: TypeRef> Display for Ty<Ref>
where Ref: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Primitive(prim) => Display::fmt(prim, f),
            Ty::Enum(vars) => Display::fmt(vars, f),
            Ty::Union(fields) if self.is_option() => {
                write!(f, "{}?", fields.get(&Field::some()).expect("optional"))
            }
            Ty::Union(fields) => Display::fmt(fields, f),
            Ty::Struct(fields) => Display::fmt(fields, f),
            Ty::Array(ty, len) => write!(f, "[{} ^ {}]", ty, len),
            Ty::UnicodeChar => write!(f, "Unicode"),
            Ty::List(ty, sizing) => write!(f, "[{}{}]", ty, sizing),
            Ty::Set(ty, sizing) => write!(f, "{{{}{}}}", ty, sizing),
            Ty::Map(key, ty, sizing) => write!(f, "{{{} ->{} {}}}", key, sizing, ty),
        }
    }
}

impl<Ref: RecursiveRef> Ty<Ref> {
    pub fn ty_at(&self, pos: u8) -> Option<&Ref> {
        match self {
            Ty::Union(fields) => fields.ty_at(pos),
            Ty::Struct(fields) => fields.ty_at(pos),
            Ty::Array(ty, _) | Ty::List(ty, _) | Ty::Set(ty, _) | Ty::Map(_, ty, _) if pos > 0 => {
                Some(ty)
            }
            _ => return None,
        }
    }
}

impl<Ref: TypeRef> Ty<Ref> {
    pub fn is_byte(&self) -> bool { matches!(self, x if x == &Ty::BYTE) }
    pub fn is_unicode_char(&self) -> bool { matches!(self, x if x == &Ty::UNICODE) }
    pub fn is_ascii_char(&self) -> bool { matches!(self, x if x == &Ty::ascii_char()) }

    pub fn try_to_key(&self) -> Result<KeyTy, &Ty<Ref>> {
        Ok(match self {
            Ty::Primitive(code) => KeyTy::Primitive(*code),
            Ty::Enum(vars) => KeyTy::Enum(vars.clone()),
            Ty::Array(ty, len) if ty.is_byte() => KeyTy::Array(*len),
            Ty::List(ty, sizing) if ty.is_byte() => KeyTy::Bytes(*sizing),
            Ty::Array(ty, len) if ty.is_unicode_char() => KeyTy::UnicodeStr(Sizing::fixed(*len)),
            Ty::List(ty, sizing) if ty.is_unicode_char() => KeyTy::UnicodeStr(*sizing),
            Ty::List(ty, sizing) if ty.is_ascii_char() => KeyTy::AsciiStr(*sizing),
            Ty::UnicodeChar => KeyTy::UnicodeStr(Sizing::ONE),
            Ty::Union(_)
            | Ty::Struct(_)
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
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[display(inner)]
pub enum KeyTy {
    Primitive(Primitive),
    #[display("({0})")]
    Enum(Variants),
    /// Fixed-size byte array
    #[display("[Byte ^ {0}]")]
    Array(u16),
    #[display("[Unicode{0}]")]
    UnicodeStr(Sizing),
    #[display("[Ascii{0}]")]
    AsciiStr(Sizing),
    #[display("[Byte{0}]")]
    Bytes(Sizing),
}

impl KeyTy {
    pub const U8: KeyTy = KeyTy::Primitive(U8);
    pub const BYTE: KeyTy = KeyTy::Primitive(BYTE);
}

/*
TODO: Use when const expression generics will arrive
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[repr(u8)]
pub enum Composition {
    #[display(" | ")]
    Add = 0,
    #[display(", ")]
    Mul = 1,
}
*/

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Fields<Ref: TypeRef, const OP: bool = true>(
    Confined<BTreeMap<Field, Ref>, 1, { u8::MAX as usize }>,
);

impl<Ref: TypeRef, const OP: bool> Deref for Fields<Ref, OP> {
    type Target = Confined<BTreeMap<Field, Ref>, 1, { u8::MAX as usize }>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<Ref: TypeRef, const OP: bool> TryFrom<BTreeMap<Field, Ref>> for Fields<Ref, OP> {
    type Error = confinement::Error;

    fn try_from(inner: BTreeMap<Field, Ref>) -> Result<Self, Self::Error> {
        Confined::try_from(inner).map(Fields::from)
    }
}

impl<Ref: TypeRef, const OP: bool> IntoIterator for Fields<Ref, OP> {
    type Item = (Field, Ref);
    type IntoIter = std::collections::btree_map::IntoIter<Field, Ref>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a, Ref: TypeRef, const OP: bool> IntoIterator for &'a Fields<Ref, OP> {
    type Item = (&'a Field, &'a Ref);
    type IntoIter = std::collections::btree_map::Iter<'a, Field, Ref>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl<Ref: TypeRef, const OP: bool> Fields<Ref, OP> {
    pub fn into_inner(self) -> BTreeMap<Field, Ref> { self.0.into_inner() }

    pub fn into_keys(self) -> std::collections::btree_map::IntoKeys<Field, Ref> {
        self.0.into_inner().into_keys()
    }

    pub fn into_values(self) -> std::collections::btree_map::IntoValues<Field, Ref> {
        self.0.into_inner().into_values()
    }

    pub fn ty_at(&self, pos: u8) -> Option<&Ref> { self.values().skip(pos as usize).next() }
}

impl<Ref: TypeRef, const OP: bool> Display for Fields<Ref, OP>
where Ref: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let sep = if OP { ", " } else { " | " };
        let mut iter = self.iter();
        let last = iter.next_back();
        for (field, ty) in iter {
            if field.name.is_some() {
                write!(f, "{} ", field)?;
            }
            write!(f, "{}{}", ty, sep)?;
        }
        if let Some((field, ty)) = last {
            if field.name.is_some() {
                write!(f, "{} ", field)?;
            }
            write!(f, "{}", ty)?;
        }
        Ok(())
    }
}

#[derive(Wrapper, WrapperMut, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Variants(Confined<BTreeSet<Field>, 1, { u8::MAX as usize }>);

impl TryFrom<BTreeSet<Field>> for Variants {
    type Error = confinement::Error;

    fn try_from(inner: BTreeSet<Field>) -> Result<Self, Self::Error> {
        Confined::try_from(inner).map(Variants::from)
    }
}

impl IntoIterator for Variants {
    type Item = Field;
    type IntoIter = std::collections::btree_set::IntoIter<Field>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a> IntoIterator for &'a Variants {
    type Item = &'a Field;
    type IntoIter = std::collections::btree_set::Iter<'a, Field>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl Variants {
    pub fn into_inner(self) -> BTreeSet<Field> { self.0.into_inner() }
}

impl Display for Variants {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        let last = iter.next_back();
        for field in iter {
            write!(f, "{:#} | ", field)?;
        }
        if let Some(field) = last {
            write!(f, "{:#}", field)?;
        }
        Ok(())
    }
}
