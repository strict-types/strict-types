// Strict encoding schema library, implementing validation and parsing of strict encoded data
// against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2019-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2024-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2022-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Deref;

use amplify::confinement::{Confined, NonEmptyOrdMap, NonEmptyOrdSet, NonEmptyVec};
use amplify::{confinement, Wrapper};
use encoding::VariantName;
use strict_encoding::{
    FieldName, Primitive, Sizing, StrictDecode, StrictDumb, StrictEncode, Variant, STRICT_TYPES_LIB,
};

use super::id::SemCommit;
use crate::ast::Iter;

/// Glue for constructing ASTs.
pub trait TypeRef:
    SemCommit + Clone + StrictEncode + StrictDecode + StrictDumb + Eq + Debug + Sized
{
    fn as_ty(&self) -> Option<&Ty<Self>> { None }
    fn type_refs(&self) -> Iter<Self> { Iter::from(self) }

    fn is_compound(&self) -> bool { false }
    fn is_byte(&self) -> bool { false }
    fn is_unicode_char(&self) -> bool { false }
}

pub trait PrimitiveRef: TypeRef {
    fn byte() -> Self;
    fn unicode_char() -> Self;
}

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

#[derive(Clone, PartialEq, Eq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    Map(Ref, Ref, Sizing),
}

impl<Ref: TypeRef> Ty<Ref> {
    pub const UNIT: Ty<Ref> = Ty::Primitive(Primitive::UNIT);
    pub const BYTE: Ty<Ref> = Ty::Primitive(Primitive::BYTE);

    pub const U8: Ty<Ref> = Ty::Primitive(Primitive::U8);
    pub const U16: Ty<Ref> = Ty::Primitive(Primitive::U16);
    pub const U24: Ty<Ref> = Ty::Primitive(Primitive::U24);
    pub const U32: Ty<Ref> = Ty::Primitive(Primitive::U32);
    pub const U40: Ty<Ref> = Ty::Primitive(Primitive::U40);
    pub const U48: Ty<Ref> = Ty::Primitive(Primitive::U48);
    pub const U56: Ty<Ref> = Ty::Primitive(Primitive::U56);
    pub const U64: Ty<Ref> = Ty::Primitive(Primitive::U64);
    pub const U128: Ty<Ref> = Ty::Primitive(Primitive::U128);
    pub const U256: Ty<Ref> = Ty::Primitive(Primitive::U256);
    pub const U512: Ty<Ref> = Ty::Primitive(Primitive::U512);
    pub const U1024: Ty<Ref> = Ty::Primitive(Primitive::U1024);

    pub const I8: Ty<Ref> = Ty::Primitive(Primitive::I8);
    pub const I16: Ty<Ref> = Ty::Primitive(Primitive::I16);
    pub const I24: Ty<Ref> = Ty::Primitive(Primitive::I24);
    pub const I32: Ty<Ref> = Ty::Primitive(Primitive::I32);
    pub const I64: Ty<Ref> = Ty::Primitive(Primitive::I64);
    pub const I128: Ty<Ref> = Ty::Primitive(Primitive::I128);
    pub const I256: Ty<Ref> = Ty::Primitive(Primitive::I256);
    pub const I512: Ty<Ref> = Ty::Primitive(Primitive::I512);
    pub const I1024: Ty<Ref> = Ty::Primitive(Primitive::I1024);

    pub const F16B: Ty<Ref> = Ty::Primitive(Primitive::F16B);
    pub const F16: Ty<Ref> = Ty::Primitive(Primitive::F16);
    pub const F32: Ty<Ref> = Ty::Primitive(Primitive::F32);
    pub const F64: Ty<Ref> = Ty::Primitive(Primitive::F64);
    pub const F80: Ty<Ref> = Ty::Primitive(Primitive::F80);
    pub const F128: Ty<Ref> = Ty::Primitive(Primitive::F128);
    pub const F256: Ty<Ref> = Ty::Primitive(Primitive::F256);

    pub const UNICODE: Ty<Ref> = Ty::UnicodeChar;

    pub fn enumerate(variants: EnumVariants) -> Self { Ty::Enum(variants) }
    pub fn union(variants: UnionVariants<Ref>) -> Self { Ty::Union(variants) }
    pub fn struc(fields: NamedFields<Ref>) -> Self { Ty::Struct(fields) }
    pub fn tuple(fields: UnnamedFields<Ref>) -> Self { Ty::Tuple(fields) }

    pub fn list(ty: Ref, sizing: Sizing) -> Self { Ty::List(ty, sizing) }
    pub fn set(ty: Ref, sizing: Sizing) -> Self { Ty::Set(ty, sizing) }
    pub fn map(key: Ref, val: Ref, sizing: Sizing) -> Self { Ty::Map(key, val, sizing) }

    pub fn is_char_enum(&self) -> bool {
        if let Ty::Tuple(fields) = self {
            fields.first().and_then(Ref::as_ty).map(Self::is_char_enum).unwrap_or_default()
        } else if let Ty::Enum(variants) = self {
            variants.iter().all(|variant| (32..=127).contains(&variant.tag))
        } else {
            false
        }
    }
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

