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

use amplify::{confinement, Wrapper};

use crate::ast::{Fields, TyInner};
use crate::{Ty, TyId, TypeName, TypeRef};

pub trait Translate<To: Sized> {
    type Context;
    type Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<To, Self::Error>;
}

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum TranslateError {
    InvalidLibName(String),

    /// a different type with name `{0}` is already present
    DuplicateName(TypeName),

    /// a type with id {0} has at least two different names `{0}` and `{1}`
    MultipleNames(TyId, TypeName, TypeName),

    /// unknown type with id `{0}`
    UnknownId(TyId),

    #[from]
    #[display(inner)]
    Confinement(confinement::Error),
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<Ty<ToRef>> for Ty<Ref>
where Ref: Translate<ToRef>
{
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<Ty<ToRef>, Self::Error> {
        Ok(match self.into_inner() {
            TyInner::Primitive(prim) => TyInner::Primitive(prim),
            TyInner::Enum(vars) => TyInner::Enum(vars),
            TyInner::Union(fields) => TyInner::Union(fields.translate(ctx)?),
            TyInner::Struct(fields) => TyInner::Struct(fields.translate(ctx)?),
            TyInner::Array(ty, len) => TyInner::Array(ty.translate(ctx)?, len),
            TyInner::UnicodeChar => TyInner::UnicodeChar,
            TyInner::List(ty, sizing) => TyInner::List(ty.translate(ctx)?, sizing),
            TyInner::Set(ty, sizing) => TyInner::Set(ty.translate(ctx)?, sizing),
            TyInner::Map(key, ty, sizing) => TyInner::Map(key, ty.translate(ctx)?, sizing),
        }
        .into())
    }
}

impl<Ref: TypeRef, ToRef: TypeRef, const OP: bool> Translate<Fields<ToRef, OP>> for Fields<Ref, OP>
where Ref: Translate<ToRef>
{
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<Fields<ToRef, OP>, Self::Error> {
        let mut fields = BTreeMap::new();
        for (name, rf) in self {
            fields.insert(name, rf.translate(ctx)?);
        }
        Ok(Fields::try_from(fields).expect("re-packing existing fields structure"))
    }
}
