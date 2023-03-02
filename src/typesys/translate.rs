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

use amplify::confinement;

use crate::typelib::{ExternRef, InlineRef, InlineRef1, InlineRef2};
use crate::typesys::TypeFqid;
use crate::{Dependency, KeyTy, LibRef, SemId, Translate, Ty, TypeLib, TypeRef, TypeSystem};

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SystemBuilder {
    dependencies: BTreeSet<Dependency>,
    types: BTreeMap<TypeFqid, Ty<SemId>>,
}

impl SystemBuilder {
    pub fn new() -> SystemBuilder { SystemBuilder::default() }

    pub fn import(&mut self, lib: TypeLib) -> Result<usize, Error> {
        self.dependencies.retain(|dep| dep.name != lib.name);

        let init_len = self.types.len();
        for (ty_name, ty) in lib.types {
            let id = ty.id(Some(&ty_name));
            let ty = ty.translate(self)?;
            let fqid = TypeFqid::named(id, lib.name.clone(), ty_name.clone());
            self.types.insert(fqid, ty);
        }

        self.dependencies.extend(lib.dependencies);

        Ok(self.types.len() - init_len)
    }

    pub fn finalize(self) -> Result<TypeSystem, Vec<Error>> {
        let mut errors = vec![];
        if let Some(dep) = self.dependencies.first() {
            errors.push(Error::UnusedImport(dep.clone()));
        }
        for (fqid, ty) in &self.types {
            for inner_id in ty.type_refs() {
                if !self.types.contains_key(&fqid) {
                    errors.push(Error::TypeAbsent {
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
    where Ref: Translate<SemId, Context = SystemBuilder, Error = Error> {
        // compute id
        let id = inline_ty.id(None);
        // run for nested types
        let ty = inline_ty.translate(self)?;
        // add to system
        self.types.insert(TypeFqid::unnamed(id), ty);
        Ok(id)
    }
}

impl Translate<SemId> for LibRef {
    type Context = SystemBuilder;
    type Error = Error;

    fn translate(self, sys: &mut Self::Context) -> Result<SemId, Self::Error> {
        match self {
            LibRef::Named(_, id) => Ok(id),
            LibRef::Inline(inline_ty) => sys.translate_inline(inline_ty),
            LibRef::Extern(ExternRef { id, .. }) => Ok(id),
        }
    }
}

impl Translate<SemId> for InlineRef {
    type Context = SystemBuilder;
    type Error = Error;

    fn translate(self, sys: &mut Self::Context) -> Result<SemId, Self::Error> {
        match self {
            InlineRef::Named(_, id) => Ok(id),
            InlineRef::Inline(inline_ty) => sys.translate_inline(inline_ty),
            InlineRef::Extern(ExternRef { id, .. }) => Ok(id),
        }
    }
}

impl Translate<SemId> for InlineRef1 {
    type Context = SystemBuilder;
    type Error = Error;

    fn translate(self, sys: &mut Self::Context) -> Result<SemId, Self::Error> {
        match self {
            InlineRef1::Named(_, id) => Ok(id),
            InlineRef1::Inline(inline_ty) => sys.translate_inline(inline_ty),
            InlineRef1::Extern(ExternRef { id, .. }) => Ok(id),
        }
    }
}

impl Translate<SemId> for InlineRef2 {
    type Context = SystemBuilder;
    type Error = Error;

    fn translate(self, sys: &mut Self::Context) -> Result<SemId, Self::Error> {
        match self {
            InlineRef2::Named(_, id) => Ok(id),
            InlineRef2::Inline(inline_ty) => sys.translate_inline(inline_ty),
            InlineRef2::Extern(ExternRef { id, .. }) => Ok(id),
        }
    }
}

impl Translate<SemId> for KeyTy {
    type Context = SystemBuilder;
    type Error = Error;

    fn translate(self, _sys: &mut Self::Context) -> Result<SemId, Self::Error> {
        Err(Error::TooDeep)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// unused import `{0}`.
    UnusedImport(Dependency),

    /// type `{unknown}` referenced from `{known}` is not known; perhaps you need to import a
    /// library defining this type.
    TypeAbsent { unknown: SemId, known: SemId },

    #[from]
    #[display(inner)]
    Confinement(confinement::Error),

    /// Too deeply nested types.
    TooDeep,
}
