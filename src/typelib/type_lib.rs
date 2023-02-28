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

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use amplify::confinement::{Confined, TinyOrdMap};
use amplify::Wrapper;
use blake3::Hasher;
use encoding::{Ident, InvalidIdent, StrictDumb};
use strict_encoding::{LibName, TypeName, STRICT_TYPES_LIB};

use crate::ast::HashId;
use crate::typelib::id::TypeLibId;
use crate::typelib::translate::TranslateError;
use crate::{KeyTy, SemId, SemVer, Ty, TypeRef};

/// Top-level data type contained within a library.
#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("data {name:16} :: {ty}")]
pub struct LibType {
    pub name: TypeName,
    pub ty: Ty<LibRef>,
}

impl LibType {
    pub fn with(name: TypeName, ty: Ty<LibRef>) -> LibType { LibType { name, ty } }
    pub fn id(&self) -> SemId { self.ty.id(Some(&self.name)) }
}

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
        hasher.update(&[b'.']);
        hasher.update(self.name.as_bytes());
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
    const TYPE_NAME: &'static str = "InlineRef";
    fn id(&self) -> SemId {
        match self {
            InlineRef::Named(_, id) => *id,
            InlineRef::Extern(ext) => ext.id,
            InlineRef::Inline(ty) => ty.id(None),
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
    const TYPE_NAME: &'static str = "InlineRef1";
    fn id(&self) -> SemId {
        match self {
            InlineRef1::Named(_, id) => *id,
            InlineRef1::Extern(ext) => ext.id,
            InlineRef1::Inline(ty) => ty.id(None),
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
    const TYPE_NAME: &'static str = "InlineRef2";
    fn id(&self) -> SemId {
        match self {
            InlineRef2::Named(_, id) => *id,
            InlineRef2::Extern(ext) => ext.id,
            InlineRef2::Inline(ty) => ty.id(None),
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
    const TYPE_NAME: &'static str = "LibRef";
    fn id(&self) -> SemId {
        match self {
            LibRef::Named(_, id) => *id,
            LibRef::Extern(ext) => ext.id,
            LibRef::Inline(ty) => ty.id(None),
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
            LibRef::Named(name, _) => write!(f, "{name}"),
            LibRef::Inline(ty) if ty.is_compound() => write!(f, "({ty})"),
            LibRef::Inline(ty) => write!(f, "{ty}"),
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
#[display("import {name}@{ver} {id:#}")]
pub struct Dependency {
    pub id: TypeLibId,
    pub name: LibName,
    pub ver: SemVer,
}

impl Dependency {
    pub fn with(id: TypeLibId, name: LibName, ver: (u16, u16, u16)) -> Self {
        let ver = SemVer::new(ver.0, ver.1, ver.2);
        Dependency { id, name, ver }
    }
}

pub type TypeMap = Confined<BTreeMap<TypeName, LibType>, 1, { u16::MAX as usize }>;

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(
    lib = STRICT_TYPES_LIB,
    dumb = { TypeLib {
        name: LibName::strict_dumb(),
        dependencies: default!(),
        types: confined_bmap!(tn!("DumbType") => LibType::strict_dumb())
    } }
)]
pub struct TypeLib {
    pub name: LibName,
    pub dependencies: TinyOrdMap<LibAlias, Dependency>,
    pub types: TypeMap,
}

impl TypeLib {
    pub fn with(name: LibName, root: LibType) -> Self {
        let types = Confined::with((root.name.clone(), root));
        TypeLib {
            name,
            dependencies: empty!(),
            types,
        }
    }

    pub fn import(
        &mut self,
        dependency: Dependency,
        alias: Option<LibAlias>,
    ) -> Result<(), TranslateError> {
        let alias = alias.unwrap_or_else(|| {
            let ident = dependency.name.to_inner();
            LibAlias::from_inner(ident)
        });
        if self.dependencies.contains_key(&alias) {
            return Err(TranslateError::DuplicatedDependency(dependency));
        }
        self.dependencies.insert(alias, dependency)?;
        Ok(())
    }

    pub fn populate(&mut self, ty: LibType) -> Result<(), TranslateError> {
        if self.types.contains_key(&ty.name) {
            return Err(TranslateError::DuplicateName(ty.name));
        }
        self.types.insert(ty.name.clone(), ty)?;
        Ok(())
    }

    // TODO: Check that all dependencies are used
}

impl Display for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typelib {} -- {:#}", self.name, self.id())?;
        writeln!(f)?;
        for (alias, dep) in &self.dependencies {
            if alias.as_inner() != dep.name.as_inner() {
                writeln!(f, "{dep} as {alias}")?;
            } else {
                Display::fmt(dep, f)?;
            }
        }
        if self.dependencies.is_empty() {
            f.write_str("-- no dependencies")?;
        }
        writeln!(f)?;
        writeln!(f)?;
        for ty in self.types.values() {
            writeln!(f, "{ty}")?;
        }
        Ok(())
    }
}
