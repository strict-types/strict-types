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
use crate::typelib::{
    BuiltinRef, Dependency, InlineRef, LibAlias, LibName, LibRef, TypeIndex, TypeLib,
};
use crate::{KeyTy, SemId, StenType, Translate, Ty, TypeName};

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

        let name = self.name.clone().ok_or_else(|| TranslateError::UnnamedTopType(self.clone()))?;
        let index = self.build_index()?;

        let builder = LibBuilder::with(index);
        let mut context = NestedContext {
            top_name: name,
            builder,
            stack: empty!(),
        };

        let root = self.ty.translate(&mut context)?;

        let name = context.builder.index.remove(&id).ok_or(TranslateError::UnknownId(id))?;
        let mut lib = context.builder.finalize(lib_name.clone())?;
        if lib.types.insert(name, root)?.is_some() {
            return Err(TranslateError::DuplicateName(context.top_name));
        }

        Ok(lib)
    }
}

pub struct NestedContext {
    top_name: TypeName,
    builder: LibBuilder,
    stack: Vec<String>,
}

impl Translate<LibRef> for StenType {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<LibRef, Self::Error> {
        let id = self.id();
        let builtin = self.is_builtin();

        ctx.stack.push(
            self.name.as_ref().map(TypeName::to_string).unwrap_or_else(|| self.ty.to_string()),
        );

        let ty = self.into_ty().translate(ctx)?;

        let lib_ref = match ctx.builder.index.get(&id) {
            Some(name) if !builtin => {
                if !ctx.builder.types.contains_key(name) {
                    ctx.builder.types.insert(name.clone(), ty)?;
                }
                LibRef::Named(name.clone(), id)
            }
            _ => LibRef::Inline(ty.translate(ctx)?),
        };

        ctx.stack.pop();

        Ok(lib_ref)
    }
}

impl Translate<InlineRef> for LibRef {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<InlineRef, Self::Error> {
        match self {
            LibRef::Named(ty_name, id) => Ok(InlineRef::Named(ty_name, id)),
            LibRef::Extern(ty_name, lib_alias, id) => Ok(InlineRef::Extern(ty_name, lib_alias, id)),
            LibRef::Inline(ty) => Ok(InlineRef::Builtin(ty.translate(ctx)?)),
        }
    }
}

impl Translate<BuiltinRef> for InlineRef {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<BuiltinRef, Self::Error> {
        match self {
            InlineRef::Builtin(ty) => ty.translate(ctx).map(BuiltinRef::Builtin),
            InlineRef::Named(ty_name, id) => Ok(BuiltinRef::Named(ty_name, id)),
            InlineRef::Extern(ty_name, lib_alias, id) => {
                Ok(BuiltinRef::Extern(ty_name, lib_alias, id))
            }
        }
    }
}

impl Translate<KeyTy> for BuiltinRef {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<KeyTy, Self::Error> {
        // We are too deep
        Err(TranslateError::NestedInline(
            ctx.top_name.clone(),
            ctx.stack.join("/"),
            self.to_string(),
        ))
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
