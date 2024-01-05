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

use std::io;

use encoding::{StrictType, TypedParent, Variant};
use strict_encoding::{
    DefineEnum, DefineStruct, DefineTuple, DefineUnion, FieldName, LibName, Primitive, Sizing,
    StrictDumb, StrictEncode, StrictEnum, StrictStruct, StrictTuple, StrictUnion, TypeName,
    TypedWrite, VariantName, WriteEnum, WriteStruct, WriteTuple, WriteUnion,
};

use crate::ast::{EnumVariants, Field, NamedFields, UnionVariants, UnnamedFields};
use crate::typetree::TyInner;
use crate::{Ty, TypeTree};

#[derive(Debug)]
pub struct TreeBuilder {
    lib: LibName,
    name: Option<TypeName>,
    ty: Option<Ty<TyInner>>,
    next_tag: u8,
}

impl TreeBuilder {
    fn new(lib: LibName) -> Self {
        TreeBuilder {
            lib,
            name: None,
            ty: None,
            next_tag: 0,
        }
    }

    fn with<T: StrictType>() -> Self {
        TreeBuilder {
            lib: libname!(T::STRICT_LIB_NAME),
            name: T::strict_name(),
            ty: None,
            next_tag: 0,
        }
    }

    fn next_tag(&mut self) -> u8 {
        let tag = self.next_tag;
        self.next_tag = self.next_tag.saturating_add(1);
        tag
    }

    fn insert_union_variant(&mut self, variant: Variant, ty: TyInner) {
        if self.ty.is_none() {
            self.ty = Some(Ty::Union(UnionVariants::with(variant, ty)));
        } else if let Some(Ty::Union(variants)) = &mut self.ty {
            variants.insert(variant, ty).expect("too many union variants");
        } else {
            panic!("{:?} is not a union", self.ty)
        }
    }
}

impl TypeTree {
    pub fn transpile<T: StrictEncode + StrictDumb>() -> Self {
        let builder = TreeBuilder::with::<T>();
        let builder =
            T::strict_dumb().strict_encode(builder).expect("memory encoding doesn't error");
        TypeTree {
            name: builder.name,
            ty: builder.ty.expect("builder is not finished"),
        }
    }
}

impl TyInner {
    fn build<T: StrictEncode>(t: &T) -> Self {
        let builder = TreeBuilder::with::<T>();
        t.strict_encode(builder).expect("memory encoding doesn't error").into()
    }

    fn new(lib: &LibName, ty: Ty<TyInner>) -> Self {
        TyInner {
            lib: lib.clone(),
            name: None,
            ty: Box::new(ty),
        }
    }
}

impl From<TreeBuilder> for TyInner {
    fn from(builder: TreeBuilder) -> Self {
        TyInner {
            lib: builder.lib,
            name: builder.name,
            ty: Box::new(builder.ty.expect("builder is not finished")),
        }
    }
}

impl TypedParent for TreeBuilder {}

impl TypedWrite for TreeBuilder {
    type TupleWriter = TreeBuilder;
    type StructWriter = TreeBuilder;
    type UnionDefiner = TreeBuilder;

    fn write_union<T: StrictUnion>(
        self,
        inner: impl FnOnce(Self::UnionDefiner) -> io::Result<Self>,
    ) -> io::Result<Self> {
        inner(self)
    }

    fn write_enum<T: StrictEnum>(mut self, value: T) -> io::Result<Self>
    where u8: From<T> {
        for (_, name) in T::ALL_VARIANTS {
            self = self.define_variant(fname!(*name));
        }
        self = DefineEnum::complete(self);
        self = self.write_variant(vname!(value.variant_name()))?;
        Ok(WriteEnum::complete(self))
    }

    fn write_tuple<T: StrictTuple>(
        self,
        inner: impl FnOnce(Self::TupleWriter) -> io::Result<Self>,
    ) -> io::Result<Self> {
        inner(self)
    }

    fn write_struct<T: StrictStruct>(
        self,
        inner: impl FnOnce(Self::StructWriter) -> io::Result<Self>,
    ) -> io::Result<Self> {
        inner(self)
    }

    unsafe fn register_primitive(mut self, prim: Primitive) -> Self {
        self.ty = Some(Ty::Primitive(prim));
        self
    }

    unsafe fn register_array(mut self, ty: &impl StrictEncode, len: u16) -> Self {
        let ty = TyInner::build(ty);
        self.ty = Some(Ty::Array(ty, len));
        self
    }

    unsafe fn register_unicode(mut self, sizing: Sizing) -> Self {
        let ty = TyInner::new(&self.lib, Ty::UnicodeChar);
        self.ty = Some(Ty::List(ty, sizing));
        self
    }

    unsafe fn register_list(mut self, ty: &impl StrictEncode, sizing: Sizing) -> Self {
        let ty = TyInner::build(ty);
        self.ty = Some(Ty::List(ty, sizing));
        self
    }

    unsafe fn register_set(mut self, ty: &impl StrictEncode, sizing: Sizing) -> Self {
        let ty = TyInner::build(ty);
        self.ty = Some(Ty::Set(ty, sizing));
        self
    }

    unsafe fn register_map(
        mut self,
        key: &impl StrictEncode,
        ty: &impl StrictEncode,
        sizing: Sizing,
    ) -> Self {
        let key_ty = TyInner::build(key);
        let val_ty = TyInner::build(ty);
        self.ty = Some(Ty::Map(key_ty, val_ty, sizing).into());
        self
    }

