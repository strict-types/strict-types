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

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use amplify::confinement::{Confined, TinyOrdSet};
use amplify::Wrapper;
use blake3::Hasher;
use encoding::{Ident, InvalidIdent, StrictDeserialize, StrictDumb, StrictSerialize};
use strict_encoding::{LibName, TypeName, STRICT_TYPES_LIB};

use crate::ast::HashId;
use crate::typelib::id::TypeLibId;
use crate::typelib::translate::TranslateError;
use crate::{KeyTy, SemId, Ty, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("{lib}.{name}")]
pub struct ExternRef {
    pub name: TypeName,
    pub lib: LibName,
    pub id: SemId,
}

impl StrictDumb for ExternRef {
    fn strict_dumb() -> Self {
        ExternRef {
            name: TypeName::strict_dumb(),
            lib: LibName::strict_dumb(),
            id: SemId::strict_dumb(),
        }
    }
}

impl HashId for ExternRef {
    fn hash_id(&self, hasher: &mut Hasher) {
        hasher.update(self.lib.as_bytes());
        hasher.update(self.id.as_slice());
    }
}

impl ExternRef {
    pub fn with(name: TypeName, lib: LibName, id: SemId) -> ExternRef {
        ExternRef { name, lib, id }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { InlineRef::Inline(Ty::strict_dumb()) })]
pub enum InlineRef {
    #[from]
    Inline(Ty<InlineRef1>),
    Named(TypeName, SemId),
    Extern(ExternRef),
}

impl TypeRef for InlineRef {
    fn id(&self) -> SemId {
        match self {
            InlineRef::Named(_, id) => *id,
            InlineRef::Extern(ext) => ext.id,
            InlineRef::Inline(ty) => ty.id(None),
        }
    }
    fn is_compound(&self) -> bool {
        match self {
            InlineRef::Inline(ty) => ty.is_compound(),
            _ => false,
        }
    }
    fn is_byte(&self) -> bool {
        match self {
            InlineRef::Inline(ty) => ty.is_byte(),
            _ => false,
        }
    }
    fn is_unicode_char(&self) -> bool {
        match self {
            InlineRef::Inline(ty) => ty.is_unicode_char(),
            _ => false,
        }
    }
    fn is_ascii_char(&self) -> bool {
        match self {
            InlineRef::Inline(ty) => ty.is_ascii_char(),
            _ => false,
        }
    }
}

