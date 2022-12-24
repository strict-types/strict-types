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

use amplify::confinement::SmallOrdMap;

use super::compile::{CompileRef, CompileType};
use crate::ast::{Fields, Variants};
use crate::encoding::{
    DefineEnum, DefineStruct, DefineTuple, DefineUnion, SplitParent, StrictEncode, StrictParent,
    StrictWriter, StructWriter, TypedParent, TypedWrite, UnionWriter, WriteEnum, WriteStruct,
    WriteTuple, WriteUnion,
};
use crate::primitive::Primitive;
use crate::util::Sizing;
use crate::{FieldName, LibName, Ty, TypeName};

pub trait BuilderParent: StrictParent<Sink> {
    /// Converts strict-encodable value into a type information. Must be propagated back to the
    /// lib builder which does the TypedWrite implementation to call strict encode on the type
    fn compile_type(self, value: &impl StrictEncode) -> (Self, CompileRef);
    /// Notifies lib builder about complete type built, even for unnamed inline types, such that it
    /// can register last compiled type for the `compile_type` procedure.
    fn report_compiled(self, name: Option<TypeName>, ty: Ty<CompileRef>) -> Self;
}

#[derive(Default)]
pub struct LibBuilder {
    types: SmallOrdMap<TypeName, CompileType>,
    last_compiled: Option<CompileRef>,
}

impl LibBuilder {
    pub fn new() -> LibBuilder {
        LibBuilder {
            types: default!(),
            last_compiled: None,
        }
    }

    pub fn process(self, ty: &impl StrictEncode) -> io::Result<Self> {
        unsafe { ty.strict_encode(self) }
    }

    pub fn into_types(self) -> SmallOrdMap<TypeName, CompileType> { self.types }
}

impl TypedWrite for LibBuilder {
    type TupleWriter = StructBuilder<Self>;
    type StructWriter = StructBuilder<Self>;
    type UnionDefiner = UnionBuilder;
    type EnumDefiner = UnionBuilder;

    fn define_union(self, lib: LibName, name: Option<TypeName>) -> Self::UnionDefiner {
        UnionBuilder::with(lib, name, self)
    }

    fn define_enum(self, lib: LibName, name: Option<TypeName>) -> Self::EnumDefiner {
        UnionBuilder::with(lib, name, self)
    }

    fn write_tuple(self, lib: LibName, name: Option<TypeName>) -> Self::TupleWriter {
        StructBuilder::with(lib, name, self)
    }

    fn write_struct(self, lib: LibName, name: Option<TypeName>) -> Self::StructWriter {
        StructBuilder::with(lib, name, self)
    }

    fn register_primitive(mut self, prim: Primitive) -> Self {
        self.last_compiled = Some(Ty::Primitive(prim).into());
        self
    }

    fn register_array(mut self, ty: &impl StrictEncode, len: u16) -> Self {
        self = unsafe { ty.strict_encode(self).expect("in-memory encoding") };
        let ty = self.last_compiled.expect("can't compile type");
        self.last_compiled = Some(Ty::Array(ty, len).into());
        self
    }

    fn register_unicode_char(mut self) -> Self {
        self.last_compiled = Some(Ty::UnicodeChar.into());
        self
    }

    fn register_unicode_string(mut self, sizing: Sizing) -> Self {
        self.last_compiled = Some(Ty::List(Ty::UnicodeChar.into(), sizing).into());
        self
    }

    fn register_ascii_string(mut self, sizing: Sizing) -> Self {
        self.last_compiled = Some(Ty::List(Ty::ascii_char().into(), sizing).into());
        self
    }

    fn register_list(mut self, ty: &impl StrictEncode, sizing: Sizing) -> Self {
        self = unsafe { ty.strict_encode(self).expect("in-memory encoding") };
        let ty = self.last_compiled.expect("can't compile type");
        self.last_compiled = Some(Ty::List(ty, sizing).into());
        self
    }

    fn register_set(mut self, ty: &impl StrictEncode, sizing: Sizing) -> Self {
        self = unsafe { ty.strict_encode(self).expect("in-memory encoding") };
        let ty = self.last_compiled.expect("can't compile type");
        self.last_compiled = Some(Ty::Set(ty, sizing).into());
        self
    }