    pub fn is_newtype(&self) -> bool { matches!(self, Ty::Tuple(fields) if fields.len() == 1) }
    pub fn is_byte_array(&self) -> bool { matches!(self, Ty::Array(ty, _) if ty.is_byte()) }
    pub fn is_option(&self) -> bool { self.as_some().is_some() }
    pub fn as_some(&self) -> Option<&Ref> {
        match self {
            Ty::Union(variants)
                if variants.len() == 2
                    && variants.unwrap_first().name == vname!("none")
                    && variants.unwrap_first().tag == 0
                    && variants.unwrap_last().name == vname!("some")
                    && variants.unwrap_last().tag == 1 =>
            {
                Some(variants.last_key_value().unwrap().1)
            }
            _ => None,
        }
    }

    pub fn as_wrapped_ty(&self) -> Option<&Ty<Ref>> {
        if let Ty::Tuple(fields) = self {
            if fields.len() == 1 {
                if let Some(inner) = fields.first() {
                    return inner.as_ty();
                }
            }
        }
        None
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

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom)]
pub enum ItemCase {
    #[strict_type(tag = 0)]
    UnnamedField(u8),
    #[strict_type(tag = 1)]
    NamedField(u8, FieldName),
    #[strict_type(tag = 2)]
    UnionVariant(u8, VariantName),
    #[strict_type(tag = 0x10, dumb)]
    ArrayItem,
    #[strict_type(tag = 0x11)]
    ListItem,
    #[strict_type(tag = 0x12)]
    SetItem,
    #[strict_type(tag = 0x13)]
    MapKey,
    #[strict_type(tag = 0x14)]
    MapValue,
}

impl<Ref: TypeRef> Ty<Ref> {
    pub fn ty_at(&self, pos: u8) -> Option<&Ref> {
        match self {
            Ty::Union(fields) => fields.ty_by_pos(pos),
            Ty::Struct(fields) => fields.ty_by_pos(pos),
            Ty::Tuple(fields) => fields.ty_by_pos(pos),
            Ty::Array(ty, _) | Ty::List(ty, _) | Ty::Set(ty, _) | Ty::Map(ty, _, _) if pos == 0 => {
                Some(ty)
            }
            Ty::Map(_, ty, _) if pos == 1 => Some(ty),
            _ => None,
        }
    }
    pub fn case_at(&self, pos: u8) -> Option<ItemCase> {
        match self {
            Ty::Union(fields) => {
                fields.name_by_pos(pos).map(|name| ItemCase::UnionVariant(pos, name.clone()))
            }
            Ty::Struct(fields) => {
                fields.get(pos as usize).map(|field| ItemCase::NamedField(pos, field.name.clone()))
            }
            Ty::Tuple(fields) if fields.len_u8() == pos => Some(ItemCase::UnnamedField(pos)),
            Ty::Array(_, _) if pos == 0 => Some(ItemCase::ArrayItem),
            Ty::List(_, _) if pos == 0 => Some(ItemCase::ListItem),
            Ty::Set(_, _) if pos == 0 => Some(ItemCase::SetItem),
            Ty::Map(_, _, _) if pos == 0 => Some(ItemCase::MapKey),
            Ty::Map(_, _, _) if pos == 1 => Some(ItemCase::MapValue),
            _ => None,
        }
    }

    pub fn is_byte(&self) -> bool { matches!(self, x if x == &Ty::BYTE || x == &Ty::U8) }
    pub fn is_unicode_char(&self) -> bool { matches!(self, x if x == &Ty::UNICODE) }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct NamedFields<Ref: TypeRef>(NonEmptyVec<Field<Ref>, { u8::MAX as usize }>);

impl<Ref: TypeRef> Wrapper for NamedFields<Ref> {
    type Inner = NonEmptyVec<Field<Ref>, { u8::MAX as usize }>;

    fn from_inner(inner: Self::Inner) -> Self { Self(inner) }

    fn as_inner(&self) -> &Self::Inner { &self.0 }

    fn into_inner(self) -> Self::Inner { self.0 }
}

impl<Ref: TypeRef> Deref for NamedFields<Ref> {
    type Target = NonEmptyVec<Field<Ref>, { u8::MAX as usize }>;

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
    pub fn into_inner(self) -> Vec<Field<Ref>> { self.0.release() }

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct UnnamedFields<Ref: TypeRef>(NonEmptyVec<Ref, { u8::MAX as usize }>);

impl<Ref: TypeRef> Wrapper for UnnamedFields<Ref> {
    type Inner = NonEmptyVec<Ref, { u8::MAX as usize }>;

    fn from_inner(inner: Self::Inner) -> Self { Self(inner) }

    fn as_inner(&self) -> &Self::Inner { &self.0 }

