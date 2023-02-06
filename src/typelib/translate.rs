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

use amplify::confinement;
use amplify::confinement::{SmallOrdMap, TinyOrdMap};
use encoding::LibName;
use strict_encoding::{InvalidIdent, TypeName};

use crate::ast::{Field, NamedFields, UnionVariants, UnnamedFields};
use crate::typelib::{
    CompileRef, CompileType, Dependency, InlineRef, InlineRef1, InlineRef2, LibRef,
};
use crate::{KeyTy, SemId, Ty, TypeRef};

pub type TypeIndex = BTreeMap<TypeName, SemId>;

pub trait Translate<To: Sized> {
    type Context;
    type Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<To, Self::Error>;
}

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum TranslateError {
    /// invalid library name; {0}
    InvalidLibName(InvalidIdent),

    /// a different type with name `{0}` is already present
    DuplicateName(TypeName),

    /// a type with id {0} has at least two different names `{0}` and `{1}`
    MultipleNames(SemId, TypeName, TypeName),

    /// unknown type with id `{0}`
    UnknownId(SemId),

    /// type `{unknown}` referenced inside `{within}` is not known
    UnknownType {
        unknown: TypeName,
        within: CompileType,
    },

    /// return type indicating continue operation
    Continue,

    /// dependency {0} is already present in the library
    DuplicatedDependency(Dependency),

    #[from]
    #[display(inner)]
    Confinement(confinement::Error),

    /// too deep type nesting for type {2} inside {0}, path {1}
    NestedInline(TypeName, String, String),

    /// no type {1} in library {0}
    UnknownExtern(LibName, TypeName),

    /// unknown library {0}
    UnknownLib(LibName),
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
            Ty::Tuple(fields) => Ty::Tuple(fields.translate(ctx)?),
            Ty::Array(ty, len) => Ty::Array(ty.translate(ctx)?, len),
            Ty::UnicodeChar => Ty::UnicodeChar,
            Ty::List(ty, sizing) => Ty::List(ty.translate(ctx)?, sizing),
            Ty::Set(ty, sizing) => Ty::Set(ty.translate(ctx)?, sizing),
            Ty::Map(key, ty, sizing) => Ty::Map(key, ty.translate(ctx)?, sizing),
        })
    }
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<UnionVariants<ToRef>> for UnionVariants<Ref>
where Ref: Translate<ToRef>
{
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<UnionVariants<ToRef>, Self::Error> {
        let mut variants = BTreeMap::new();
        for (variant, ty) in self {
            variants.insert(variant, ty.translate(ctx)?);
        }
        Ok(UnionVariants::try_from(variants).expect("re-packing existing fields structure"))
    }
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<NamedFields<ToRef>> for NamedFields<Ref>
where Ref: Translate<ToRef>
{
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<NamedFields<ToRef>, Self::Error> {
        let mut fields = Vec::with_capacity(self.len());
        for field in self {
            fields.push(Field {
                name: field.name,
                ty: field.ty.translate(ctx)?,
            });
        }
        Ok(NamedFields::try_from(fields).expect("re-packing existing fields structure"))
    }
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<UnnamedFields<ToRef>> for UnnamedFields<Ref>
where Ref: Translate<ToRef>
{
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<UnnamedFields<ToRef>, Self::Error> {
        let mut fields = Vec::with_capacity(self.len());
        for ty in self {
            fields.push(ty.translate(ctx)?);
        }
        Ok(UnnamedFields::try_from(fields).expect("re-packing existing fields structure"))
    }
}

pub struct NestedContext {
    pub top_name: TypeName,
    pub index: TypeIndex,
    pub extern_types: TinyOrdMap<LibName, SmallOrdMap<TypeName, SemId>>,
    pub stack: Vec<String>,
}

impl Translate<LibRef> for CompileRef {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<LibRef, Self::Error> {
        match self {
            CompileRef::Embedded(ty) => {
                ctx.stack.push(ty.cls().to_string());
                let res = ty.translate(ctx).map(LibRef::Inline);
                ctx.stack.pop();
                res
            }
            CompileRef::Named(name) => {
                let id = ctx.index.get(&name).ok_or(TranslateError::Continue)?;
                Ok(LibRef::Named(name, *id))
            }
            CompileRef::Extern(name, lib, id) => Ok(LibRef::Extern(name, lib, id)),
        }
    }
}

impl Translate<InlineRef> for CompileRef {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<InlineRef, Self::Error> {
        match self {
            CompileRef::Embedded(ty) => {
                ctx.stack.push(ty.cls().to_string());
                let res = ty.translate(ctx).map(InlineRef::Inline);
                ctx.stack.pop();
                res
            }
            CompileRef::Named(name) => {
                let id = ctx.index.get(&name).ok_or(TranslateError::Continue)?;
                Ok(InlineRef::Named(name, *id))
            }
            CompileRef::Extern(name, lib, id) => Ok(InlineRef::Extern(name, lib, id)),
        }
    }
}

impl Translate<InlineRef1> for CompileRef {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<InlineRef1, Self::Error> {
        match self {
            CompileRef::Embedded(ty) => {
                ctx.stack.push(ty.cls().to_string());
                let res = ty.translate(ctx).map(InlineRef1::Inline);
                ctx.stack.pop();
                res
            }
            CompileRef::Named(name) => {
                let id = ctx.index.get(&name).ok_or(TranslateError::Continue)?;
                Ok(InlineRef1::Named(name, *id))
            }
            CompileRef::Extern(name, lib, id) => Ok(InlineRef1::Extern(name, lib, id)),
        }
    }
}

impl Translate<InlineRef2> for CompileRef {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<InlineRef2, Self::Error> {
        match self {
            CompileRef::Embedded(ty) => {
                ctx.stack.push(ty.cls().to_string());
                let res = ty.translate(ctx).map(InlineRef2::Inline);
                ctx.stack.pop();
                res
            }
            CompileRef::Named(name) => {
                let id = ctx.index.get(&name).ok_or(TranslateError::Continue)?;
                Ok(InlineRef2::Named(name, *id))
            }
            CompileRef::Extern(name, lib, id) => Ok(InlineRef2::Extern(name, lib, id)),
        }
    }
}

impl Translate<KeyTy> for CompileRef {
    type Context = NestedContext;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<KeyTy, Self::Error> {
        let me = self.to_string();
        match self {
            CompileRef::Embedded(ty) => ty.try_to_key().ok(),
            CompileRef::Named(_) | CompileRef::Extern(_, _, _) => None,
        }
        .ok_or(TranslateError::NestedInline(ctx.top_name.clone(), ctx.stack.join("/"), me))
    }
}