    fn register_map(
        mut self,
        key: &impl StrictEncode,
        ty: &impl StrictEncode,
        sizing: Sizing,
    ) -> Self {
        self = unsafe { ty.strict_encode(self).expect("in-memory encoding") };
        let ty = self.last_compiled.clone().expect("can't compile type");
        self = unsafe { key.strict_encode(self).expect("in-memory encoding") };
        let key_ty = match self.last_compiled.clone().expect("can't compile key type") {
            CompileRef::Inline(ty) => {
                ty.try_to_key().expect(&format!("not supported map key type {}", ty))
            }
            me @ CompileRef::Named(_) | me @ CompileRef::Extern(_, _) => {
                panic!("not supported map key type {}", me)
            }
        };
        self.last_compiled = Some(Ty::Map(key_ty, ty, sizing).into());
        self
    }

    unsafe fn _write_raw<const LEN: usize>(self, _bytes: impl AsRef<[u8]>) -> io::Result<Self> {
        // Nothing to do here
        Ok(self)
    }
}

impl TypedParent for LibBuilder {}
impl StrictParent<Sink> for LibBuilder {
    type Remnant = LibBuilder;
    fn from_write_split(_: StrictWriter<Sink>, remnant: Self::Remnant) -> Self { remnant }
    fn into_write_split(self) -> (StrictWriter<Sink>, Self::Remnant) {
        (StrictWriter::sink(), self)
    }
}
impl BuilderParent for LibBuilder {
    fn compile_type(mut self, value: &impl StrictEncode) -> (Self, CompileRef) {
        self = unsafe { value.strict_encode(self).expect("too many types in the library") };
        let r = self.last_compiled.clone().expect("no type found after strict encoding procedure");
        (self, r)
    }

    fn report_compiled(mut self, name: Option<TypeName>, ty: Ty<CompileRef>) -> Self {
        let r = match name {
            Some(name) => {
                let new_ty = CompileType::new(name.clone(), ty);
                let old_ty = self.types.insert(name.clone(), new_ty).expect("too many types");
                if let Some(old_ty) = old_ty {
                    let new_ty = self.types.get(&name).expect("just inserted");
                    assert_eq!(
                        &old_ty, new_ty,
                        "repeated type name {} for two different types {} and {}",
                        name, old_ty, new_ty
                    );
                }
                CompileRef::Named(name)
            }
            None => CompileRef::Inline(Box::new(ty)),
        };
        self.last_compiled = Some(r);
        self
    }
}

pub struct StructBuilder<P: BuilderParent> {
    lib: LibName,
    name: Option<TypeName>,
    writer: StructWriter<Sink, P>,
    fields: BTreeMap<u8, CompileRef>,
}

impl<P: BuilderParent> StructBuilder<P> {
    pub fn with(lib: LibName, name: Option<TypeName>, parent: P) -> Self {
        StructBuilder {
            lib,
            name: name.clone(),
            writer: StructWriter::with(name, parent),
            fields: empty!(),
        }
    }

    pub fn name(&self) -> &str { self.name.as_ref().map(|n| n.as_str()).unwrap_or("<unnamed>") }

    fn _define_field<T: StrictEncode>(mut self, ord: u8) -> Self {
        let (parent, remnant) = self.writer.into_parent_split();
        let (parent, ty) = parent.compile_type(&T::strict_encode_dumb());
        self.writer = StructWriter::from_parent_split(parent, remnant);
        let _ = self.fields.insert(ord, ty); // type repetition is checked by self.parent
        self
    }

    fn _write_field(mut self, ord: u8, value: &impl StrictEncode) -> io::Result<Self> {
        let (parent, remnant) = self.writer.into_parent_split();
        let (parent, ty) = parent.compile_type(value);
        self.writer = StructWriter::from_parent_split(parent, remnant);
        if let Some(expect_ty) = self.fields.insert(ord, ty.clone()) {
            assert_eq!(
                expect_ty,
                ty,
                "field #{} in {} has a wrong type {} (expected {})",
                ord,
                self.writer.name(),
                ty,
                expect_ty
            );
        }
        Ok(self)
    }

    fn _build_struct(&self) -> Ty<CompileRef> {
        let fields = self
            .writer
            .fields()
            .iter()
            .map(|field| {
                let lib_ref = self.fields.get(&field.ord).expect("type guarantees");
                (field.clone(), lib_ref.clone())
            })
            .collect::<BTreeMap<_, _>>();
        let fields = Fields::try_from(fields)
            .expect(&format!("structure {} has invalid number of fields", self.name()));
        Ty::Struct(fields)
    }

