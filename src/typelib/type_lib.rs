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

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use amplify::confinement::{Confined, TinyOrdMap};

use crate::ast::TranslateError;
use crate::typelib::id::TypeLibId;
use crate::{Ident, KeyTy, SemId, SemVer, Ty, TypeName, TypeRef};

/// Top-level data type contained within a library.
#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display("data {name:16} :: {ty}")]
pub struct StrictType {
    pub name: TypeName,
    pub ty: Ty<LibRef>,
}

impl StrictType {
    pub fn with(name: TypeName, ty: Ty<LibRef>) -> StrictType { StrictType { name, ty } }

    pub fn id(&self) -> SemId { self.ty.id(Some(&self.name)) }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum InlineRef {
    Builtin(Ty<InlineRef1>),
    Named(TypeName, SemId),
    Extern(TypeName, LibAlias, SemId),
}

impl TypeRef for InlineRef {
    fn id(&self) -> SemId {
        match self {
            InlineRef::Named(_, id) | InlineRef::Extern(_, _, id) => *id,
            InlineRef::Builtin(ty) => ty.id(None),
        }
    }
}

impl Display for InlineRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef::Named(name, _) => write!(f, "{}", name),
            InlineRef::Extern(name, lib, _) => write!(f, "{}.{}", lib, name),
            InlineRef::Builtin(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum InlineRef1 {
    Builtin(Ty<InlineRef2>),
    Named(TypeName, SemId),
    Extern(TypeName, LibAlias, SemId),
}

impl TypeRef for InlineRef1 {
    fn id(&self) -> SemId {
        match self {
            InlineRef1::Named(_, id) | InlineRef1::Extern(_, _, id) => *id,
            InlineRef1::Builtin(ty) => ty.id(None),
        }
    }
}

impl Display for InlineRef1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef1::Named(name, _) => write!(f, "{}", name),
            InlineRef1::Extern(name, lib, _) => write!(f, "{}.{}", lib, name),
            InlineRef1::Builtin(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum InlineRef2 {
    Builtin(Ty<KeyTy>),
    Named(TypeName, SemId),
    Extern(TypeName, LibAlias, SemId),
}

impl TypeRef for InlineRef2 {
    fn id(&self) -> SemId {
        match self {
            InlineRef2::Named(_, id) | InlineRef2::Extern(_, _, id) => *id,
            InlineRef2::Builtin(ty) => ty.id(None),
        }
    }
}

impl Display for InlineRef2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InlineRef2::Named(name, _) => write!(f, "{}", name),
            InlineRef2::Extern(name, lib, _) => write!(f, "{}.{}", lib, name),
            InlineRef2::Builtin(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum LibRef {
    #[from]
    Inline(Ty<InlineRef>),

    Named(TypeName, SemId),

    Extern(TypeName, LibAlias, SemId),
}

impl TypeRef for LibRef {
    fn id(&self) -> SemId {
        match self {
            LibRef::Named(_, id) | LibRef::Extern(_, _, id) => *id,
            LibRef::Inline(ty) => ty.id(None),
        }
    }
}

impl Display for LibRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LibRef::Named(name, _) => write!(f, "{}", name),
            LibRef::Inline(ty) if ty.is_compound() => write!(f, "({})", ty),
            LibRef::Inline(ty) => write!(f, "{}", ty),
            LibRef::Extern(name, lib, _) => write!(f, "{}.{}", lib, name),
        }
    }
}

pub type LibAlias = Ident;
pub type LibName = Ident;

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display("typelib {name}@{ver} {id:#}")]
pub struct Dependency {
    pub id: TypeLibId,
    pub name: LibName,
    pub ver: SemVer,
}

pub type TypeMap = Confined<BTreeMap<TypeName, StrictType>, 1, { u16::MAX as usize }>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TypeLib {
    pub name: LibName,
    pub dependencies: TinyOrdMap<LibAlias, Dependency>,
    pub types: TypeMap,
}

impl TypeLib {
    pub fn with(name: LibName, root: StrictType) -> Self {
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
        let alias = alias.unwrap_or_else(|| dependency.name.clone());
        if self.dependencies.contains_key(&alias) {
            return Err(TranslateError::DuplicatedDependency(dependency));
        }
        self.dependencies.insert(alias, dependency)?;
        Ok(())
    }

    pub fn populate(&mut self, ty: StrictType) -> Result<(), TranslateError> {
        if self.types.contains_key(&ty.name) {
            return Err(TranslateError::DuplicateName(ty.name.clone()));
        }
        self.types.insert(ty.name.clone(), ty)?;
        Ok(())
    }

    // TODO: Check that all dependencies are used
}

impl Display for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "namespace {}", self.name)?;
        writeln!(f)?;
        for (alias, dep) in &self.dependencies {
            if alias != &dep.name {
                writeln!(f, "{} as {}", dep, alias)?;
            } else {
                Display::fmt(dep, f)?;
            }
        }
        if self.dependencies.is_empty() {
            f.write_str("-- no dependencies\n")?;
        }
        writeln!(f)?;
        for ty in self.types.values() {
            writeln!(f, "{}", ty)?;
        }
        Ok(())
    }
}
