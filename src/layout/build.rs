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

use encoding::{
    DefineEnum, DefineStruct, DefineTuple, DefineUnion, FieldName, Primitive, Sizing, StrictDumb,
    StrictEncode, StrictEnum, StrictStruct, StrictTuple, StrictUnion, TypedParent, TypedWrite,
    WriteEnum, WriteStruct, WriteTuple, WriteUnion,
};

pub struct LayoutBuilder {
    pieces: Vec<LayoutPiece>,
}

impl TypedWrite for LayoutBuilder {
    type TupleWriter = Self;
    type StructWriter = Self;
    type UnionDefiner = Self;

    fn write_union<T: StrictUnion>(
        self,
        inner: impl FnOnce(Self::UnionDefiner) -> std::io::Result<Self>,
    ) -> std::io::Result<Self> {
        todo!()
    }

    fn write_enum<T: StrictEnum>(self, value: T) -> std::io::Result<Self>
    where u8: From<T> {
        todo!()
    }

    fn write_tuple<T: StrictTuple>(
        self,
        inner: impl FnOnce(Self::TupleWriter) -> std::io::Result<Self>,
    ) -> std::io::Result<Self> {
        todo!()
    }

    fn write_struct<T: StrictStruct>(
        self,
        inner: impl FnOnce(Self::StructWriter) -> std::io::Result<Self>,
    ) -> std::io::Result<Self> {
        todo!()
    }

    unsafe fn register_primitive(self, prim: Primitive) -> Self { todo!() }

    unsafe fn register_array(self, ty: &impl StrictEncode, len: u16) -> Self { todo!() }

    unsafe fn register_unicode(self, sizing: Sizing) -> Self { todo!() }

    unsafe fn register_ascii(self, sizing: Sizing) -> Self { todo!() }

    unsafe fn register_list(self, ty: &impl StrictEncode, sizing: Sizing) -> Self { todo!() }

    unsafe fn register_set(self, ty: &impl StrictEncode, sizing: Sizing) -> Self { todo!() }

    unsafe fn register_map(
        self,
        ket: &impl StrictEncode,
        ty: &impl StrictEncode,
        sizing: Sizing,
    ) -> Self {
        todo!()
    }

    unsafe fn _write_raw<const MAX_LEN: usize>(
        self,
        bytes: impl AsRef<[u8]>,
    ) -> std::io::Result<Self> {
        todo!()
    }
}

impl TypedParent for LayoutBuilder {}

impl DefineUnion for LayoutBuilder {
    type Parent = Self;
    type TupleDefiner = Self;
    type StructDefiner = Self;
    type UnionWriter = Self;

    fn define_unit(self, name: FieldName) -> Self { todo!() }

    fn define_tuple(self, name: FieldName, inner: impl FnOnce(Self::TupleDefiner) -> Self) -> Self {
        todo!()
    }

    fn define_struct(
        self,
        name: FieldName,
        inner: impl FnOnce(Self::StructDefiner) -> Self,
    ) -> Self {
        todo!()
    }

    fn complete(self) -> Self::UnionWriter { todo!() }
}

impl DefineEnum for LayoutBuilder {
    type Parent = Self;
    type EnumWriter = Self;

    fn define_variant(self, name: FieldName) -> Self { todo!() }

    fn complete(self) -> Self::EnumWriter { todo!() }
}

impl DefineTuple for LayoutBuilder {
    type Parent = Self;

    fn define_field<T: StrictEncode + StrictDumb>(self) -> Self { todo!() }

    fn complete(self) -> Self::Parent { todo!() }
}

impl DefineStruct for LayoutBuilder {
    type Parent = Self;

    fn define_field<T: StrictEncode + StrictDumb>(self, name: FieldName) -> Self { todo!() }

    fn complete(self) -> Self::Parent { todo!() }
}

impl WriteEnum for LayoutBuilder {
    type Parent = Self;

    fn write_variant(self, name: FieldName) -> std::io::Result<Self> { todo!() }

    fn complete(self) -> Self::Parent { todo!() }
}

impl WriteUnion for LayoutBuilder {
    type Parent = Self;
    type TupleWriter = Self;
    type StructWriter = Self;

    fn write_unit(self, name: FieldName) -> std::io::Result<Self> { todo!() }

    fn write_tuple(
        self,
        name: FieldName,
        inner: impl FnOnce(Self::TupleWriter) -> std::io::Result<Self>,
    ) -> std::io::Result<Self> {
        todo!()
    }

    fn write_struct(
        self,
        name: FieldName,
        inner: impl FnOnce(Self::StructWriter) -> std::io::Result<Self>,
    ) -> std::io::Result<Self> {
        todo!()
    }

    fn complete(self) -> Self::Parent { todo!() }
}

impl WriteTuple for LayoutBuilder {
    type Parent = Self;

    fn write_field(self, value: &impl StrictEncode) -> std::io::Result<Self> { todo!() }

    fn complete(self) -> Self::Parent { todo!() }
}

impl WriteStruct for LayoutBuilder {
    type Parent = Self;

    fn write_field(self, name: FieldName, value: &impl StrictEncode) -> std::io::Result<Self> {
        todo!()
    }

    fn complete(self) -> Self::Parent { todo!() }
}
