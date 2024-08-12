// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2024 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2024 UBIDECO Institute
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

use amplify::confinement::Confined;

use crate::ast::{Field, NamedFields, UnionVariants, UnnamedFields};
use crate::{Ty, TypeRef};

pub trait Translate<To: Sized> {
    type Context;
    type Builder;
    type Error;

    fn translate(self, builder: &mut Self::Builder, ctx: &Self::Context)
    -> Result<To, Self::Error>;
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<Ty<ToRef>> for Ty<Ref>
where Ref: Translate<ToRef>
{
    type Builder = <Ref as Translate<ToRef>>::Builder;
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(
        self,
        builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<Ty<ToRef>, Self::Error> {
        Ok(match self {
            Ty::Primitive(prim) => Ty::Primitive(prim),
            Ty::Enum(vars) => Ty::Enum(vars),
            Ty::Union(fields) => Ty::Union(fields.translate(builder, ctx)?),
            Ty::Struct(fields) => Ty::Struct(fields.translate(builder, ctx)?),
            Ty::Tuple(fields) => Ty::Tuple(fields.translate(builder, ctx)?),
            Ty::Array(ty, len) => Ty::Array(ty.translate(builder, ctx)?, len),
            Ty::UnicodeChar => Ty::UnicodeChar,
            Ty::List(ty, sizing) => Ty::List(ty.translate(builder, ctx)?, sizing),
            Ty::Set(ty, sizing) => Ty::Set(ty.translate(builder, ctx)?, sizing),
            Ty::Map(key, ty, sizing) => {
                Ty::Map(key.translate(builder, ctx)?, ty.translate(builder, ctx)?, sizing)
            }
        })
    }
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<UnionVariants<ToRef>> for UnionVariants<Ref>
where Ref: Translate<ToRef>
{
    type Builder = <Ref as Translate<ToRef>>::Builder;
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(
        self,
        builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<UnionVariants<ToRef>, Self::Error> {
        let mut variants = BTreeMap::new();
        for (variant, ty) in self {
            variants.insert(variant, ty.translate(builder, ctx)?);
        }
        Ok(Confined::from_checked(variants).into())
    }
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<NamedFields<ToRef>> for NamedFields<Ref>
where Ref: Translate<ToRef>
{
    type Builder = <Ref as Translate<ToRef>>::Builder;
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(
        self,
        builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<NamedFields<ToRef>, Self::Error> {
        let mut fields = Vec::with_capacity(self.len());
        for field in self {
            fields.push(Field {
                name: field.name,
                ty: field.ty.translate(builder, ctx)?,
            });
        }
        Ok(NamedFields::try_from(fields).expect("re-packing existing fields structure"))
    }
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<UnnamedFields<ToRef>> for UnnamedFields<Ref>
where Ref: Translate<ToRef>
{
    type Builder = <Ref as Translate<ToRef>>::Builder;
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(
        self,
        builder: &mut Self::Builder,
        ctx: &Self::Context,
    ) -> Result<UnnamedFields<ToRef>, Self::Error> {
        let mut fields = Vec::with_capacity(self.len());
        for ty in self {
            fields.push(ty.translate(builder, ctx)?);
        }
        Ok(UnnamedFields::try_from(fields).expect("re-packing existing fields structure"))
    }
}
