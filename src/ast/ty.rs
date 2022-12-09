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
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use amplify::ascii::{AsAsciiStrError, AsciiChar, AsciiString};
use amplify::confinement;
use amplify::confinement::Confined;

use crate::primitive::constants::*;
use crate::util::{Size, Sizing};

/// Glue for constructing ASTs.
pub trait TypeRef: Clone + Eq + Sized {}

#[derive(Wrapper, WrapperMut, Clone, PartialEq, Eq, Debug, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct SubTy(Box<Ty>);

impl TypeRef for SubTy {}

impl SubTy {
    pub fn ty(&self) -> &Ty { &self.0.deref() }
}

impl From<Ty> for SubTy {
    fn from(ty: Ty) -> Self { SubTy(Box::new(ty)) }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum InvalidIdent {
    /// identifier name must start with alphabetic character and not `{0}`
    NonAlphabetic(AsciiChar),

    /// identifier name contains invalid character `{0}`
    InvalidChar(AsciiChar),

    #[from(AsAsciiStrError)]
    /// identifier name contains non-ASCII character(s)
    NonAsciiChar,

    /// identifier name has invalid length
    #[from]
    Confinement(confinement::Error),
}

/// Identifier (field or type name).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
#[display(inner)]
pub struct Ident(Confined<AsciiString, 1, 32>);

impl Deref for Ident {
    type Target = Confined<AsciiString, 1, 32>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<&'static str> for Ident {
    fn from(s: &'static str) -> Self {
        Ident::try_from(
            Confined::try_from(AsciiString::from_ascii(s).expect("invalid identifier name"))
                .expect("invalid identifier name"),
        )
        .expect("invalid identifier name")
    }
}

pub type FieldName = Ident;

#[derive(Clone, Eq, Hash, Debug, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[display("{name}")]
pub struct Field {
    pub name: FieldName,
    pub value: u8,
}

impl Field {
    pub fn new(name: FieldName, value: u8) -> Field { Field { name, value } }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool { self.name == other.name || self.value == other.value }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        self.value.cmp(&other.value)
    }
}

/// Provides guarantees that the type information fits maximum type size
/// requirements, i.e. the serialized AST does not exceed `u24::MAX` bytes.
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Ty<Ref: TypeRef = SubTy>(TyInner<Ref>);

impl<Ref: TypeRef> Deref for Ty<Ref> {
    type Target = TyInner<Ref>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<Ref: TypeRef> Ty<Ref> {
    pub const UNIT: Ty<Ref> = Ty(TyInner::Primitive(UNIT));
    pub const BYTE: Ty<Ref> = Ty(TyInner::Primitive(BYTE));
    pub const CHAR: Ty<Ref> = Ty(TyInner::Primitive(CHAR));

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

    pub fn enumerate(variants: Variants) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::Enum(variants))
    }
    pub fn union(fields: Fields<Ref>) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::Union(fields))
    }
    pub fn composition(fields: Fields<Ref>) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::Struct(fields))
    }

    pub fn ascii(sizing: Sizing) -> Self { Ty(TyInner::Ascii(sizing)) }
    pub fn string(sizing: Sizing) -> Self { Ty(TyInner::Unicode(sizing)) }

    pub fn list(ty: Ref, sizing: Sizing) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::List(ty, sizing))
    }
    pub fn set(ty: Ref, sizing: Sizing) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::Set(ty, sizing))
    }
    pub fn map(key: KeyTy, val: Ref, sizing: Sizing) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::Map(key, val, sizing))
    }
}

impl Ty {
    pub fn byte_array(len: u16) -> Self { Ty(TyInner::Array(Ty::BYTE.into(), len)) }
    pub fn bytes(sizing: Sizing) -> Self { Ty(TyInner::List(Ty::BYTE.into(), sizing)) }
    pub fn option(ty: Ty) -> Self {
        // TODO: Check for AST size
        Ty(TyInner::Union(fields![
            "None" => Ty::UNIT,
            "Some" => ty
        ]))
    }
}

impl Ty {
    pub fn try_into_key_ty(self) -> Result<KeyTy, TyInner> {
        Ok(match self.0 {
            TyInner::Primitive(code) => KeyTy::Primitive(code),
            TyInner::Enum(vars) => KeyTy::Enum(vars),
            TyInner::Array(ty, len) if **ty == Ty::BYTE => KeyTy::Array(len),
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub enum TyInner<Ref: TypeRef = SubTy> {
    Primitive(Primitive),
    Enum(Variants),
    Union(Fields<Ref>),
    Struct(Fields<Ref>),
    Array(Ref, u16),
    Ascii(Sizing),
    Unicode(Sizing),
    List(Ref, Sizing),
    Set(Ref, Sizing),
    Map(KeyTy, Ref, Sizing),
}

impl TyInner<SubTy> {
    pub fn size(&self) -> Size {
        match self {
            TyInner::Primitive(UNIT) | TyInner::Primitive(BYTE) | TyInner::Primitive(CHAR) => {
                Size::Fixed(1)
            }
            TyInner::Primitive(F16B) => Size::Fixed(2),
            TyInner::Primitive(primitive) => Size::Fixed(primitive.size()),
            TyInner::Union(fields) => {
                fields.values().map(|alt| alt.ty().size()).max().unwrap_or(Size::Fixed(0))
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub enum KeyTy {
    Primitive(Primitive),
    Enum(Variants),
    /// Fixed-size byte array
    Array(u16),
    Ascii(Sizing),
    Unicode(Sizing),
    Bytes(Sizing),
}

pub type Fields<Ref = SubTy> = Confined<BTreeMap<Field, Ref>, 1, { u8::MAX as usize }>;

#[derive(Wrapper, WrapperMut, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Variants(Confined<BTreeSet<Field>, 1, { u8::MAX as usize }>);

impl Display for Variants {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        let last = iter.next_back();
        for variant in iter {
            write!(f, "{}, ", variant)?;
        }
        if let Some(variant) = last {
            write!(f, "{}", variant)?;
        }
        Ok(())
    }
}