impl HashId for InlineRef {
    fn hash_id(&self, hasher: &mut Hasher) {
        match self {
            InlineRef::Inline(ty) => ty.hash_id(hasher),
            InlineRef::Named(name, id) => {
                hasher.update(name.as_bytes());
                id.hash_id(hasher);
            }
            InlineRef::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl Display for InlineRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef::Named(name, _) => write!(f, "{name}"),
            InlineRef::Extern(ext) => Display::fmt(ext, f),
            InlineRef::Inline(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { InlineRef1::Inline(Ty::strict_dumb()) })]
pub enum InlineRef1 {
    #[from]
    Inline(Ty<InlineRef2>),
    Named(TypeName, SemId),
    Extern(ExternRef),
}

impl TypeRef for InlineRef1 {
    fn id(&self) -> SemId {
        match self {
            InlineRef1::Named(_, id) => *id,
            InlineRef1::Extern(ext) => ext.id,
            InlineRef1::Inline(ty) => ty.id(None),
        }
    }
    fn is_compound(&self) -> bool {
        match self {
            InlineRef1::Inline(ty) => ty.is_compound(),
            _ => false,
        }
    }
    fn is_byte(&self) -> bool {
        match self {
            InlineRef1::Inline(ty) => ty.is_byte(),
            _ => false,
        }
    }
    fn is_unicode_char(&self) -> bool {
        match self {
            InlineRef1::Inline(ty) => ty.is_unicode_char(),
            _ => false,
        }
    }
    fn is_ascii_char(&self) -> bool {
        match self {
            InlineRef1::Inline(ty) => ty.is_ascii_char(),
            _ => false,
        }
    }
}

impl HashId for InlineRef1 {
    fn hash_id(&self, hasher: &mut Hasher) {
        match self {
            InlineRef1::Inline(ty) => ty.hash_id(hasher),
            InlineRef1::Named(name, id) => {
                hasher.update(name.as_bytes());
                id.hash_id(hasher);
            }
            InlineRef1::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl Display for InlineRef1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef1::Named(name, _) => write!(f, "{name}"),
            InlineRef1::Extern(ext) => Display::fmt(ext, f),
            InlineRef1::Inline(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { InlineRef2::Inline(Ty::strict_dumb()) })]
pub enum InlineRef2 {
    #[from]
    Inline(Ty<KeyTy>),
    Named(TypeName, SemId),
    Extern(ExternRef),
}

impl TypeRef for InlineRef2 {
    fn id(&self) -> SemId {
        match self {
            InlineRef2::Named(_, id) => *id,
            InlineRef2::Extern(ext) => ext.id,
            InlineRef2::Inline(ty) => ty.id(None),
        }
    }
    fn is_compound(&self) -> bool {
        match self {
            InlineRef2::Inline(ty) => ty.is_compound(),
            _ => false,
        }
    }
    fn is_byte(&self) -> bool {
        match self {
            InlineRef2::Inline(ty) => ty.is_byte(),
            _ => false,
        }
    }
    fn is_unicode_char(&self) -> bool {
        match self {
            InlineRef2::Inline(ty) => ty.is_unicode_char(),
            _ => false,
        }
    }
    fn is_ascii_char(&self) -> bool {
        match self {
            InlineRef2::Inline(ty) => ty.is_ascii_char(),
            _ => false,
        }
    }
}

impl HashId for InlineRef2 {
    fn hash_id(&self, hasher: &mut Hasher) {
        match self {
            InlineRef2::Inline(ty) => ty.hash_id(hasher),
            InlineRef2::Named(name, id) => {
                hasher.update(name.as_bytes());
                id.hash_id(hasher);
            }
            InlineRef2::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl Display for InlineRef2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef2::Named(name, _) => write!(f, "{name}"),
            InlineRef2::Extern(ext) => Display::fmt(ext, f),
            InlineRef2::Inline(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { LibRef::Inline(Ty::strict_dumb()) })]
pub enum LibRef {
    #[from]
    Inline(Ty<InlineRef>),
    Named(TypeName, SemId),
    Extern(ExternRef),
}

impl TypeRef for LibRef {
    fn id(&self) -> SemId {
        match self {
            LibRef::Named(_, id) => *id,
            LibRef::Extern(ext) => ext.id,
            LibRef::Inline(ty) => ty.id(None),
        }
    }
    fn is_compound(&self) -> bool {
        match self {
            LibRef::Inline(ty) => ty.is_compound(),
            _ => false,
        }
    }
    fn is_byte(&self) -> bool {
        match self {
            LibRef::Inline(ty) => ty.is_byte(),
            _ => false,
        }
    }
    fn is_unicode_char(&self) -> bool {
        match self {
            LibRef::Inline(ty) => ty.is_unicode_char(),
            _ => false,
        }
    }
    fn is_ascii_char(&self) -> bool {
        match self {
            LibRef::Inline(ty) => ty.is_ascii_char(),
            _ => false,
        }
    }
}

impl HashId for LibRef {
    fn hash_id(&self, hasher: &mut Hasher) {
        match self {
            LibRef::Inline(ty) => ty.hash_id(hasher),
            LibRef::Named(name, id) => {
                hasher.update(name.as_bytes());
                id.hash_id(hasher);
            }
            LibRef::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl Display for LibRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LibRef::Named(name, _) => Display::fmt(name, f),
            LibRef::Inline(ty) => Display::fmt(ty, f),
            LibRef::Extern(ext) => Display::fmt(ext, f),
        }
    }
}

#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, Display, FromStr)]
#[wrapper_mut(DerefMut)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct LibAlias(Ident);

impl From<&'static str> for LibAlias {
    fn from(ident: &'static str) -> Self { LibAlias(Ident::from(ident)) }
}

impl TryFrom<String> for LibAlias {
    type Error = InvalidIdent;

    fn try_from(s: String) -> Result<Self, Self::Error> { Ident::try_from(s).map(Self) }
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("import {id:+}")]
pub struct Dependency {
    pub id: TypeLibId,
    pub name: LibName,
}

impl PartialOrd for Dependency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Dependency {
    fn cmp(&self, other: &Self) -> Ordering { self.id.cmp(&other.id) }
}

impl Dependency {
    pub fn with(id: TypeLibId, name: LibName) -> Self { Dependency { id, name } }
}

pub type TypeMap = Confined<BTreeMap<TypeName, Ty<LibRef>>, 1, { u16::MAX as usize }>;

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(
    lib = STRICT_TYPES_LIB,
    dumb = { TypeLib {
        name: LibName::strict_dumb(),
        dependencies: default!(),
        types: confined_bmap!(tn!("DumbType") => Ty::strict_dumb())
    } }
)]
pub struct TypeLib {
    pub name: LibName,
    pub dependencies: TinyOrdSet<Dependency>,
    pub types: TypeMap,
}

impl StrictSerialize for TypeLib {}
impl StrictDeserialize for TypeLib {}

impl TypeLib {
    pub fn import(&mut self, dependency: Dependency) -> Result<(), TranslateError> {
        if self.dependencies.contains(&dependency) {
            return Err(TranslateError::DuplicatedDependency(dependency));
        }
        self.dependencies.push(dependency)?;
        Ok(())
    }

    pub fn populate(&mut self, name: TypeName, ty: Ty<LibRef>) -> Result<(), TranslateError> {
        if self.types.contains_key(&name) {
            return Err(TranslateError::DuplicateName(name));
        }
        self.types.insert(name, ty)?;
        Ok(())
    }

    // TODO: Check that all dependencies are used
}

impl Display for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typelib {} -- {:+}", self.name, self.id())?;
        writeln!(f)?;
        for dep in &self.dependencies {
            writeln!(f, "{dep} as {}", dep.name)?;
        }
        if self.dependencies.is_empty() {
            f.write_str("-- no dependencies")?;
        }
        writeln!(f)?;
        writeln!(f)?;
        for (name, ty) in &self.types {
            writeln!(f, "data {name:16} :: {ty}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "base64")]
impl fmt::UpperHex for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use baid58::ToBaid58;
        use base64::Engine;

        let id = self.id();

        writeln!(f, "----- BEGIN STRICT TYPE LIB -----")?;
        writeln!(f, "Id: {}", id)?;
        writeln!(f, "Checksum: {}", id.to_baid58().mnemonic())?;
        writeln!(f)?;

        let data = self.to_strict_serialized::<0xFFFFFF>().expect("in-memory");
        let engine = base64::engine::general_purpose::STANDARD;
        let data = engine.encode(data);
        let mut data = data.as_str();
        while data.len() >= 76 {
            let (line, rest) = data.split_at(76);
            writeln!(f, "{}", line)?;
            data = rest;
        }
        writeln!(f, "{}", data)?;

        writeln!(f, "\n----- END STRICT TYPE LIB -----")?;
        Ok(())
    }
}
