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

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

use amplify::confinement::Confined;
use amplify::{confinement, Wrapper};

use crate::primitive::constants::*;
use crate::util::{Size, Sizing};
use crate::{Encode, Ident, Serialize, StenSchema, StenType, TyId, TyIter};

pub const MAX_SERIALIZED_SIZE: usize = 1 << 24 - 1;

/// Glue for constructing ASTs.
pub trait TypeRef: StenSchema + Clone + Eq + Debug + Encode + Sized {
    fn id(&self) -> TyId;
}
pub trait NestedRef: TypeRef + Deref<Target = Ty<Self>> {
    fn as_ty(&self) -> &Ty<Self>;
    fn into_ty(self) -> Ty<Self>;
    fn iter(&self) -> TyIter<Self> { TyIter::from(self) }
}
pub trait RecursiveRef: NestedRef {
    fn byte_size(&self) -> Size { self.as_ty().byte_size() }
}

impl TypeRef for SubTy {
    fn id(&self) -> TyId { self.as_ty().id() }
}
impl NestedRef for SubTy {
    fn as_ty(&self) -> &Ty<Self> { &self.0.deref() }
    fn into_ty(self) -> Ty<Self> { *self.0 }
}
impl RecursiveRef for SubTy {}

impl TypeRef for StenType {
    fn id(&self) -> TyId { self.as_ty().id() }
}
impl NestedRef for StenType {
    fn as_ty(&self) -> &Ty<Self> { &self.ty }
    fn into_ty(self) -> Ty<Self> { *self.ty }
}
impl RecursiveRef for StenType {}

#[derive(Wrapper, WrapperMut, Clone, PartialEq, Eq, Debug, From)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct SubTy(Box<Ty>);

impl StenSchema for SubTy {
    const STEN_TYPE_NAME: &'static str = "SubTy";

    fn sten_ty() -> Ty<StenType> { Ty::composition(fields!(Ty::<StenType>::sten_type())) }
}

impl Deref for SubTy {
    type Target = Ty;

    fn deref(&self) -> &Self::Target { self.0.deref() }
}

impl DerefMut for SubTy {
    fn deref_mut(&mut self) -> &mut Self::Target { self.0.deref_mut() }
}

impl From<Ty> for SubTy {
    fn from(ty: Ty) -> Self { SubTy(Box::new(ty)) }
}

pub type FieldName = Ident;

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
            write!(f, "{} ", name)?;
        }
        Ok(())
    }
}

/// Provides guarantees that the type information fits maximum type size
/// requirements, i.e. the serialized AST does not exceed `u24::MAX` bytes.
#[derive(Wrapper, Clone, PartialEq, Eq, Debug, From)]
#[wrapper(Deref)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Ty<Ref = SubTy>(TyInner<Ref>)
where Ref: TypeRef;

impl<Ref: TypeRef> StenSchema for Ty<Ref> {
    const STEN_TYPE_NAME: &'static str = "Ty";

