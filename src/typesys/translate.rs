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
use std::fmt::{self, Display, Formatter};

use amplify::confinement;
use encoding::{LibName, TypeName, STRICT_TYPES_LIB};

use crate::ast::SemCommit;
use crate::typelib::{ExternRef, InlineRef, InlineRef1, InlineRef2, LibSubref};
use crate::typesys::symbols::SymbolicSys;
use crate::typesys::{SymTy, TypeFqn};
use crate::{CommitConsume, Dependency, LibRef, SemId, Translate, Ty, TypeLib, TypeRef};

/// Information about type semantic id and fully qualified name, if any.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeSymbol {
    pub id: SemId,
    pub fqn: Option<TypeFqn>,
}

impl TypeSymbol {
    pub fn with(id: SemId, fqn: TypeFqn) -> TypeSymbol { TypeSymbol { id, fqn: Some(fqn) } }

    pub fn unnamed(id: SemId) -> TypeSymbol { TypeSymbol { id, fqn: None } }

    pub fn named(id: SemId, lib: LibName, name: TypeName) -> TypeSymbol {
        TypeSymbol {
            id,
            fqn: Some(TypeFqn::with(lib, name)),
        }
    }
}

impl SemCommit for TypeSymbol {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) { self.id.sem_commit(hasher); }
}

impl TypeRef for TypeSymbol {}

impl Display for TypeSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.fqn {
            Some(fqn) => Display::fmt(fqn, f),
            None => Display::fmt(&self.id, f),
        }
    }
}

impl Translate<TypeSymbol> for SemId {
    type Builder = ();
    type Context = SymbolicSys;
    type Error = Error;

    fn translate(
        self,
        _builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<TypeSymbol, Self::Error> {
        match ctx.lookup(self) {
            None => Ok(TypeSymbol::unnamed(self)),
            Some(fqn) => Ok(TypeSymbol::with(self, fqn.clone())),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SystemBuilder {
    pending_deps: BTreeSet<Dependency>,
    imported_deps: BTreeSet<Dependency>,
    types: BTreeMap<SemId, SymTy>,
}

impl SystemBuilder {
    pub fn new() -> SystemBuilder { SystemBuilder::default() }

    pub fn import(mut self, lib: TypeLib) -> Result<Self, Error> {
        let dependency = Dependency::from(&lib);
        self.pending_deps.remove(&dependency);
        self.imported_deps.insert(dependency);
        self.pending_deps
            .extend(lib.dependencies.into_iter().filter(|dep| !self.imported_deps.contains(dep)));

        for (ty_name, ty) in lib.types {
            let id = ty.sem_id_named(&ty_name);
            let ty = ty.translate(&mut self, &())?;
            let info = SymTy::named(lib.name.clone(), ty_name.clone(), ty);
            self.types.insert(id, info);
        }

        Ok(self)
    }

    pub fn finalize(self) -> Result<SymbolicSys, Vec<Error>> {
        let mut errors = vec![];

        for dep in self.pending_deps {
            errors.push(Error::AbsentImport(dep));
        }

        for (sem_id, info) in &self.types {
            for (inner_id, _) in info.ty.type_refs() {
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

        SymbolicSys::with(self.imported_deps, self.types).map_err(|err| vec![err])
    }

    #[allow(clippy::multiple_bound_locations)]
    fn translate_inline<Ref: LibSubref>(&mut self, inline_ty: Ty<Ref>) -> Result<SemId, Error>
    where Ref: Translate<SemId, Context = (), Builder = SystemBuilder, Error = Error> {
        // compute id
        let id = inline_ty.sem_id_unnamed();
        // run for nested types
        let ty = inline_ty.translate(self, &())?;
        // add to system
        self.types.insert(id, SymTy::unnamed(ty));
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
        _builder: &mut Self::Builder,
        _ctx: &Self::Context,
    ) -> Result<SemId, Self::Error> {
        match self {
            InlineRef2::Named(sem_id) => Ok(sem_id),
            InlineRef2::Extern(ExternRef { sem_id, .. }) => Ok(sem_id),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// required dependency `{0}` was not imported into the builder.
    AbsentImport(Dependency),

    /// type `{new}` is already exists in the type system as `{present}`.
    RepeatedType {
        new: TypeSymbol,
        present: TypeSymbol,
    },

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
