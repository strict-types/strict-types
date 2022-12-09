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

use std::collections::BTreeMap;

use amplify::ascii::AsciiString;
use amplify::confinement::{Confined, SmallOrdMap};

use crate::ast::{Alternative, FieldName, Variants};
use crate::util::Sizing;

pub type TypeName = Confined<AsciiString, 1, 32>;
pub type TypeRef = TypeName;

pub struct TypeLib {
    types: SmallOrdMap<TypeName, TypeDef>,
}

pub type EnumDef = Variants;
pub type UnionDef = Confined<BTreeMap<TypeRef, Alternative>, 1, { u8::MAX as usize }>;
pub type StructDef = Confined<BTreeMap<FieldName, TypeRef>, 1, { u8::MAX as usize }>;

pub enum TypeDef {
    Primitive(u8),
    Enum(EnumDef),
    Union(UnionDef),
    Struct(StructDef),
    Array(TypeRef, u16),
    Ascii(Sizing),
    Unicode(Sizing),
    List(TypeRef, Sizing),
    Set(TypeRef, Sizing),
    Map(KeyDef, TypeRef, Sizing),
}

pub enum KeyDef {
    Primitive(u8),
    Enum(TypeRef),
    /// Fixed-size byte array
    Array(u16),
    Ascii(Sizing),
    Unicode(Sizing),
    Bytes(Sizing),
}