    unsafe fn _write_raw<const LEN: usize>(self, _bytes: impl AsRef<[u8]>) -> io::Result<Self> {
        // TODO: Collect information on which raw data are written
        Ok(self)
    }
}

impl TreeBuilder {
    fn _define_struct_field(mut self, name: FieldName, value: &impl StrictEncode) -> Self {
        let ty = TyInner::build(value);
        let field = Field::new(name, ty);
        if self.ty.is_none() {
            self.ty = Some(Ty::Struct(NamedFields::with(field)));
        } else if let Some(Ty::Struct(fields)) = &mut self.ty {
            fields.push(field).expect("too many struct fields");
        } else {
            panic!("{:?} is not a structure", self.ty)
        }
        self
    }

    fn _define_tuple_field(mut self, value: &impl StrictEncode) -> Self {
        let ty = TyInner::build(&value);
        if self.ty.is_none() {
            self.ty = Some(Ty::Tuple(UnnamedFields::with(ty)));
        } else if let Some(Ty::Tuple(fields)) = &mut self.ty {
            fields.push(ty).expect("too many tuple fields");
        } else {
            panic!("{:?} is not a tuple", self.ty)
        }
        self
    }
}

impl DefineStruct for TreeBuilder {
    type Parent = TreeBuilder;

    fn define_field<T: StrictEncode + StrictDumb>(self, name: FieldName) -> Self {
        self._define_struct_field(name, &T::strict_dumb())
    }

    fn complete(self) -> Self::Parent { self }
}

impl WriteStruct for TreeBuilder {
    type Parent = TreeBuilder;

    fn write_field(self, name: FieldName, value: &impl StrictEncode) -> io::Result<Self> {
        Ok(self._define_struct_field(name, value))
    }

    fn complete(self) -> Self::Parent { self }
}

impl DefineTuple for TreeBuilder {
    type Parent = TreeBuilder;

    fn define_field<T: StrictEncode + StrictDumb>(self) -> Self {
        self._define_tuple_field(&T::strict_dumb())
    }
    fn complete(self) -> Self::Parent { self }
}

impl WriteTuple for TreeBuilder {
    type Parent = TreeBuilder;

    fn write_field(self, value: &impl StrictEncode) -> io::Result<Self> {
        Ok(self._define_tuple_field(value))
    }

    fn complete(self) -> Self::Parent { self }
}

impl DefineEnum for TreeBuilder {
    type Parent = TreeBuilder;
    type EnumWriter = TreeBuilder;

    fn define_variant(mut self, name: VariantName) -> Self {
        let tag = self.next_tag();
        let variant = Variant::named(tag, name);
        if self.ty.is_none() {
            self.ty = Some(Ty::Enum(EnumVariants::with(variant)));
        } else if let Some(Ty::Enum(variants)) = &mut self.ty {
            variants.push(variant).expect("too many enum variants");
        } else {
            panic!("{:?} is not an enum", self.ty)
        }
        self
    }

    fn complete(self) -> Self::EnumWriter { self }
}

impl WriteEnum for TreeBuilder {
    type Parent = TreeBuilder;

    fn write_variant(self, _name: VariantName) -> io::Result<Self> { Ok(self) }

    fn complete(self) -> Self::Parent { self }
}

impl DefineUnion for TreeBuilder {
    type Parent = TreeBuilder;
    type TupleDefiner = TreeBuilder;
    type StructDefiner = TreeBuilder;
    type UnionWriter = TreeBuilder;

    fn define_unit(mut self, name: VariantName) -> Self {
        let tag = self.next_tag();
        let ty = TyInner::build(&());
        let variant = Variant::named(tag, name);
        self.insert_union_variant(variant, ty);
        self
    }

    fn define_tuple(
        mut self,
        name: VariantName,
        inner: impl FnOnce(Self::TupleDefiner) -> Self,
    ) -> Self {
        let tag = self.next_tag();
        let builder = TreeBuilder::new(self.lib.clone());
        let builder = inner(builder);
        let ty = builder.into();
        let variant = Variant::named(tag, name);
        self.insert_union_variant(variant, ty);
        self
    }

    fn define_struct(
        mut self,
        name: VariantName,
        inner: impl FnOnce(Self::StructDefiner) -> Self,
    ) -> Self {
        let tag = self.next_tag();
        let builder = TreeBuilder::new(self.lib.clone());
        let builder = inner(builder);
        let ty = builder.into();
        let variant = Variant::named(tag, name);
        self.insert_union_variant(variant, ty);
        self
    }

    fn complete(self) -> Self::UnionWriter { self }
}

impl WriteUnion for TreeBuilder {
    type Parent = TreeBuilder;
    type TupleWriter = TreeBuilder;
    type StructWriter = TreeBuilder;

    fn write_unit(self, _name: VariantName) -> io::Result<Self> { Ok(self) }

    fn write_tuple(
        self,
        _name: VariantName,
        _inner: impl FnOnce(Self::TupleWriter) -> io::Result<Self>,
    ) -> io::Result<Self> {
        Ok(self)
    }

    fn write_struct(
        self,
        _name: VariantName,
        _inner: impl FnOnce(Self::StructWriter) -> io::Result<Self>,
    ) -> io::Result<Self> {
        Ok(self)
    }

    fn complete(self) -> Self::Parent { self }
}