    fn _complete_definition(self) -> P {
        let ty = self._build_struct();
        DefineStruct::complete(self.writer).report_compiled(self.name.clone(), ty)
    }

    fn _complete_write(self) -> P {
        let ty = self._build_struct();
        WriteStruct::complete(self.writer).report_compiled(self.name.clone(), ty)
    }
}

impl<P: BuilderParent> DefineStruct for StructBuilder<P> {
    type Parent = P;

    fn define_field<T: StrictEncode>(mut self, name: TypeName) -> Self {
        let ord = self.writer.field_ord(&name).expect("StructWriter is broken");
        self.writer = DefineStruct::define_field::<T>(self.writer, name);
        self._define_field::<T>(ord)
    }

    fn define_field_ord<T: StrictEncode>(mut self, name: TypeName, ord: u8) -> Self {
        self.writer = DefineStruct::define_field_ord::<T>(self.writer, name, ord);
        self._define_field::<T>(ord)
    }

    fn complete(self) -> P { self._complete_definition() }
}

impl<P: BuilderParent> WriteStruct for StructBuilder<P> {
    type Parent = P;

    fn write_field(mut self, name: TypeName, value: &impl StrictEncode) -> io::Result<Self> {
        let ord = self.writer.next_ord();
        self.writer = WriteStruct::write_field(self.writer, name, value)?;
        self._write_field(ord, value)
    }

    fn write_field_ord(
        mut self,
        name: TypeName,
        ord: u8,
        value: &impl StrictEncode,
    ) -> io::Result<Self> {
        self.writer = WriteStruct::write_field_ord(self.writer, name, ord, value)?;
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
    lib: LibName,
    name: Option<TypeName>,
    variants: BTreeMap<u8, CompileRef>,
    parent: LibBuilder,
    writer: UnionWriter<Sink>,
    current_ord: u8,
}

impl UnionBuilder {
    pub fn with(lib: LibName, name: Option<TypeName>, parent: LibBuilder) -> Self {
        UnionBuilder {
            lib,
            name: name.clone(),
            variants: empty!(),
            parent,
            writer: UnionWriter::with(name, StrictWriter::sink()),
            current_ord: 0,
        }
    }

    pub fn name(&self) -> &str { self.name.as_ref().map(|n| n.as_str()).unwrap_or("<unnamed>") }

    fn _define_field(&mut self, ord: Option<u8>) {
        self.current_ord = ord.unwrap_or_else(|| self.writer.next_ord());
        let ty = self.parent.last_compiled.clone().unwrap_or_else(CompileRef::unit);
        self.variants.insert(self.current_ord, ty);
    }

    fn _write_field(&mut self, name: FieldName) {
        self.current_ord = self.writer.ord_by_name(&name).expect(&format!(
            "writing field {} of {} without declaration",
            name,
            self.name()
        ));
        let ty = self.parent.last_compiled.clone().expect("no compiled type");
        self.variants.insert(self.current_ord, ty);
    }

    fn _build_union(&self) -> Ty<CompileRef> {
        let fields = self
            .writer
            .variants()
            .keys()
            .map(|field| {
                let lib_ref = self.variants.get(&field.ord).expect("type guarantees");
                (field.clone(), lib_ref.clone())
            })
            .collect::<BTreeMap<_, _>>();
        let fields = Fields::try_from(fields)
            .expect(&format!("union {} has invalid number of variants", self.name()));
        Ty::Union(fields)
    }

    fn _build_enum(&self) -> Ty<CompileRef> {
        let fields = self.writer.variants().keys().cloned().collect::<BTreeSet<_>>();
        let fields = Variants::try_from(fields)
            .expect(&format!("enum {} has invalid number of variants", self.name()));
        Ty::Enum(fields)
    }

    fn _complete_definition(mut self, ty: Ty<CompileRef>) -> UnionBuilder {
        self.writer = DefineUnion::complete(self.writer);
        self.parent = self.parent.report_compiled(self.name.clone(), ty);
        self
    }

