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

use encoding::{LibName, TypeName};

use crate::typelib::{ExternRef, ExternTypes, InlineRef, InlineRef1, InlineRef2};
use crate::{KeyTy, LibRef, SemId, SymbolRef, Translate, TranspileRef, Ty, TypeLibId, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum SymbolError {
    /// type with semantic id `{0}` is not known to the library
    UnknownType(SemId),

    /// unknown library reference `{0}`
    UnknownLib(TypeLibId),

    /// library `{0}` contains too many types.
    LibTooLarge(LibName),

    /// too many dependencies.
    TooManyDependencies,
}

pub struct SymbolContext {
    pub(super) reverse_index: BTreeMap<SemId, TypeName>,
    pub(super) lib_index: BTreeMap<TypeLibId, LibName>,
}

impl SymbolContext {
    fn embedded<Ref>(
        &self,
        builder: &mut ExternTypes,
        ty: Ty<Ref>,
    ) -> Result<TranspileRef, SymbolError>
    where
        Ref: TypeRef
            + Translate<TranspileRef, Builder = ExternTypes, Context = Self, Error = SymbolError>,
    {
        ty.translate(builder, self).map(Box::new).map(TranspileRef::Embedded)
    }

    fn named(&self, id: SemId) -> Result<TranspileRef, SymbolError> {
        self.reverse_index
            .get(&id)
            .ok_or(SymbolError::UnknownType(id))
            .cloned()
            .map(TranspileRef::Named)
    }

    fn external(
        &self,
        builder: &mut ExternTypes,
        ext: ExternRef,
    ) -> Result<TranspileRef, SymbolError> {
        let lib_name =
            self.lib_index.get(&ext.lib_id).ok_or(SymbolError::UnknownLib(ext.lib_id))?;
        let ty_name = builder
            .get(lib_name)
            .ok_or(SymbolError::UnknownLib(ext.lib_id.clone()))?
            .get(&ext.sem_id)
            .ok_or(SymbolError::UnknownType(ext.sem_id))?
            .clone();
        let r = SymbolRef::with(lib_name.clone(), ty_name.clone(), ext.lib_id, ext.sem_id);
        let mut index = builder.remove(&lib_name).ok().flatten().unwrap_or_default();
        index
            .insert(ext.sem_id, ty_name)
            .map_err(|_| SymbolError::LibTooLarge(lib_name.clone()))?;
        builder.insert(lib_name.clone(), index).map_err(|_| SymbolError::TooManyDependencies)?;
        Ok(TranspileRef::Extern(r))
    }
}

impl Translate<TranspileRef> for LibRef {
    type Context = SymbolContext;
    type Builder = ExternTypes;
    type Error = SymbolError;

    fn translate(
        self,
        builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<TranspileRef, Self::Error> {
        match self {
            LibRef::Inline(ty) => ctx.embedded(builder, ty),
            LibRef::Named(id) => ctx.named(id),
            LibRef::Extern(ext) => ctx.external(builder, ext),
        }
    }
}

impl Translate<TranspileRef> for InlineRef {
    type Context = SymbolContext;
    type Builder = ExternTypes;
    type Error = SymbolError;

    fn translate(
        self,
        builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<TranspileRef, Self::Error> {
        match self {
            InlineRef::Inline(ty) => ctx.embedded(builder, ty),
            InlineRef::Named(id) => ctx.named(id),
            InlineRef::Extern(ext) => ctx.external(builder, ext),
        }
    }
}

impl Translate<TranspileRef> for InlineRef1 {
    type Context = SymbolContext;
    type Builder = ExternTypes;
    type Error = SymbolError;

    fn translate(
        self,
        builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<TranspileRef, Self::Error> {
        match self {
            InlineRef1::Inline(ty) => ctx.embedded(builder, ty),
            InlineRef1::Named(id) => ctx.named(id),
            InlineRef1::Extern(ext) => ctx.external(builder, ext),
        }
    }
}

impl Translate<TranspileRef> for InlineRef2 {
    type Context = SymbolContext;
    type Builder = ExternTypes;
    type Error = SymbolError;

    fn translate(
        self,
        builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<TranspileRef, Self::Error> {
        match self {
            InlineRef2::Inline(ty) => ctx.embedded(builder, ty),
            InlineRef2::Named(id) => ctx.named(id),
            InlineRef2::Extern(ext) => ctx.external(builder, ext),
        }
    }
}

impl Translate<TranspileRef> for KeyTy {
    type Context = SymbolContext;
    type Builder = ExternTypes;
    type Error = SymbolError;

    fn translate(
        self,
        _builder: &mut Self::Builder,
        _ctx: &Self::Context,
    ) -> Result<TranspileRef, Self::Error> {
        Ok(TranspileRef::Embedded(Box::new(self.into_ty())))
    }
}
