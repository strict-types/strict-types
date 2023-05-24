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

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};

use amplify::confinement;
use blake3::Hasher;
use encoding::{LibName, TypeName, STRICT_TYPES_LIB};

use crate::ast::HashId;
use crate::typelib::{ExternRef, InlineRef, InlineRef1, InlineRef2};
use crate::typesys::{TypeFqn, TypeInfo};
use crate::{Dependency, KeyTy, LibRef, SemId, Translate, Ty, TypeLib, TypeRef, TypeSystem};

/// Information about type origin.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct TypeOrig {
    pub id: SemId,
    pub orig: Option<TypeFqn>,
}

impl TypeOrig {
    pub fn unnamed(id: SemId) -> TypeOrig { TypeOrig { id, orig: None } }

    pub fn named(id: SemId, lib: LibName, name: TypeName) -> TypeOrig {
        TypeOrig {
            id,
            orig: Some(TypeFqn::with(lib, name)),
        }
    }
}

impl HashId for TypeOrig {
    fn hash_id(&self, hasher: &mut Hasher) { hasher.update(self.id.as_slice()); }
}

impl TypeRef for TypeOrig {}

impl Display for TypeOrig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.orig {
            Some(fqn) => Display::fmt(fqn, f),
            None => Display::fmt(&self.id, f),
        }
    }
}

impl Translate<TypeOrig> for SemId {
    type Builder = ();
    type Context = TypeSystem;
    type Error = Error;

    fn translate(
        self,
        _builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<TypeOrig, Self::Error> {
        ctx.iter()
            .find(|(id, _)| **id == self)
            .map(|(id, info)| TypeOrig {
                id: *id,
                orig: info.orig.first().cloned(),
            })
            .ok_or(Error::UnknownType(self))
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SystemBuilder {
    pending_deps: BTreeSet<Dependency>,
    imported_deps: BTreeSet<LibName>,
    types: BTreeMap<SemId, TypeInfo>,
}

impl SystemBuilder {
    pub fn new() -> SystemBuilder { SystemBuilder::default() }

    pub fn import(mut self, lib: TypeLib) -> Result<Self, Error> {
        self.imported_deps.insert(lib.name.clone());
        self.pending_deps.extend(
            lib.dependencies.into_iter().filter(|dep| !self.imported_deps.contains(&dep.name)),
        );
        self.pending_deps.retain(|dep| dep.name != lib.name);

        for (ty_name, ty) in lib.types {
            let id = ty.id(Some(&ty_name));
            let ty = ty.translate(&mut self, &())?;
            let info = TypeInfo::named(lib.name.clone(), ty_name.clone(), ty);
            self.types.insert(id, info);
        }

        Ok(self)
    }

    pub fn finalize(self) -> Result<TypeSystem, Vec<Error>> {
        let mut errors = vec![];

        for dep in self.pending_deps {
            errors.push(Error::AbsentImport(dep));
        }

        for (sem_id, info) in &self.types {
            for inner_id in info.ty.type_refs() {
                if !self.types.contains_key(sem_id) {
                    errors.push(Error::InnerTypeAbsent {
                        unknown: *inner_id,
                        known: *sem_id,
                    });
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        let mut sys = TypeSystem::new();
        for (sem_id, ty) in self.types {
            match sys.insert_unchecked(sem_id, ty) {
                Err(err) => {
                    errors.push(err.into());
                    return Err(errors);
                }
                Ok(true) => unreachable!("repeated type"),
                Ok(false) => {}
            }
        }

        if errors.is_empty() {
            Ok(sys)
        } else {
            Err(errors)
        }
    }

    fn translate_inline<Ref: TypeRef>(&mut self, inline_ty: Ty<Ref>) -> Result<SemId, Error>
    where Ref: Translate<SemId, Context = (), Builder = SystemBuilder, Error = Error> {
        // compute id
        let id = inline_ty.id(None);
        // run for nested types
        let ty = inline_ty.translate(self, &())?;
        // add to system
        self.types.insert(id, TypeInfo::unnamed(ty));
        Ok(id)
    }
}

impl Translate<SemId> for LibRef {
    type Context = ();
    type Builder = SystemBuilder;
    type Error = Error;

    fn translate(
        self,
        builder: &mut Self::Builder,
        _ctx: &Self::Context,
    ) -> Result<SemId, Self::Error> {
        match self {
            LibRef::Named(sem_id) => Ok(sem_id),
            LibRef::Inline(inline_ty) => builder.translate_inline(inline_ty),
            LibRef::Extern(ExternRef { sem_id, .. }) => Ok(sem_id),
        }
    }
}

impl Translate<SemId> for InlineRef {
    type Context = ();
    type Builder = SystemBuilder;
    type Error = Error;

    fn translate(
        self,
        builder: &mut Self::Builder,
        _ctx: &Self::Context,
    ) -> Result<SemId, Self::Error> {
        match self {
            InlineRef::Named(sem_id) => Ok(sem_id),
            InlineRef::Inline(inline_ty) => builder.translate_inline(inline_ty),
            InlineRef::Extern(ExternRef { sem_id, .. }) => Ok(sem_id),
        }
    }
}

impl Translate<SemId> for InlineRef1 {
    type Context = ();
    type Builder = SystemBuilder;
    type Error = Error;

    fn translate(
        self,
        builder: &mut Self::Builder,
        _ctx: &Self::Context,
    ) -> Result<SemId, Self::Error> {
        match self {
            InlineRef1::Named(sem_id) => Ok(sem_id),
            InlineRef1::Inline(inline_ty) => builder.translate_inline(inline_ty),
            InlineRef1::Extern(ExternRef { sem_id, .. }) => Ok(sem_id),
        }
    }
}

impl Translate<SemId> for InlineRef2 {
    type Context = ();
    type Builder = SystemBuilder;
    type Error = Error;

    fn translate(
        self,
        builder: &mut Self::Builder,
        _ctx: &Self::Context,
    ) -> Result<SemId, Self::Error> {
        match self {
            InlineRef2::Named(sem_id) => Ok(sem_id),
            InlineRef2::Inline(inline_ty) => builder.translate_inline(inline_ty),
            InlineRef2::Extern(ExternRef { sem_id, .. }) => Ok(sem_id),
        }
    }
}

impl Translate<SemId> for KeyTy {
    type Context = ();
    type Builder = SystemBuilder;
    type Error = Error;

    fn translate(
        self,
        _builder: &mut Self::Builder,
        _ctx: &Self::Context,
    ) -> Result<SemId, Self::Error> {
        Err(Error::TooDeep)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// required dependency `{0}` was not imported into the builder.
    AbsentImport(Dependency),

    /// type with id `{0}` is not a part of the type system.
    UnknownType(SemId),

    /// type `{unknown}` referenced from `{known}` is not known; perhaps you need to import a
    /// library defining this type.
    InnerTypeAbsent { unknown: SemId, known: SemId },

    #[from]
    #[display(inner)]
    Confinement(confinement::Error),

    /// Too deeply nested types.
    TooDeep,
}
