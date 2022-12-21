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

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use crate::ast::{Field, Fields, Variants};
use crate::encoding::{
    StrictEncode, ToIdent, ToMaybeIdent, TypedWrite, WriteEnum, WriteStruct, WriteTuple, WriteUnion,
};
use crate::{LibName, LibRef, Ty, TypeName};

pub struct TypeBuilder {}

impl TypeBuilder {}

impl TypedWrite for TypeBuilder {
    type TupleWriter = StructBuilder<Self, false>;
    type StructWriter = StructBuilder<Self, true>;
    type UnionWriter = UnionBuilder;
    type EnumWriter = EnumBuilder<Self>;

    fn write_tuple(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::TupleWriter {
        todo!()
    }

    fn write_struct(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::StructWriter {
        StructBuilder::with(ns.to_ident(), name.to_maybe_ident(), self)
    }

    fn write_union(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::UnionWriter {
        todo!()
    }

    fn write_enum(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::EnumWriter {
        todo!()
    }
}

pub trait BuilderParent: Sized {
    fn process(&mut self, value: &impl StrictEncode) -> LibRef;
    fn complete(self, lib: LibName, name: Option<TypeName>, ty: Ty<LibRef>) -> Self;
}
impl BuilderParent for TypeBuilder {
    fn process(&mut self, value: &impl StrictEncode) -> LibRef { todo!() }
    fn complete(self, lib: LibName, name: Option<TypeName>, ty: Ty<LibRef>) -> Self { todo!() }
}
impl BuilderParent for UnionBuilder {
    fn process(&mut self, value: &impl StrictEncode) -> LibRef { self.parent.process(value) }
    fn complete(self, lib: LibName, name: Option<TypeName>, ty: Ty<LibRef>) -> Self { todo!() }
}

pub struct EnumBuilder<P: BuilderParent> {
    lib: LibName,
    name: Option<TypeName>,
    variants: BTreeSet<Field>,
    ord: u8,
    parent: P,
}

impl<P: BuilderParent> WriteEnum<P> for EnumBuilder<P> {
    fn write_variant(mut self, name: impl ToIdent, value: u8) -> io::Result<Self> {
        let field = Field::named(name.to_ident(), value);
        assert!(self.variants.insert(field), "repeated enum variant name or value");
        self.ord = value + 1;
        Ok(self)
    }

    fn complete(self) -> P {
        assert!(!self.variants.is_empty(), "building enum with zero variants");
        let variants = Variants::try_from(self.variants).expect("too many enum variants");
        self.parent.complete(self.lib, self.name, Ty::Enum(variants))
    }
}

pub struct UnionBuilder {
    lib: LibName,
    name: Option<TypeName>,
    variants: BTreeSet<Field>,
    ord: u8,
    parent: TypeBuilder,
}

impl WriteUnion<TypeBuilder> for UnionBuilder {
    type TupleWriter = StructBuilder<Self, false>;
    type StructWriter = StructBuilder<Self, true>;

    fn write_unit(self, name: impl ToIdent) -> io::Result<Self> { todo!() }

    fn write_tuple(self, name: impl ToIdent) -> Self::TupleWriter { todo!() }

    fn write_struct(self, name: impl ToIdent) -> Self::StructWriter { todo!() }

    fn complete(self) -> TypeBuilder {
        assert!(!self.variants.is_empty(), "building union with zero variants");
        let variants = Variants::try_from(self.variants).expect("too many union variants");
        self.parent.complete(self.lib, self.name, Ty::Enum(variants))
    }
}

pub struct StructBuilder<P: BuilderParent, const NAMED: bool> {
    lib: LibName,
    name: Option<TypeName>,
    fields: BTreeMap<Field, LibRef>,
    ord: u8,
    parent: P,
}

impl<P: BuilderParent, const NAMED: bool> StructBuilder<P, NAMED> {
    pub fn with(lib: LibName, name: Option<TypeName>, parent: P) -> Self {
        StructBuilder {
            lib,
            name,
            fields: empty!(),
            ord: 0,
            parent,
        }
    }

    fn _write_field(
        mut self,
        name: Option<impl ToIdent>,
        ord: u8,
        value: &impl StrictEncode,
    ) -> io::Result<Self> {
        let ty = self.parent.process(value);
        let field = match name {
            Some(name) => Field::named(name.to_ident(), ord),
            None => Field::unnamed(ord),
        };
        self.fields.insert(field, ty).expect("repeated field name");
        self.ord = ord + 1;
        Ok(self)
    }

    fn _complete(self) -> P {
        assert!(!self.fields.is_empty(), "building structure with zero fields");
        let fields = Fields::try_from(self.fields).expect("too many fields");
        self.parent.complete(self.lib, self.name, Ty::Struct(fields))
    }
}

impl<P: BuilderParent> WriteStruct<P> for StructBuilder<P, true> {
    fn write_field(self, name: impl ToIdent, value: &impl StrictEncode) -> io::Result<Self> {
        let ord = self.ord;
        self.write_field_ord(name, ord, value)
    }

    fn write_field_ord(
        self,
        name: impl ToIdent,
        ord: u8,
        value: &impl StrictEncode,
    ) -> io::Result<Self> {
        self._write_field(Some(name), ord, value)
    }

    fn complete(self) -> P { self._complete() }
}

impl<P: BuilderParent> WriteTuple<P> for StructBuilder<P, false> {
    fn write_field(self, value: &impl StrictEncode) -> io::Result<Self> {
        let ord = self.ord;
        self.write_field_ord(ord, value)
    }

    fn write_field_ord(self, ord: u8, value: &impl StrictEncode) -> io::Result<Self> {
        self._write_field(None::<String>, ord, value)
    }

    fn complete(self) -> P { self._complete() }
}