    fn _complete_write(self, ty: Ty<CompileRef>) -> LibBuilder {
        let _ = WriteUnion::complete(self.writer);
        self.parent.report_compiled(self.name, ty)
    }
}

impl TypedParent for UnionBuilder {}
impl StrictParent<Sink> for UnionBuilder {
    type Remnant = UnionBuilder;
    fn from_write_split(_: StrictWriter<Sink>, remnant: Self::Remnant) -> Self { remnant }
    fn into_write_split(self) -> (StrictWriter<Sink>, Self::Remnant) {
        (StrictWriter::sink(), self)
    }
}
impl BuilderParent for UnionBuilder {
    fn compile_type(mut self, value: &impl StrictEncode) -> (Self, CompileRef) {
        let (parent, r) = self.parent.compile_type(value);
        self.parent = parent;
        (self, r)
    }
    fn report_compiled(mut self, name: Option<TypeName>, ty: Ty<CompileRef>) -> Self {
        self.variants.insert(self.current_ord, CompileRef::Inline(Box::new(ty.clone())));
        self.parent = self.parent.report_compiled(name, ty);
        self
    }
}

impl DefineEnum for UnionBuilder {
    type Parent = LibBuilder;
    type EnumWriter = Self;

    fn define_variant(mut self, name: TypeName, value: u8) -> Self {
        self.parent = self.parent.report_compiled(None, Ty::U8);
        self._define_field(Some(value));
        self.writer = DefineEnum::define_variant(self.writer, name, value);
        self
    }

    fn complete(self) -> Self::EnumWriter {
        let ty = self._build_enum();
        self._complete_definition(ty)
    }
}

impl WriteEnum for UnionBuilder {
    type Parent = LibBuilder;

    fn write_variant(mut self, name: TypeName) -> io::Result<Self> {
        let name = name;
        self.parent = self.parent.report_compiled(None, Ty::U8);
        self._write_field(name.clone());
        self.writer = WriteEnum::write_variant(self.writer, name)?;
        Ok(self)
    }

    fn complete(self) -> LibBuilder {
        let ty = self._build_enum();
        self._complete_write(ty)
    }
}

impl DefineUnion for UnionBuilder {
    type Parent = LibBuilder;
    type TupleDefiner = StructBuilder<Self>;
    type StructDefiner = StructBuilder<Self>;
    type UnionWriter = Self;

    fn define_unit(mut self, name: TypeName) -> Self {
        self.parent = self.parent.report_compiled(None, Ty::UNIT);
        self._define_field(None);
        self.writer = DefineUnion::define_unit(self.writer, name);
        self
    }

    fn define_tuple(mut self, name: TypeName) -> Self::TupleDefiner {
        self._define_field(None);
        let sink = DefineUnion::define_tuple(self.writer, name);
        self.writer = sink.into_parent();
        StructBuilder::with(self.lib.clone(), None, self)
    }

    fn define_struct(mut self, name: TypeName) -> Self::StructDefiner {
        self._define_field(None);
        let sink = DefineUnion::define_struct(self.writer, name);
        self.writer = sink.into_parent();
        StructBuilder::with(self.lib.clone(), None, self)
    }

    fn complete(self) -> Self::UnionWriter {
        let ty = self._build_union();
        self._complete_definition(ty)
    }
}

impl WriteUnion for UnionBuilder {
    type Parent = LibBuilder;
    type TupleWriter = StructBuilder<Self>;
    type StructWriter = StructBuilder<Self>;

    fn write_unit(mut self, name: TypeName) -> io::Result<Self> {
        let name = name;
        self.parent = self.parent.report_compiled(None, Ty::UNIT);
        self._write_field(name.clone());
        self.writer = WriteUnion::write_unit(self.writer, name)?;
        Ok(self)
    }

    fn write_tuple(mut self, name: TypeName) -> io::Result<Self::TupleWriter> {
        let name = name;
        self._write_field(name.clone());
        let sink = WriteUnion::write_tuple(self.writer, name)?;
        self.writer = sink.into_parent();
        Ok(StructBuilder::with(self.lib.clone(), None, self))
    }

    fn write_struct(mut self, name: TypeName) -> io::Result<Self::StructWriter> {
        let name = name;
        self._write_field(name.clone());
        let sink = WriteUnion::write_struct(self.writer, name)?;
        self.writer = sink.into_parent();
        Ok(StructBuilder::with(self.lib.clone(), None, self))
    }

    fn complete(self) -> LibBuilder {
        let ty = self._build_union();
        self._complete_write(ty)
    }
}
