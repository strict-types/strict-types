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

use amplify::confinement;

use crate::ast::Fields;
use crate::{SemId, Ty, TypeName, TypeRef};

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
    MultipleNames(SemId, TypeName, TypeName),

    /// unknown type with id `{0}`
    UnknownId(SemId),

    #[from]
    #[display(inner)]
    Confinement(confinement::Error),

    /// too deep type nesting for type {ty} in {nested_in}
    NestedInline {
        nested_in: String,
        ty: String,
    },
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<Ty<ToRef>> for Ty<Ref>
where Ref: Translate<ToRef>
{
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<Ty<ToRef>, Self::Error> {
        Ok(match self {
            Ty::Primitive(prim) => Ty::Primitive(prim),
            Ty::Enum(vars) => Ty::Enum(vars),
            Ty::Union(fields) => Ty::Union(fields.translate(ctx)?),
            Ty::Struct(fields) => Ty::Struct(fields.translate(ctx)?),
            Ty::Array(ty, len) => Ty::Array(ty.translate(ctx)?, len),
            Ty::UnicodeChar => Ty::UnicodeChar,
            Ty::List(ty, sizing) => Ty::List(ty.translate(ctx)?, sizing),
            Ty::Set(ty, sizing) => Ty::Set(ty.translate(ctx)?, sizing),
            Ty::Map(key, ty, sizing) => Ty::Map(key, ty.translate(ctx)?, sizing),
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
