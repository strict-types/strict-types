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
use std::io::Sink;

use amplify::confinement;
use amplify::confinement::{Confined, SmallOrdMap};

use crate::ast::{Fields, Variants};
use crate::encoding::{
    DefineEnum, DefineStruct, DefineTuple, DefineUnion, StrictEncode, StrictParent, StrictWriter,
    StructWriter, ToIdent, ToMaybeIdent, TypedParent, TypedWrite, UnionWriter, WriteEnum,
    WriteStruct, WriteTuple, WriteUnion,
};
use crate::typelib::type_lib::StrictType;
use crate::typelib::TypeIndex;
use crate::{LibName, LibRef, Ty, TypeLib, TypeName};

#[derive(Default)]
pub struct LibBuilder {
    index: TypeIndex,
    types: SmallOrdMap<TypeName, StrictType>,
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

impl TypedWrite for LibBuilder {
    type TupleWriter = StructBuilder<Self>;
    type StructWriter = StructBuilder<Self>;
    type UnionDefiner = UnionBuilder;
    type EnumDefiner = UnionBuilder;

    fn define_union(self, name: Option<impl ToIdent>) -> Self::UnionDefiner {
        UnionBuilder::with(name.to_maybe_ident(), self)
    }

    fn define_enum(self, name: Option<impl ToIdent>) -> Self::EnumDefiner {
        UnionBuilder::with(name.to_maybe_ident(), self)
    }

    fn write_tuple(self, name: Option<impl ToIdent>) -> Self::TupleWriter {
        StructBuilder::with(name.to_maybe_ident(), self)
    }

    fn write_struct(self, name: Option<impl ToIdent>) -> Self::StructWriter {
        StructBuilder::with(name.to_maybe_ident(), self)
    }

    unsafe fn _write_raw<const LEN: usize>(self, _bytes: impl AsRef<[u8]>) -> io::Result<Self> {
        // Nothing to do here
        Ok(self)
    }
}

pub struct StructBuilder<P: BuilderParent> {
    name: Option<TypeName>,
    writer: StructWriter<Sink, P>,
    types: BTreeMap<u8, LibRef>,
}

impl<P: BuilderParent> StructBuilder<P> {
    pub fn with(name: Option<TypeName>, parent: P) -> Self {
        StructBuilder {
            name: name.clone(),
            writer: StructWriter::with(name, parent),
            types: empty!(),
        }
    }

    fn _define_field<T: StrictEncode>(mut self, ord: u8) -> Self {
        let ty = self.writer.as_parent_mut().process(&T::strict_encode_dumb());
        self.types.insert(ord, ty).expect("checked by self.writer");
        self
    }

    fn _write_field(mut self, ord: u8, value: &impl StrictEncode) -> io::Result<Self> {
        let expect_ty = self.types.get(&ord).expect("type guarantees");
        let ty = self.writer.as_parent_mut().process(value);
        assert_eq!(
            expect_ty,
            &ty,
            "field #{} in {} has a wrong type {} (expected {})",
            ord,
            self.writer.name(),
            ty,
            expect_ty
        );
        Ok(self)
    }

    fn _complete_definition(self) -> P { DefineStruct::complete(self.writer) }

    fn _complete_write(self) -> P {
        let ty = self.name.map(|name| {
            let fields = self
                .writer
                .fields()
                .iter()
                .map(|field| {
                    let lib_ref = self.types.get(&field.ord).expect("type guarantees");
                    (field.clone(), lib_ref.clone())
                })
                .collect::<BTreeMap<_, _>>();
            let fields = Fields::try_from(fields)
                .expect(&format!("structure {} has invalid number of fields", name));
            StrictType::with(name, Ty::Struct(fields))
        });
        let parent = WriteStruct::complete(self.writer);
        match ty {
            Some(ty) => parent.complete(ty),
            None => parent,
        }
    }
}

impl<P: BuilderParent> DefineStruct for StructBuilder<P> {
    type Parent = P;

    fn define_field<T: StrictEncode>(mut self, name: impl ToIdent) -> Self {
        let ord = self.writer.field_ord(&name.to_ident()).expect("StructWriter is broken");
        self.writer = DefineStruct::define_field::<T>(self.writer, name.to_ident());
        self._define_field::<T>(ord)
    }

