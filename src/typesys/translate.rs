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
use crate::typesys::TypeFqn;
use crate::{Dependency, KeyTy, LibRef, SemId, Translate, Ty, TypeLib, TypeRef, TypeSystem};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct TypeFqid {
    pub id: SemId,
    pub fqn: Option<TypeFqn>,
}

impl TypeFqid {
    pub fn unnamed(id: SemId) -> TypeFqid { TypeFqid { id, fqn: None } }

    pub fn named(id: SemId, lib: LibName, name: TypeName) -> TypeFqid {
        TypeFqid {
            id,
            fqn: Some(TypeFqn::with(lib, name)),
        }
    }
}

impl HashId for TypeFqid {
    fn hash_id(&self, hasher: &mut Hasher) { hasher.update(self.id.as_slice()); }
}

impl TypeRef for TypeFqid {
    fn id(&self) -> SemId { self.id }
}

impl Display for TypeFqid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.fqn {
            Some(fqn) => Display::fmt(fqn, f),
            None => Display::fmt(&self.id, f),
        }
    }
}

impl Translate<TypeFqid> for SemId {
    type Builder = ();
    type Context = TypeSystem;
    type Error = Error;

    fn translate(
        self,
        _builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<TypeFqid, Self::Error> {
        ctx.iter()
            .find(|(id, _)| **id == self)
            .map(|(id, info)| TypeFqid {
                id: *id,
                fqn: info.fqn.clone(),
            })
            .ok_or(Error::UnknownType(self))
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SystemBuilder {
    unused_deps: BTreeSet<Dependency>,
    used_deps: BTreeSet<LibName>,
    types: BTreeMap<TypeFqid, Ty<SemId>>,
}

impl SystemBuilder {
    pub fn new() -> SystemBuilder { SystemBuilder::default() }

    pub fn import(mut self, lib: TypeLib) -> Result<Self, Error> {
        self.used_deps.insert(lib.name.clone());
        self.unused_deps
            .extend(lib.dependencies.into_iter().filter(|dep| !self.used_deps.contains(&dep.name)));
        self.unused_deps.retain(|dep| dep.name != lib.name);

        for (ty_name, ty) in lib.types {
            let id = ty.id(Some(&ty_name));
            let ty = ty.translate(&mut self, &())?;
            let fqid = TypeFqid::named(id, lib.name.clone(), ty_name.clone());
            self.types.insert(fqid, ty);
        }

        Ok(self)
    }

    pub fn finalize(self) -> Result<TypeSystem, Vec<Error>> {
        let mut errors = vec![];
        for dep in self.unused_deps {
            errors.push(Error::UnusedImport(dep));
        }
        for (fqid, ty) in &self.types {
            for inner_id in ty.type_refs() {
                if !self.types.contains_key(&fqid) {
                    errors.push(Error::InnerTypeAbsent {
                        unknown: *inner_id,
                        known: fqid.id,
                    });
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        let mut sys = TypeSystem::new();
        for (fqid, ty) in self.types {
            match sys.insert_unchecked(fqid, ty) {
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
        self.types.insert(TypeFqid::unnamed(id), ty);
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
            LibRef::Named(_, id) => Ok(id),
            LibRef::Inline(inline_ty) => builder.translate_inline(inline_ty),
            LibRef::Extern(ExternRef { id, .. }) => Ok(id),
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
            InlineRef::Named(_, id) => Ok(id),
            InlineRef::Inline(inline_ty) => builder.translate_inline(inline_ty),
            InlineRef::Extern(ExternRef { id, .. }) => Ok(id),
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
            InlineRef1::Named(_, id) => Ok(id),
            InlineRef1::Inline(inline_ty) => builder.translate_inline(inline_ty),
            InlineRef1::Extern(ExternRef { id, .. }) => Ok(id),
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
            InlineRef2::Named(_, id) => Ok(id),
            InlineRef2::Inline(inline_ty) => builder.translate_inline(inline_ty),
            InlineRef2::Extern(ExternRef { id, .. }) => Ok(id),
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
    /// unused import `{0}`.
    UnusedImport(Dependency),

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