    fn into_inner(self) -> Self::Inner { self.0 }
}

impl<Ref: TypeRef> Deref for UnnamedFields<Ref> {
    type Target = NonEmptyVec<Ref, { u8::MAX as usize }>;

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
    pub fn into_inner(self) -> Vec<Ref> { self.0.release() }

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct UnionVariants<Ref: TypeRef>(NonEmptyOrdMap<Variant, Ref, { u8::MAX as usize }>);

impl<Ref: TypeRef> Wrapper for UnionVariants<Ref> {
    type Inner = NonEmptyOrdMap<Variant, Ref, { u8::MAX as usize }>;

    fn from_inner(inner: Self::Inner) -> Self { Self(inner) }

    fn as_inner(&self) -> &Self::Inner { &self.0 }

    fn into_inner(self) -> Self::Inner { self.0 }
}

impl<Ref: TypeRef> Deref for UnionVariants<Ref> {
    type Target = NonEmptyOrdMap<Variant, Ref, { u8::MAX as usize }>;

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
    pub fn into_inner(self) -> BTreeMap<Variant, Ref> { self.0.release() }

    pub fn unwrap_first(&self) -> &Variant { self.0.first_key_value().unwrap().0 }
    pub fn unwrap_last(&self) -> &Variant { self.0.last_key_value().unwrap().0 }

    pub fn into_keys(self) -> std::collections::btree_map::IntoKeys<Variant, Ref> {
        self.0.release().into_keys()
    }

    pub fn into_values(self) -> std::collections::btree_map::IntoValues<Variant, Ref> {
        self.0.release().into_values()
    }

    pub fn has_tag(&self, tag: u8) -> bool { self.0.keys().any(|v| v.tag == tag) }
    pub fn by_tag(&self, tag: u8) -> Option<(&Variant, &Ref)> {
        self.0.iter().find(|(v, _)| v.tag == tag)
    }
    pub fn by_name(&self, name: &VariantName) -> Option<(&Variant, &Ref)> {
        self.0.iter().find(|(v, _)| &v.name == name)
    }
    pub fn ty_by_name(&self, name: &VariantName) -> Option<&Ref> {
        self.0.iter().find(|(v, _)| &v.name == name).map(|(_, ty)| ty)
    }
    pub fn ty_by_tag(&self, tag: u8) -> Option<&Ref> {
        self.0.iter().find(|(v, _)| v.tag == tag).map(|(_, ty)| ty)
    }
    pub fn ty_by_pos(&self, pos: u8) -> Option<&Ref> { self.0.values().nth(pos as usize) }
    pub fn tag_by_name(&self, name: &VariantName) -> Option<u8> {
        self.0.keys().find(|v| &v.name == name).map(|v| v.tag)
    }
    pub fn name_by_tag(&self, tag: u8) -> Option<&VariantName> {
        self.0.keys().find(|v| v.tag == tag).map(|v| &v.name)
    }
    pub fn name_by_pos(&self, pos: u8) -> Option<&VariantName> {
        self.0.keys().nth(pos as usize).map(|v| &v.name)
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
                write!(f, "#{last_tag} ")?;
            } else {
                f.write_str(" ")?;
            }
            last_tag = last_tag.saturating_add(1);
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
                write!(f, "#{last_tag} ")?;
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
            if self.len() == 1 {
                f.write_str(" | (|)")?;
            }
        } else {
            f.write_str("(|)")?;
        }
        Ok(())
    }
}

#[derive(Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[wrapper(Deref)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, dumb = variants!("dumb"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct EnumVariants(NonEmptyOrdSet<Variant, { u8::MAX as usize }>);

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
    pub fn into_inner(self) -> BTreeSet<Variant> { self.0.release() }

    pub fn tag_by_name(&self, name: &VariantName) -> Option<u8> {
        self.0.iter().find(|v| &v.name == name).map(|v| v.tag)
    }
    pub fn name_by_tag(&self, tag: u8) -> Option<&VariantName> {
        self.0.iter().find(|v| v.tag == tag).map(|v| &v.name)
    }
    pub fn has_tag(&self, tag: u8) -> bool { self.0.iter().any(|v| v.tag == tag) }
    pub fn has_name(&self, name: &VariantName) -> bool { self.0.iter().any(|v| &v.name == name) }
    pub fn by_tag(&self, tag: u8) -> Option<&Variant> { self.0.iter().find(|v| v.tag == tag) }
    pub fn by_name(&self, name: &VariantName) -> Option<&Variant> {
        self.0.iter().find(|v| &v.name == name)
    }
}

impl Display for EnumVariants {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("(|)");
        }
        let mut iter = self.iter();
        let mut last_tag = 0;
        if let Some(variant) = iter.next() {
            write!(f, "{variant}")?;
            if variant.tag != last_tag {
                last_tag = variant.tag;
                write!(f, "#{last_tag}")?;
            }
            last_tag = last_tag.saturating_add(1);
        }
        let mut chunk_size = None;
        if self.len() == 1 {
            f.write_str(" | (|)")?;
        }
        loop {
            for variant in iter.by_ref().take(chunk_size.unwrap_or(3)) {
                write!(f, " | {variant}")?;
                if variant.tag != last_tag {
                    last_tag = variant.tag;
                    write!(f, "#{last_tag}")?;
                }
                last_tag = last_tag.saturating_add(1);
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