    fn define_field_ord<T: StrictEncode>(mut self, name: impl ToIdent, ord: u8) -> Self {
        self.writer = DefineStruct::define_field_ord::<T>(self.writer, name.to_ident(), ord);
        self._define_field::<T>(ord)
    }

    fn complete(self) -> P { self._complete_definition() }
}

impl<P: BuilderParent> WriteStruct for StructBuilder<P> {
    type Parent = P;

    fn write_field(mut self, name: impl ToIdent, value: &impl StrictEncode) -> io::Result<Self> {
        let ord = self.writer.next_ord();
        self.writer = WriteStruct::write_field(self.writer, name.to_ident(), value)?;
        self._write_field(ord, value)
    }

    fn write_field_ord(
        mut self,
        name: impl ToIdent,
        ord: u8,
        value: &impl StrictEncode,
    ) -> io::Result<Self> {
        self.writer = WriteStruct::write_field_ord(self.writer, name.to_ident(), ord, value)?;
        self._write_field(ord, value)
    }

    fn complete(self) -> P { self._complete_write() }
}

impl<P: BuilderParent> DefineTuple for StructBuilder<P> {
    type Parent = P;

    fn define_field<T: StrictEncode>(mut self) -> Self {
        let ord = self.writer.next_ord();
        self.writer = DefineTuple::define_field::<T>(self.writer);
        self._define_field::<T>(ord)
    }

    fn define_field_ord<T: StrictEncode>(mut self, ord: u8) -> Self {
        self.writer = DefineTuple::define_field_ord::<T>(self.writer, ord);
        self._define_field::<T>(ord)
    }

    fn complete(self) -> P { self._complete_definition() }
}

impl<P: BuilderParent> WriteTuple for StructBuilder<P> {
    type Parent = P;

    fn write_field(mut self, value: &impl StrictEncode) -> io::Result<Self> {
        let ord = self.writer.next_ord();
        self.writer = WriteTuple::write_field(self.writer, value)?;
        self._write_field(ord, value)
    }

    fn write_field_ord(mut self, ord: u8, value: &impl StrictEncode) -> io::Result<Self> {
        self.writer = WriteTuple::write_field_ord(self.writer, ord, value)?;
        self._write_field(ord, value)
    }

    fn complete(self) -> P { self._complete_write() }
}

pub struct UnionBuilder {
    name: Option<TypeName>,
    parent: LibBuilder,
    writer: UnionWriter<Sink>,
}

impl UnionBuilder {
    pub fn with(name: Option<TypeName>, parent: LibBuilder) -> Self {
        UnionBuilder {
            name: name.clone(),
            parent,
            writer: UnionWriter::with(name, StrictWriter::sink()),
        }
    }

    fn _complete_definition(mut self) -> UnionBuilder {
        self.writer = DefineUnion::complete(self.writer);
        self
    }

    fn _complete_write(self, ty: Option<StrictType>) -> LibBuilder {
        let _ = WriteUnion::complete(self.writer);
        match ty {
            Some(ty) => self.parent.complete(ty),
            None => self.parent,
        }
    }
}

impl DefineEnum for UnionBuilder {
    type Parent = LibBuilder;
    type EnumWriter = Self;

    fn define_variant(mut self, name: impl ToIdent, value: u8) -> Self {
        self.writer = DefineEnum::define_variant(self.writer, name, value);
        self
    }

    fn complete(self) -> Self::EnumWriter { self._complete_definition() }
}

impl WriteEnum for UnionBuilder {
    type Parent = LibBuilder;

    fn write_variant(mut self, name: impl ToIdent) -> io::Result<Self> {
        self.writer = WriteEnum::write_variant(self.writer, name)?;
        Ok(self)
    }

    fn complete(self) -> LibBuilder {
        let ty = self.name.as_ref().map(|name| {
            let fields = self.writer.variants().keys().cloned().collect::<BTreeSet<_>>();
            let fields = Variants::try_from(fields)
                .expect(&format!("enum {} has invalid number of variants", name));
            StrictType::with(name.clone(), Ty::Enum(fields))
        });
        self._complete_write(ty)
    }
}

impl DefineUnion for UnionBuilder {
    type Parent = LibBuilder;
    type TupleDefiner = StructBuilder<Self>;
    type StructDefiner = StructBuilder<Self>;
    type UnionWriter = Self;