    fn sten_ty() -> Ty<StenType> {
        Ty::union(fields! {
            "Primitive" => Primitive::sten_type(),
            "Enum" => Variants::sten_type(),
            "Union" => Fields::<Ref, false>::sten_type(),
            "Struct" => Fields::<Ref, true>::sten_type(),
            "Array" => <(Ref, u16)>::sten_type(),
            "Unicode" => Sizing::sten_type(),
            "List" => <(Ref, Sizing)>::sten_type(),
            "Set" => <(Ref, Sizing)>::sten_type(),
            "Map" => <(KeyTy, Ref, Sizing)>::sten_type(),
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
    pub const UNIT: Ty<Ref> = Ty(TyInner::Primitive(UNIT));
    pub const BYTE: Ty<Ref> = Ty(TyInner::Primitive(BYTE));
    pub const ASCII: Ty<Ref> = Ty(TyInner::Primitive(ASCII));

    pub const U8: Ty<Ref> = Ty(TyInner::Primitive(U8));
    pub const U16: Ty<Ref> = Ty(TyInner::Primitive(U16));
    pub const U24: Ty<Ref> = Ty(TyInner::Primitive(U24));
    pub const U32: Ty<Ref> = Ty(TyInner::Primitive(U32));
    pub const U64: Ty<Ref> = Ty(TyInner::Primitive(U64));
    pub const U128: Ty<Ref> = Ty(TyInner::Primitive(U128));
    pub const U256: Ty<Ref> = Ty(TyInner::Primitive(U256));
    pub const U512: Ty<Ref> = Ty(TyInner::Primitive(U512));
    pub const U1024: Ty<Ref> = Ty(TyInner::Primitive(U1024));

    pub const I8: Ty<Ref> = Ty(TyInner::Primitive(I8));
    pub const I16: Ty<Ref> = Ty(TyInner::Primitive(I16));
    pub const I24: Ty<Ref> = Ty(TyInner::Primitive(I24));
    pub const I32: Ty<Ref> = Ty(TyInner::Primitive(I32));
    pub const I64: Ty<Ref> = Ty(TyInner::Primitive(I64));
    pub const I128: Ty<Ref> = Ty(TyInner::Primitive(I128));
    pub const I256: Ty<Ref> = Ty(TyInner::Primitive(I256));
    pub const I512: Ty<Ref> = Ty(TyInner::Primitive(I512));
    pub const I1024: Ty<Ref> = Ty(TyInner::Primitive(I1024));

    pub const F16B: Ty<Ref> = Ty(TyInner::Primitive(F16B));
    pub const F16: Ty<Ref> = Ty(TyInner::Primitive(F16));
    pub const F32: Ty<Ref> = Ty(TyInner::Primitive(F32));
    pub const F64: Ty<Ref> = Ty(TyInner::Primitive(F64));
    pub const F80: Ty<Ref> = Ty(TyInner::Primitive(F80));
    pub const F128: Ty<Ref> = Ty(TyInner::Primitive(F128));
    pub const F256: Ty<Ref> = Ty(TyInner::Primitive(F256));

    pub fn enumerate(variants: Variants) -> Self { Ty(TyInner::Enum(variants)) }
    pub fn union(fields: Fields<Ref, false>) -> Self {
        let ty = Ty(TyInner::Union(fields));
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }
    pub fn composition(fields: Fields<Ref, true>) -> Self {
        let ty = Ty(TyInner::Struct(fields));
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }

    pub fn unicode(sizing: Sizing) -> Self { Ty(TyInner::Unicode(sizing)) }

    pub fn list(ty: Ref, sizing: Sizing) -> Self {
        let ty = Ty(TyInner::List(ty, sizing));
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }
    pub fn set(ty: Ref, sizing: Sizing) -> Self {
        let ty = Ty(TyInner::Set(ty, sizing));
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }
    pub fn map(key: KeyTy, val: Ref, sizing: Sizing) -> Self {
        let ty = Ty(TyInner::Map(key, val, sizing));
        assert!(ty.serialized_len() <= MAX_SERIALIZED_SIZE);
        ty
    }

    pub fn is_primitive(&self) -> bool { matches!(self.as_inner(), TyInner::Primitive(_)) }
    pub fn is_compound(&self) -> bool {
        matches!(self.as_inner(), TyInner::Struct(fields)
            if fields.len() > 1
            || fields.keys().next().expect("always at least one field").name.is_some())
            || (matches!(self.as_inner(), TyInner::Enum(_) | TyInner::Union(_) | TyInner::Map(..))
                && !self.is_option())
    }
    pub fn is_option(&self) -> bool {
        matches!(self.as_inner(),
            TyInner::Union(fields) if fields.len() == 2
            && fields.contains_key(&Field::none())
            && fields.contains_key(&Field::some())
        )
    }
}

impl Ty {
    pub fn byte_array(len: u16) -> Self { Ty(TyInner::Array(Ty::BYTE.into(), len)) }
    pub fn bytes(sizing: Sizing) -> Self { Ty(TyInner::List(Ty::BYTE.into(), sizing)) }
    pub fn ascii(sizing: Sizing) -> Self { Ty(TyInner::List(Ty::ASCII.into(), sizing)) }
    pub fn option(ty: Ty) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::Union(fields![
            "None" => Ty::UNIT,
            "Some" => ty
        ]))
    }
}

impl Ty<StenType> {
    pub fn byte_array(len: u16) -> Self { Ty(TyInner::Array(StenType::byte(), len)) }
    pub fn bytes(sizing: Sizing) -> Self { Ty(TyInner::List(StenType::byte(), sizing)) }
    pub fn ascii(sizing: Sizing) -> Self { Ty(TyInner::List(StenType::ascii(), sizing)) }
    pub fn option(ty: StenType) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::Union(fields![
            "None" => StenType::unit(),
            "Some" => ty
        ]))
    }
}

impl<Ref: TypeRef> Display for Ty<Ref>
where Ref: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.as_inner() {
            TyInner::Primitive(prim) => Display::fmt(prim, f),
            TyInner::Enum(vars) => Display::fmt(vars, f),
            TyInner::Union(fields) if self.is_option() => {
                write!(f, "{}?", fields.get(&Field::some()).expect("optional"))
            }
            TyInner::Union(fields) => Display::fmt(fields, f),
            TyInner::Struct(fields) => Display::fmt(fields, f),
            TyInner::Array(ty, len) => write!(f, "[{} ^ {}]", ty, len),
            TyInner::Unicode(sizing) => write!(f, "[Char{}]", sizing),
            TyInner::List(ty, sizing) => write!(f, "[{}{}]", ty, sizing),
            TyInner::Set(ty, sizing) => write!(f, "{{{}{}}}", ty, sizing),
            TyInner::Map(key, ty, sizing) => write!(f, "{{{}{}}} -> [{}]", key, sizing, ty),
        }
    }
}

