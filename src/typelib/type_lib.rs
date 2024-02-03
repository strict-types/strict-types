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
use encoding::StrictDumb;
use strict_encoding::{LibName, TypeName, STRICT_TYPES_LIB};

use crate::typelib::compile::CompileError;
use crate::typelib::id::TypeLibId;
use crate::typelib::ExternTypes;
use crate::{SemId, Ty, TypeRef};

pub trait LibSubref: TypeRef {}
impl LibSubref for LibRef {}
impl LibSubref for InlineRef {}
impl LibSubref for InlineRef1 {}
impl LibSubref for InlineRef2 {}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("{lib_id}.{sem_id:0}")]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct ExternRef {
    pub lib_id: TypeLibId,
    pub sem_id: SemId,
}

impl ExternRef {
    pub fn with(lib_id: TypeLibId, sem_id: SemId) -> ExternRef { ExternRef { lib_id, sem_id } }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { InlineRef::Inline(Ty::strict_dumb()) })]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub enum InlineRef {
    #[from]
    Inline(Ty<InlineRef1>),
    Named(SemId),
    Extern(ExternRef),
}

impl TypeRef for InlineRef {
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
}

impl Display for InlineRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef::Named(sem_id) => write!(f, "{sem_id:0}"),
            InlineRef::Extern(ext) => Display::fmt(ext, f),
            InlineRef::Inline(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { InlineRef1::Inline(Ty::strict_dumb()) })]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub enum InlineRef1 {
    #[from]
    Inline(Ty<InlineRef2>),
    Named(SemId),
    Extern(ExternRef),
}

impl TypeRef for InlineRef1 {
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
}

impl Display for InlineRef1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef1::Named(sem_id) => write!(f, "{sem_id:0}"),
            InlineRef1::Extern(ext) => Display::fmt(ext, f),
            InlineRef1::Inline(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { InlineRef2::Named(SemId::strict_dumb()) })]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub enum InlineRef2 {
    Named(SemId),
    Extern(ExternRef),
}

impl TypeRef for InlineRef2 {
    fn is_compound(&self) -> bool { false }
    fn is_byte(&self) -> bool { false }
    fn is_unicode_char(&self) -> bool { false }
}

impl Display for InlineRef2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef2::Named(sem_id) => write!(f, "{sem_id:0}"),
            InlineRef2::Extern(ext) => Display::fmt(ext, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { LibRef::Inline(Ty::strict_dumb()) })]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub enum LibRef {
    #[from]
    Inline(Ty<InlineRef>),
    Named(SemId),
    Extern(ExternRef),
}

impl TypeRef for LibRef {
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
}

impl Display for LibRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LibRef::Named(sem_id) => Display::fmt(sem_id, f),
            LibRef::Inline(ty) => Display::fmt(ty, f),
            LibRef::Extern(ext) => Display::fmt(ext, f),
        }
    }
}

#[derive(Clone, Eq, Debug, Display)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("import {id:+}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Dependency {
    pub id: TypeLibId,
    pub name: LibName,
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool { self.id == other.id || self.name == other.name }
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

impl From<&TypeLib> for Dependency {
    fn from(lib: &TypeLib) -> Self {
        Dependency {
            id: lib.id(),
            name: lib.name.clone(),
        }
    }
}

pub type TypeMap = Confined<BTreeMap<TypeName, Ty<LibRef>>, 1, { u16::MAX as usize }>;

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(
    lib = STRICT_TYPES_LIB,
    dumb = { TypeLib {
        name: LibName::strict_dumb(),
        dependencies: default!(),
        extern_types: default!(),
        types: confined_bmap!(tn!("DumbType") => Ty::strict_dumb())
    } }
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct TypeLib {
    pub name: LibName,
    pub dependencies: TinyOrdSet<Dependency>,
    pub extern_types: ExternTypes,
    pub types: TypeMap,
}

impl TypeLib {
    pub fn to_dependency(&self) -> Dependency { Dependency::with(self.id(), self.name.clone()) }

    pub fn import(&mut self, dependency: Dependency) -> Result<(), CompileError> {
        if self.dependencies.contains(&dependency) {
            return Err(CompileError::DuplicatedDependency(dependency));
        }
        self.dependencies.push(dependency).map_err(|_| CompileError::TooManyDependencies)?;
        Ok(())
    }

    pub fn populate(&mut self, name: TypeName, ty: Ty<LibRef>) -> Result<(), CompileError> {
        if self.types.contains_key(&name) {
            return Err(CompileError::DuplicateName(name));
        }
        self.types.insert(name, ty).map_err(|_| CompileError::TooManyTypes)?;
        Ok(())
    }

    // TODO: Check that all dependencies are used
}