    fn define_unit(mut self, name: impl ToIdent) -> Self {
        self.writer = DefineUnion::define_unit(self.writer, name);
        self
    }

    fn define_tuple(mut self, name: impl ToIdent) -> Self::TupleDefiner {
        let sink = DefineUnion::define_tuple(self.writer, name);
        self.writer = sink.into_parent();
        StructBuilder::with(None, self)
    }

    fn define_struct(mut self, name: impl ToIdent) -> Self::StructDefiner {
        let sink = DefineUnion::define_struct(self.writer, name);
        self.writer = sink.into_parent();
        StructBuilder::with(None, self)
    }

    fn complete(self) -> Self::UnionWriter { self._complete_definition() }
}

impl WriteUnion for UnionBuilder {
    type Parent = LibBuilder;
    type TupleWriter = StructBuilder<Self>;
    type StructWriter = StructBuilder<Self>;

    fn write_unit(mut self, name: impl ToIdent) -> io::Result<Self> {
        self.writer = WriteUnion::write_unit(self.writer, name)?;
        Ok(self)
    }

    fn write_tuple(mut self, name: impl ToIdent) -> io::Result<Self::TupleWriter> {
        let sink = WriteUnion::write_tuple(self.writer, name)?;
        self.writer = sink.into_parent();
        Ok(StructBuilder::with(None, self))
    }

    fn write_struct(mut self, name: impl ToIdent) -> io::Result<Self::StructWriter> {
        let sink = WriteUnion::write_struct(self.writer, name)?;
        self.writer = sink.into_parent();
        Ok(StructBuilder::with(None, self))
    }

    fn complete(self) -> LibBuilder {
        let ty = self.name.as_ref().map(|name| {
            let fields = self
                .writer
                .variants()
                .keys()
                .map(|field| {
                    let lib_ref = self.types.get(&field.ord).expect("type guarantees");
                    (field.clone(), lib_ref.clone())
                })
                .collect::<BTreeMap<_, _>>();
            let fields = Fields::try_from(fields)
                .expect(&format!("union {} has invalid number of variants", name));
            StrictType::with(name.clone(), Ty::Union(fields))
        });
        self._complete_write(ty)
    }
}

pub trait BuilderParent: StrictParent<Sink> {
    fn process(&mut self, value: &impl StrictEncode) -> LibRef;
    fn complete(self, ty: StrictType) -> Self;
}

impl TypedParent for LibBuilder {}
impl StrictParent<Sink> for LibBuilder {
    type Remnant = LibBuilder;
    fn from_split(_: StrictWriter<Sink>, remnant: Self::Remnant) -> Self { remnant }
    fn split_typed_write(self) -> (StrictWriter<Sink>, Self::Remnant) {
        (StrictWriter::sink(), self)
    }
}
impl BuilderParent for LibBuilder {
    fn process(&mut self, value: &impl StrictEncode) -> LibRef { todo!() }
    fn complete(mut self, ty: StrictType) -> Self {
        let id = ty.id();
        self.types
            .insert(ty.name.clone(), ty)
            .expect("too many types")
            .expect("repeated type name");
        self.index.insert(id, ty.name.clone()).expect(&format!(
            "type with the same id as {} is already present within the library",
            id
        ));
        self
    }
}

impl TypedParent for UnionBuilder {}
impl StrictParent<Sink> for UnionBuilder {
    type Remnant = UnionBuilder;
    fn from_split(_: StrictWriter<Sink>, remnant: Self::Remnant) -> Self { remnant }
    fn split_typed_write(self) -> (StrictWriter<Sink>, Self::Remnant) {
        (StrictWriter::sink(), self)
    }
}
impl BuilderParent for UnionBuilder {
    fn process(&mut self, value: &impl StrictEncode) -> LibRef { self.parent.process(value) }
    fn complete(mut self, ty: StrictType) -> Self {
        self.parent = self.parent.complete(ty);
        self
    }
}
