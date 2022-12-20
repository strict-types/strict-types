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

use amplify::confinement;
use amplify::confinement::{Confined, SmallOrdMap};

use crate::ast::{NestedRef, TranslateError};
use crate::typelib::{Dependency, InlineRef, LibAlias, LibName, LibRef, TypeIndex, TypeLib};
use crate::{SemId, StenType, Translate, Ty, TypeName};

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum Warning {
    /// unused import `{0}` for `{1}`
    UnusedImport(LibAlias, Dependency),

    /// type {1} from library {0} with id {2} is already known
    RepeatedType(LibAlias, TypeName, SemId),
}

#[derive(Clone, Eq, PartialEq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// type library {0} is not imported.
    UnknownLib(LibAlias),

    /// type {2} is not present in the type library {0}. The current version of the library is {1},
    /// perhaps you need to import a different version.
    TypeAbsent(LibAlias, Dependency, TypeName),

    #[from]
    #[display(inner)]
    Confinement(confinement::Error),

    /// type {name} in {dependency} expected to have a type id {expected} but {found} is found.
    /// Perhaps a wrong version of the library is used?
    TypeMismatch {
        dependency: Dependency,
        name: TypeName,
        expected: SemId,
        found: SemId,
    },
}

#[derive(Default)]
pub struct LibBuilder {
    index: TypeIndex,
    types: SmallOrdMap<TypeName, Ty<LibRef>>,
}

impl LibBuilder {
    pub(crate) fn with(index: TypeIndex) -> LibBuilder {
        LibBuilder {
            index,
            types: default!(),
        }
    }

    pub(crate) fn finalize(self, name: LibName) -> Result<TypeLib, confinement::Error> {
        let types = Confined::try_from(self.types.into_inner())?;
        Ok(TypeLib {
            name,
            dependencies: none!(),
            types,
        })
    }
}

impl Translate<TypeLib> for StenType {
    type Context = LibName;
    type Error = TranslateError;

    fn translate(self, lib_name: &mut Self::Context) -> Result<TypeLib, Self::Error> {
        let id = self.id();

        let index = self.build_index()?;

        let mut builder = LibBuilder::with(index);
        let root = self.ty.translate(&mut builder)?;

        let name = builder.index.remove(&id).ok_or(TranslateError::UnknownId(id))?;
        let mut lib = builder.finalize(lib_name.clone())?;
        if lib.types.insert(name.clone(), root)?.is_some() {
            return Err(TranslateError::DuplicateName(name));
        }

        Ok(lib)
    }
}

impl Translate<LibRef> for StenType {
    type Context = LibBuilder;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<LibRef, Self::Error> {
        let id = self.id();
        let builtin = self.is_builtin();
        let ty = self.into_ty().translate(ctx)?;
        Ok(match ctx.index.get(&id) {
            Some(name) if !builtin => {
                if !ctx.types.contains_key(name) {
                    ctx.types.insert(name.clone(), ty)?;
                }
                LibRef::Named(name.clone(), id)
            }
            _ => LibRef::Inline(ty.translate(&mut ())?),
        })
    }
}

impl Translate<InlineRef> for LibRef {
    type Context = ();
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<InlineRef, Self::Error> {
        match self {
            LibRef::Named(ty_name, id) => Ok(InlineRef::Named(ty_name, id)),
            LibRef::Extern(ty_name, lib_alias, id) => Ok(InlineRef::Extern(ty_name, lib_alias, id)),
            LibRef::Inline(ty) => Ok(InlineRef::Builtin(ty.translate(ctx)?)),
        }
    }
}

impl Translate<SemId> for InlineRef {
    type Context = ();
    type Error = TranslateError;

    fn translate(self, _: &mut Self::Context) -> Result<SemId, Self::Error> {
        match self {
            InlineRef::Builtin(ref ty) if ty.is_builtin() => Ok(ty.id(None)),
            InlineRef::Builtin(_) => Err(TranslateError::NestedInline(self.to_string())),
            InlineRef::Named(_, id) | InlineRef::Extern(_, _, id) => Ok(id),
        }
    }
}

impl StenType {
    pub fn build_index(&self) -> Result<TypeIndex, TranslateError> {
        let mut index = empty!();
        self.update_index(&mut index).map(|_| index)
    }

    pub fn update_index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
        if let Some(name) = self.name.clone() {
            let id = self.id();
            match index.get(&id) {
                None => index.insert(id, name),
                Some(n) if n != &name => {
                    return Err(TranslateError::MultipleNames(id, n.clone(), name))
                }
                _ => None,
            };
        }

        self.ty.update_index(index)?;

        Ok(())
    }
}

impl Ty<StenType> {
    pub fn update_index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
        match self {
            Ty::Union(fields) => {
                for ty in fields.values() {
                    ty.update_index(index)?;
                }
            }
            Ty::Struct(fields) => {
                for ty in fields.values() {
                    ty.update_index(index)?;
                }
            }
            Ty::Array(ty, _) | Ty::List(ty, _) | Ty::Set(ty, _) | Ty::Map(_, ty, _) => {
                ty.update_index(index)?
            }
            _ => {}
        }
        Ok(())
    }
}
