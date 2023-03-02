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

use crate::ast::NestedRef;
use crate::typelib::{Dependency, Error, InlineRef, LibAlias, LibRef, TypeLib, Warning};
use crate::{EmbeddedRef, SemId, Translate, Ty, TypeName, TypeSystem};

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SystemBuilder {
    pub(super) dependencies: BTreeMap<LibAlias, Dependency>,
    types: BTreeMap<(LibAlias, TypeName), Ty<LibRef>>,
}

impl SystemBuilder {
    pub fn new() -> SystemBuilder { SystemBuilder::default() }

    pub fn import(&mut self, lib: TypeLib) {
        let alias = self
            .dependencies
            .iter()
            .find(|(_, d)| d.name == lib.name)
            .map(|(name, _)| name)
            .unwrap_or(&lib.name);
        let alias = alias.clone();
        self.dependencies.remove(&alias);
        self.types
            .extend(lib.types.into_iter().map(|(ty_name, ty)| ((alias.clone(), ty_name), ty)));
        self.dependencies.extend(lib.dependencies);
    }

    pub fn finalize(mut self) -> Result<(TypeSystem, Vec<Warning>), Vec<Error>> {
        let mut warnings: Vec<Warning> = empty!();
        let mut errors: Vec<Error> = empty!();
        let mut lib: BTreeSet<Ty<EmbeddedRef>> = empty!();

        for ((lib_alias, ty_name), ty) in self.types.clone() {
            match ty.clone().translate(&mut self) {
                Err(err) => errors.push(err),
                Ok(ty) => {
                    let id = ty.id(Some(&ty_name));
                    if !lib.insert(ty) {
                        warnings.push(Warning::RepeatedType(lib_alias, ty_name, id))
                    }
                }
            }
        }

        match TypeSystem::try_from_iter(lib) {
            Err(err) => {
                errors.push(err.into());
                return Err(errors);
            }
            Ok(lib) if errors.is_empty() => Ok((lib, warnings)),
            Ok(_) => Err(errors),
        }
    }
}

impl Translate<EmbeddedRef> for LibRef {
    type Context = SystemBuilder;
    type Error = Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<EmbeddedRef, Self::Error> {
        match self {
            LibRef::Named(_, id) => Ok(EmbeddedRef::SemId(id)),
            LibRef::Inline(inline_ty) => Ok(EmbeddedRef::Inline(inline_ty.translate(ctx)?)),
            LibRef::Extern(ty_name, lib_alias, id) => {
                let dep =
                    ctx.dependencies.get(&lib_alias).ok_or(Error::UnknownLib(lib_alias.clone()))?;
                let ty = ctx
                    .types
                    .get(&(lib_alias.clone(), ty_name.clone()))
                    .ok_or_else(|| Error::TypeAbsent(lib_alias, dep.clone(), ty_name.clone()))?;
                if id != ty.id() {
                    return Err(Error::TypeMismatch {
                        dependency: dep.clone(),
                        name: ty_name,
                        expected: id,
                        found: ty.id(),
                    });
                }
                Ok(EmbeddedRef::SemId(id))
            }
        }
    }
}

impl Translate<SemId> for InlineRef {
    type Context = SystemBuilder;
    type Error = Error;

    fn translate(self, _: &mut Self::Context) -> Result<SemId, Self::Error> {
        match self {
            InlineRef::Named(_, id) => Ok(id),
            InlineRef::Extern(_, _, id) => Ok(id),
        }
    }
}