impl<Ref: NestedRef> Ty<Ref> {
    pub fn try_into_key(self) -> Result<KeyTy, Ty<Ref>> {
        Ok(match self.0 {
            TyInner::Primitive(code) => KeyTy::Primitive(code),
            TyInner::Enum(vars) => KeyTy::Enum(vars),
            TyInner::Array(ty, len) if ty.as_ty() == &Ty::BYTE => KeyTy::Array(len),
            TyInner::List(ty, sizing) if ty.as_ty() == &Ty::BYTE => KeyTy::Bytes(sizing),
            TyInner::Unicode(sizing) => KeyTy::Unicode(sizing),
            me @ TyInner::Union(_)
            | me @ TyInner::Struct(_)
            | me @ TyInner::Array(_, _)
            | me @ TyInner::List(_, _)
            | me @ TyInner::Set(_, _)
            | me @ TyInner::Map(_, _, _) => return Err(Ty::from_inner(me)),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub enum TyInner<Ref: TypeRef = SubTy> {
    Primitive(Primitive),
    Enum(Variants),
    Union(Fields<Ref, false>),
    Struct(Fields<Ref, true>),
    Array(Ref, u16),
    Unicode(Sizing),
    List(Ref, Sizing),
    Set(Ref, Sizing),
    Map(KeyTy, Ref, Sizing),
}

impl<Ref: NestedRef> TyInner<Ref> {
    pub fn byte_size(&self) -> Size {
        match self {
            TyInner::Primitive(UNIT) | TyInner::Primitive(BYTE) => Size::Fixed(1),
            TyInner::Primitive(F16B) => Size::Fixed(2),
            TyInner::Primitive(primitive) => Size::Fixed(primitive.size()),
            TyInner::Union(fields) => {
                fields.values().map(|alt| alt.as_ty().byte_size()).max().unwrap_or(Size::Fixed(0))
            }
            TyInner::Struct(fields) => fields.values().map(|ty| ty.byte_size()).sum(),
            TyInner::Enum(_) => Size::Fixed(1),
            TyInner::Array(_, len) => Size::Fixed(*len),
            TyInner::Unicode(..) | TyInner::List(..) | TyInner::Set(..) | TyInner::Map(..) => {
                Size::Variable
            }
        }
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
    Unicode(Sizing),
    Bytes(Sizing),
}

impl StenSchema for KeyTy {
    const STEN_TYPE_NAME: &'static str = "KeyTy";

    fn sten_ty() -> Ty<StenType> {
        Ty::union(fields! {
            "Primitive" => Primitive::sten_type(),
            "Enum" => Variants::sten_type(),
            "Array" => u16::sten_type(),
            "Unicode" => Sizing::sten_type(),
            "Bytes" => Sizing::sten_type(),
        })
    }
}

impl KeyTy {
    pub const U8: KeyTy = KeyTy::Primitive(Primitive::U8);
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
pub struct Fields<Ref: TypeRef = SubTy, const OP: bool = true>(
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
pub struct Variants(Confined<BTreeMap<Field, u8>, 1, { u8::MAX as usize }>);

impl StenSchema for Variants {
    const STEN_TYPE_NAME: &'static str = "Variants";

    fn sten_ty() -> Ty<StenType> {
        // TODO: Serialize according to this schema
        let val_ty = <(Option<FieldName>, u8)>::sten_type();
        Ty::map(KeyTy::U8, val_ty, Sizing::U8_NONEMPTY)
    }
}

impl TryFrom<BTreeMap<Field, u8>> for Variants {
    type Error = confinement::Error;

    fn try_from(inner: BTreeMap<Field, u8>) -> Result<Self, Self::Error> {
        Confined::try_from(inner).map(Variants::from)
    }
}

impl IntoIterator for Variants {
    type Item = (Field, u8);
    type IntoIter = std::collections::btree_map::IntoIter<Field, u8>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a> IntoIterator for &'a Variants {
    type Item = (&'a Field, &'a u8);
    type IntoIter = std::collections::btree_map::Iter<'a, Field, u8>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl Variants {
    pub fn into_inner(self) -> BTreeMap<Field, u8> { self.0.into_inner() }

    pub fn into_keys(self) -> std::collections::btree_map::IntoKeys<Field, u8> {
        self.0.into_inner().into_keys()
    }

    pub fn into_values(self) -> std::collections::btree_map::IntoValues<Field, u8> {
        self.0.into_inner().into_values()
    }
}

impl Display for Variants {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        let last = iter.next_back();
        for (field, val) in iter {
            write!(f, "{}= {} | ", field, val)?;
        }
        if let Some((field, val)) = last {
            write!(f, "{}= {}", field, val)?;
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

impl StenSchema for (Option<FieldName>, u8) {
    const STEN_TYPE_NAME: &'static str = "EnumTy";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields! {
            Option::<FieldName>::sten_type(),
            u8::sten_type(),
        })
    }
}
