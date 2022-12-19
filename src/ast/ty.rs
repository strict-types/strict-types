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
use crate::primitive::constants::*;
use crate::util::Sizing;
use crate::{Encode, Ident, LibAlias, SemId, Serialize, StenSchema, StenType, TypeName};

pub const MAX_SERIALIZED_SIZE: usize = 1 << 24 - 1;

/// Glue for constructing ASTs.
pub trait TypeRef: StenSchema + Clone + Eq + Debug + Encode + Sized {
    fn id(&self) -> SemId;
}
pub trait NestedRef: TypeRef {
    fn as_ty(&self) -> &Ty<Self>;
    fn into_ty(self) -> Ty<Self>;
    fn iter(&self) -> Iter<Self> { Iter::from(self) }
}

impl TypeRef for StenType {
    fn id(&self) -> SemId { self.as_ty().id() }
}
impl NestedRef for StenType {
    fn as_ty(&self) -> &Ty<Self> { &self.ty }
    fn into_ty(self) -> Ty<Self> { *self.ty }
}

pub type FieldName = Ident;

#[derive(Clone, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Field {
    pub name: Option<FieldName>,
    pub ord: u8,
}

impl StenSchema for Field {
    const STEN_TYPE_NAME: &'static str = "Field";

    fn sten_ty() -> Ty<StenType> {
        Ty::<StenType>::composition(fields! {
            "name" => Option::<FieldName>::sten_type(),
            "ord" => u8::sten_type()
        })
    }
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
            write!(f, "{} ", name)?;
        }
        if f.alternate() {
            if self.name.is_some() {
                f.write_str("= ")?;
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

impl<Ref: TypeRef> StenSchema for Ty<Ref> {
    const STEN_TYPE_NAME: &'static str = "Ty";

    fn sten_ty() -> Ty<StenType> {
        Ty::union(fields! {
            "primitive" => Primitive::sten_type(),
            "unicode" => <()>::sten_type(),
            "enum" => Variants::sten_type(),
            "union" => Fields::<Ref, false>::sten_type(),
            "struct" => Fields::<Ref, true>::sten_type(),
            "array" => <(Ref, u16)>::sten_type(),
            "list" => <(Ref, Sizing)>::sten_type(),
            "set" => <(Ref, Sizing)>::sten_type(),
            "map" => <(KeyTy, Ref, Sizing)>::sten_type(),
        })
    }
}

impl<Ref: TypeRef> Ord for Ty<Ref> {
    fn cmp(&self, other: &Self) -> Ordering { self.id().cmp(&other.id()) }
}

impl<Ref: TypeRef> PartialOrd for Ty<Ref> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
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
    pub fn union(fields: Fields<Ref, false>) -> Self {
        let ty = Ty::Union(fields);
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }
    pub fn composition(fields: Fields<Ref, true>) -> Self {
        let ty = Ty::Struct(fields);
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }

    pub fn list(ty: Ref, sizing: Sizing) -> Self {
        let ty = Ty::List(ty, sizing);
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }
    pub fn set(ty: Ref, sizing: Sizing) -> Self {
        let ty = Ty::Set(ty, sizing);
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }
    pub fn map(key: KeyTy, val: Ref, sizing: Sizing) -> Self {
        let ty = Ty::Map(key, val, sizing);
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }

    pub fn ascii_char() -> Self { Ty::Enum(variants!(0..=127)) }

    pub fn is_primitive(&self) -> bool { matches!(self, Ty::Primitive(_)) }
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

impl Ty<StenType> {
    pub fn byte_array(len: u16) -> Self { Ty::Array(StenType::byte(), len) }
    pub fn bytes(sizing: Sizing) -> Self { Ty::List(StenType::byte(), sizing) }
    pub fn ascii_string(sizing: Sizing) -> Self { Ty::List(StenType::ascii(), sizing) }
    pub fn option(ty: StenType) -> Self {
        // TODO: Check for AST size
        Ty::Union(fields![
            "None" => <()>::sten_type(),
            "Some" => ty
        ])
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
            Ty::Map(key, ty, sizing) => write!(f, "{{{}{}}} -> [{}]", key, sizing, ty),
        }
    }
}

impl<Ref: NestedRef> Ty<Ref> {
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

    pub fn try_to_key(&self) -> Result<KeyTy, &Ty<Ref>> {
        Ok(match self {
            Ty::Primitive(code) => KeyTy::Primitive(*code),
            Ty::Enum(vars) => KeyTy::Enum(vars.clone()),
            Ty::Array(ty, len) if ty.as_ty() == &Ty::BYTE => KeyTy::Array(*len),
            Ty::List(ty, sizing) if ty.as_ty() == &Ty::BYTE => KeyTy::Bytes(*sizing),
            Ty::Array(ty, len) if ty.as_ty() == &Ty::UNICODE => {
                KeyTy::UnicodeStr(Sizing::fixed(*len))
            }
            Ty::List(ty, sizing) if ty.as_ty() == &Ty::UNICODE => KeyTy::UnicodeStr(*sizing),
            Ty::List(ty, sizing) if ty.as_ty() == &Ty::<Ref>::ascii_char() => {
                KeyTy::AsciiStr(*sizing)
            }
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
    Enum(Variants),
    /// Fixed-size byte array
    #[display("[Byte ^ {0}]")]
    Array(u16),
    UnicodeStr(Sizing),
    AsciiStr(Sizing),
    Bytes(Sizing),
}

impl StenSchema for KeyTy {
    const STEN_TYPE_NAME: &'static str = "KeyTy";

    fn sten_ty() -> Ty<StenType> {
        Ty::union(fields! {
            "primitive" => Primitive::sten_type(),
            "enum" => Variants::sten_type(),
            "array" => u16::sten_type(),
            "unicodeStr" => Sizing::sten_type(),
            "asciiStr" => Sizing::sten_type(),
            "bytes" => Sizing::sten_type(),
        })
    }
}

impl KeyTy {
    pub const U8: KeyTy = KeyTy::Primitive(U8);
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
pub struct Fields<Ref: TypeRef = StenType, const OP: bool = true>(
    Confined<BTreeMap<Field, Ref>, 1, { u8::MAX as usize }>,
);

impl<Ref: TypeRef, const OP: bool> StenSchema for Fields<Ref, OP> {
    const STEN_TYPE_NAME: &'static str = "Fields";

    fn sten_ty() -> Ty<StenType> {
        // TODO: Serialize according to this schema
        let val_ty = <(Option<FieldName>, Ref)>::sten_type();
        Ty::map(KeyTy::U8, val_ty, Sizing::U8_NONEMPTY)
    }
}

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
            write!(f, "{}{}{}", field, ty, sep)?;
        }
        if let Some((field, ty)) = last {
            write!(f, "{}{}", field, ty)?;
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

impl StenSchema for Variants {
    const STEN_TYPE_NAME: &'static str = "Variants";

    fn sten_ty() -> Ty<StenType> { Ty::set(Field::sten_type(), Sizing::U8_NONEMPTY) }
}

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

impl<Ref: TypeRef> StenSchema for (Ref, u16) {
    const STEN_TYPE_NAME: &'static str = "ArrayTy";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields! {
            Ref::sten_type(),
            u16::sten_type(),
        })
    }
}

impl<Ref: TypeRef> StenSchema for (Ref, Sizing) {
    const STEN_TYPE_NAME: &'static str = "ListTy";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields! {
            Ref::sten_type(),
            Sizing::sten_type(),
        })
    }
}

impl<Ref: TypeRef> StenSchema for (KeyTy, Ref, Sizing) {
    const STEN_TYPE_NAME: &'static str = "ListTy";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields! {
            KeyTy::sten_type(),
            Ref::sten_type(),
            Sizing::sten_type(),
        })
    }
}

impl<Ref: TypeRef> StenSchema for (Option<FieldName>, Ref) {
    const STEN_TYPE_NAME: &'static str = "FieldTy";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields! {
            Option::<FieldName>::sten_type(),
            Ref::sten_type(),
        })
    }
}

impl StenSchema for (TypeName, SemId) {
    const STEN_TYPE_NAME: &'static str = "TypeDef";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields!(TypeName::sten_type(), SemId::sten_type()))
    }
}

impl StenSchema for (TypeName, LibAlias, SemId) {
    const STEN_TYPE_NAME: &'static str = "TypeDefFull";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields!(TypeName::sten_type(), LibAlias::sten_type(), SemId::sten_type()))
    }
}
