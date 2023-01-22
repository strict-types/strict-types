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

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::io::Sink;

use amplify::confinement::SmallOrdMap;
use strict_encoding::{
    DefineEnum, DefineStruct, DefineTuple, DefineUnion, FieldName, LibName, Primitive, Sizing,
    SplitParent, StrictDumb, StrictEncode, StrictEnum, StrictParent, StrictStruct, StrictSum,
    StrictTuple, StrictUnion, StrictWriter, StructWriter, TypeName, TypedParent, TypedWrite,
    UnionWriter, WriteEnum, WriteStruct, WriteTuple, WriteUnion,
};

use super::compile::{CompileRef, CompileType};
use crate::ast::{EnumVariants, Field, NamedFields, UnionVariants, UnnamedFields};
use crate::Ty;

pub trait BuilderParent: StrictParent<Sink> {
    /// Converts strict-encodable value into a type information. Must be propagated back to the
    /// lib builder which does the TypedWrite implementation to call strict encode on the type
    fn compile_type<T: StrictEncode>(self, value: &T) -> (Self, CompileRef);
    /// Notifies lib builder about complete type built, even for unnamed inline types, such that it
    /// can register last compiled type for the `compile_type` procedure.
    fn report_compiled(self, name: Option<TypeName>, ty: Ty<CompileRef>) -> Self;
}

#[derive(Debug)]
pub struct LibBuilder {
    name: LibName,
    types: SmallOrdMap<TypeName, CompileType>,
    last_compiled: Option<CompileRef>,
}

impl LibBuilder {
    pub fn new(name: LibName) -> LibBuilder {
        LibBuilder {
            name,
            types: default!(),
            last_compiled: None,
        }
    }

    pub fn name(&self) -> LibName { self.name.clone() }

    pub fn process(self, ty: &impl StrictEncode) -> io::Result<Self> { ty.strict_encode(self) }

    pub fn into_types(self) -> SmallOrdMap<TypeName, CompileType> { self.types }
}

impl TypedWrite for LibBuilder {
    type TupleWriter = StructBuilder<Self>;
    type StructWriter = StructBuilder<Self>;
    type UnionDefiner = UnionBuilder;

    fn write_union<T: StrictUnion>(
        self,
        inner: impl FnOnce(Self::UnionDefiner) -> io::Result<Self>,
    ) -> io::Result<Self> {
        let builder = UnionBuilder::with::<T>(self);
        inner(builder)
    }

    fn write_enum<T: StrictEnum>(self, value: T) -> io::Result<Self>
    where u8: From<T> {
        let mut writer = UnionBuilder::with::<T>(self);
        for (ord, name) in T::ALL_VARIANTS {
            writer = writer.define_variant(fname!(*name), *ord);
        }
        writer = DefineEnum::complete(writer);
        writer = writer.write_variant(fname!(value.variant_name()))?;
        Ok(WriteEnum::complete(writer))
    }

    fn write_tuple<T: StrictTuple>(
        self,
        inner: impl FnOnce(Self::TupleWriter) -> io::Result<Self>,
    ) -> io::Result<Self> {
        let lib = self.name.clone();
        let writer = StructWriter::tuple::<T>(self);
        let builder = StructBuilder::with(lib, T::strict_name(), writer, false);
        inner(builder)
    }

    fn write_struct<T: StrictStruct>(
        self,
        inner: impl FnOnce(Self::StructWriter) -> io::Result<Self>,
    ) -> io::Result<Self> {
        let lib = self.name.clone();
        let writer = StructWriter::structure::<T>(self);
        let builder = StructBuilder::with(lib, T::strict_name(), writer, false);
        inner(builder)
    }

    unsafe fn register_primitive(mut self, prim: Primitive) -> Self {
        self.last_compiled = Some(Ty::Primitive(prim).into());
        self
    }

    unsafe fn register_array(mut self, ty: &impl StrictEncode, len: u16) -> Self {
        self = ty.strict_encode(self).expect("in-memory encoding");
        let ty = self.last_compiled.expect("can't compile type");
        self.last_compiled = Some(Ty::Array(ty, len).into());
        self
    }

    unsafe fn register_unicode(mut self, sizing: Sizing) -> Self {
        self.last_compiled = Some(Ty::List(Ty::UnicodeChar.into(), sizing).into());
        self
    }

    unsafe fn register_ascii(mut self, sizing: Sizing) -> Self {
        self.last_compiled = Some(Ty::List(Ty::ascii_char().into(), sizing).into());
        self
    }

    unsafe fn register_list(mut self, ty: &impl StrictEncode, sizing: Sizing) -> Self {
        self = ty.strict_encode(self).expect("in-memory encoding");
        let ty = self.last_compiled.expect("can't compile type");
        self.last_compiled = Some(Ty::List(ty, sizing).into());
        self
    }

    unsafe fn register_set(mut self, ty: &impl StrictEncode, sizing: Sizing) -> Self {
        self = ty.strict_encode(self).expect("in-memory encoding");
        let ty = self.last_compiled.expect("can't compile type");
        self.last_compiled = Some(Ty::Set(ty, sizing).into());
        self
    }

    unsafe fn register_map(
        mut self,
        key: &impl StrictEncode,
        ty: &impl StrictEncode,
        sizing: Sizing,
    ) -> Self {
        self = ty.strict_encode(self).expect("in-memory encoding");
        let ty = self.last_compiled.clone().expect("can't compile type");
        self = key.strict_encode(self).expect("in-memory encoding");

        let mut r = self.last_compiled.as_ref().expect("can't compile key type");
        let key_ty = loop {
            let ty = match r {
                CompileRef::Embedded(ty) => ty.as_ref(),
                CompileRef::Named(name) => {
                    &self
                        .types
                        .get(name)
                        .unwrap_or_else(|| panic!("unknown map key type '{name}'"))
                        .ty
                }
                me @ CompileRef::Extern(_, _) => {
                    panic!("not supported map key type '{}'", me)
                }
            };
            if let Ty::Tuple(fields) = ty {
                if fields.len() == 1 {
                    r = &fields[0];
                } else {
                    panic!("not supported map key type '{ty}'")
                }
            } else {
                break ty
                    .try_to_key()
                    .unwrap_or_else(|_| panic!("not supported map key type '{ty}'"));
            };
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
    fn compile_type<T: StrictEncode>(mut self, value: &T) -> (Self, CompileRef) {
        if let Some(name) = T::strict_name() {
            if self.types.contains_key(&name) {
                return (self, CompileRef::Named(name));
            }
        }
        self = value.strict_encode(self).expect("too many types in the library");
        let r = self.last_compiled.clone().expect("no type found after strict encoding procedure");
        (self, r)
    }

    fn report_compiled(mut self, name: Option<TypeName>, ty: Ty<CompileRef>) -> Self {
        let r = match name {
            Some(name) => {
                let new_ty = CompileType::new(name.clone(), ty);
                if let Some(old_ty) = self.types.get(&name) {
                    assert_eq!(
                        old_ty, &new_ty,
                        "repeated type name '{}' for two different types '{}' and '{}'",
                        name, old_ty, new_ty
                    );
                }
                self.types.insert(name.clone(), new_ty).expect("too many types");
                CompileRef::Named(name)
            }
            None => CompileRef::Embedded(Box::new(ty)),
        };
        self.last_compiled = Some(r);
        self
    }
}

#[derive(Debug)]
pub struct StructBuilder<P: BuilderParent> {
    lib: LibName,
    name: Option<TypeName>,
    writer: StructWriter<Sink, P>,
    fields: Vec<CompileRef>,
    cursor: Option<u8>,
}

impl<P: BuilderParent> StructBuilder<P> {
    pub fn with(
        lib: LibName,
        name: Option<TypeName>,
        writer: StructWriter<Sink, P>,
        definer: bool,
    ) -> Self {
        StructBuilder {
            lib,
            name,
            writer,
            fields: empty!(),
            cursor: if definer { Some(0) } else { None },
        }
    }

    pub fn name(&self) -> &str { self.name.as_ref().map(|n| n.as_str()).unwrap_or("<unnamed>") }

    fn _define_field<T: StrictEncode + StrictDumb>(mut self) -> Self {
        let (parent, remnant) = self.writer.into_parent_split();
        let (parent, ty) = parent.compile_type(&T::strict_dumb());
        self.writer = StructWriter::from_parent_split(parent, remnant);
        let _ = self.fields.push(ty); // type repetition is checked by self.parent
        self
    }

    fn _write_field(mut self, value: &impl StrictEncode) -> io::Result<Self> {
        let (parent, remnant) = self.writer.into_parent_split();
        let (parent, ty) = parent.compile_type(value);
        self.writer = StructWriter::from_parent_split(parent, remnant);
        if let Some(pos) = &mut self.cursor {
            let expect_ty = &self.fields[*pos as usize];
            let msg = format!(
                "'{}.{}' has type '{}' instead of '{}'",
                pos,
                self.writer.name(),
                ty,
                expect_ty
            );
            assert_eq!(expect_ty, &ty, "{}", msg);
            *pos += 1;
        }
        self.fields.push(ty);
        Ok(self)
    }

    fn _build_struct(&self) -> Ty<CompileRef> {
        match self.writer.is_tuple() {
            true => {
                let fields = UnnamedFields::try_from(self.fields.clone())
                    .expect(&format!("tuple '{}' has invalid number of fields", self.name()));
                Ty::Tuple(fields)
            }
            false => {
                let fields = self
                    .writer
                    .named_fields()
                    .iter()
                    .enumerate()
                    .map(|(no, name)| {
                        let lib_ref = self.fields.get(no).expect("type guarantees");
                        Field {
                            name: name.clone(),
                            ty: lib_ref.clone(),
                        }
                    })
                    .collect::<Vec<_>>();
                let fields = NamedFields::try_from(fields)
                    .expect(&format!("structure '{}' has invalid number of fields", self.name()));
                Ty::Struct(fields)
            }
        }
    }

    fn _complete_definition(self) -> P {
        let ty = self._build_struct();
        if self.writer.is_tuple() {
            DefineTuple::complete(self.writer).report_compiled(self.name.clone(), ty)
        } else {
            DefineStruct::complete(self.writer).report_compiled(self.name.clone(), ty)
        }
    }

    fn _complete_write(self) -> P {
        let ty = self._build_struct();
        if let Some(pos) = self.cursor {
            assert_eq!(
                pos as usize,
                self.fields.len(),
                "not all fields were written for '{}'",
                self.writer.name()
            );
        }
        if self.writer.is_tuple() {
            WriteTuple::complete(self.writer).report_compiled(self.name.clone(), ty)
        } else {
            WriteStruct::complete(self.writer).report_compiled(self.name.clone(), ty)
        }
    }
}

impl<P: BuilderParent> DefineStruct for StructBuilder<P> {
    type Parent = P;

    fn define_field<T: StrictEncode + StrictDumb>(mut self, name: FieldName) -> Self {
        self.writer = DefineStruct::define_field::<T>(self.writer, name);
        self._define_field::<T>()
    }

    fn complete(self) -> P { self._complete_definition() }
}

impl<P: BuilderParent> WriteStruct for StructBuilder<P> {
    type Parent = P;

    fn write_field(mut self, name: FieldName, value: &impl StrictEncode) -> io::Result<Self> {
        self.writer = WriteStruct::write_field(self.writer, name, value)?;
        self._write_field(value)
    }

    fn complete(self) -> P { self._complete_write() }
}

impl<P: BuilderParent> DefineTuple for StructBuilder<P> {
    type Parent = P;

    fn define_field<T: StrictEncode + StrictDumb>(mut self) -> Self {
        self.writer = DefineTuple::define_field::<T>(self.writer);
        self._define_field::<T>()
    }

    fn complete(self) -> P { self._complete_definition() }
}

impl<P: BuilderParent> WriteTuple for StructBuilder<P> {
    type Parent = P;

    fn write_field(mut self, value: &impl StrictEncode) -> io::Result<Self> {
        self.writer = WriteTuple::write_field(self.writer, value)?;
        self._write_field(value)
    }

    fn complete(self) -> P { self._complete_write() }
}

#[derive(Debug)]
pub struct UnionBuilder {
    lib: LibName,
    name: Option<TypeName>,
    variants: BTreeMap<u8, CompileRef>,
    parent: LibBuilder,
    writer: UnionWriter<Sink>,
    current_ord: u8,
}

impl UnionBuilder {
    pub fn with<T: StrictSum>(parent: LibBuilder) -> Self {
        UnionBuilder {
            lib: libname!(T::STRICT_LIB_NAME),
            name: T::strict_name(),
            variants: empty!(),
            parent,
            writer: UnionWriter::with::<T>(StrictWriter::sink()),
            current_ord: 0,
        }
    }

    fn _fork(&self) -> Self {
        Self {
            lib: self.lib.clone(),
            name: self.name.clone(),
            variants: self.variants.clone(),
            parent: LibBuilder::new(self.lib.clone()),
            writer: UnionWriter::sink(),
            current_ord: self.current_ord,
        }
    }

    pub fn name(&self) -> &str { self.name.as_ref().map(|n| n.as_str()).unwrap_or("<unnamed>") }

    fn _define_field(&mut self, ord: Option<u8>) {
        self.current_ord = ord.unwrap_or_else(|| self.writer.next_ord()) - 1;
        let ty = self.parent.last_compiled.clone().expect("no compiled type found");
        self.variants.insert(self.current_ord, ty);
    }

    fn _write_field(&mut self, name: FieldName) {
        self.current_ord = self.writer.ord_by_name(&name).expect(&format!(
            "writing field '{}' of '{}' without declaration",
            name,
            self.name()
        ));
    }

    fn _build_union(&self) -> Ty<CompileRef> {
        let variants = self
            .writer
            .variants()
            .keys()
            .map(|variant| {
                let lib_ref = self.variants.get(&variant.ord).expect("type guarantees");
                (variant.clone(), lib_ref.clone())
            })
            .collect::<BTreeMap<_, _>>();
        let fields = UnionVariants::try_from(variants)
            .expect(&format!("union '{}' has invalid number of variants", self.name()));
        Ty::Union(fields)
    }

    fn _build_enum(&self) -> Ty<CompileRef> {
        let fields = self.writer.variants().keys().cloned().collect::<BTreeSet<_>>();
        let fields = EnumVariants::try_from(fields)
            .expect(&format!("enum '{}' has invalid number of variants", self.name()));
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

    fn from_split(writer: UnionWriter<Sink>, mut remnant: Self) -> Self {
        remnant.writer = writer;
        remnant
    }

    fn into_split(self) -> (UnionWriter<Sink>, UnionBuilder) {
        let (_, writer) = self.writer.into_write_split();
        let remnant = Self {
            lib: self.lib,
            name: self.name,
            variants: self.variants,
            parent: self.parent,
            writer: UnionWriter::sink(),
            current_ord: self.current_ord,
        };
        (writer, remnant)
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
    fn compile_type<T: StrictEncode>(mut self, value: &T) -> (Self, CompileRef) {
        let (parent, r) = self.parent.compile_type(value);
        self.parent = parent;
        (self, r)
    }
    fn report_compiled(mut self, name: Option<TypeName>, ty: Ty<CompileRef>) -> Self {
        self.variants.insert(self.current_ord, CompileRef::Embedded(Box::new(ty.clone())));
        self.parent = self.parent.report_compiled(name, ty);
        self
    }
}

impl DefineEnum for UnionBuilder {
    type Parent = LibBuilder;
    type EnumWriter = Self;

    fn define_variant(mut self, name: FieldName, value: u8) -> Self {
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

    fn write_variant(mut self, name: FieldName) -> io::Result<Self> {
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

    fn define_unit(mut self, name: FieldName) -> Self {
        self.parent = self.parent.report_compiled(None, Ty::UNIT);
        self.writer = DefineUnion::define_unit(self.writer, name);
        self._define_field(None);
        self
    }

    fn define_tuple(
        mut self,
        name: FieldName,
        inner: impl FnOnce(Self::TupleDefiner) -> Self,
    ) -> Self {
        let lib = self.lib.clone();
        let (writer, remnant) = self.into_split();
        let mut clone = remnant._fork();
        let mut lib_builder = clone.parent;
        let writer = writer.define_tuple(name, |d| {
            let (writer, _) = d.into_parent_split();
            let mut reconstructed_self = Self::from_split(writer, remnant);
            let struct_writer = StructWriter::unnamed(reconstructed_self, true);
            let struct_builder = StructBuilder::with(lib, None, struct_writer, true);
            reconstructed_self = inner(struct_builder);
            lib_builder = reconstructed_self.parent;
            reconstructed_self.writer
        });
        clone.parent = lib_builder;
        self = Self::from_split(writer, clone);
        self._define_field(None);
        self
    }

    fn define_struct(
        mut self,
        name: FieldName,
        inner: impl FnOnce(Self::StructDefiner) -> Self,
    ) -> Self {
        let lib = self.lib.clone();
        let (writer, remnant) = self.into_split();
        let mut clone = remnant._fork();
        let mut lib_builder = clone.parent;
        let writer = writer.define_struct(name, |d| {
            let (writer, _) = d.into_parent_split();
            let mut reconstructed_self = Self::from_split(writer, remnant);
            let struct_writer = StructWriter::unnamed(reconstructed_self, false);
            let struct_builder = StructBuilder::with(lib, None, struct_writer, true);
            reconstructed_self = inner(struct_builder);
            lib_builder = reconstructed_self.parent;
            reconstructed_self.writer
        });
        clone.parent = lib_builder;
        self = Self::from_split(writer, clone);
        self._define_field(None);
        self
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

    fn write_unit(mut self, name: FieldName) -> io::Result<Self> {
        self.parent = self.parent.report_compiled(None, Ty::UNIT);
        self.writer = WriteUnion::write_unit(self.writer, name.clone())?;
        self._write_field(name);
        Ok(self)
    }

    fn write_tuple(
        mut self,
        name: FieldName,
        inner: impl FnOnce(Self::TupleWriter) -> io::Result<Self>,
    ) -> io::Result<Self> {
        let lib = self.lib.clone();
        let (writer, remnant) = self.into_split();
        let mut clone = remnant._fork();
        let mut lib_builder = clone.parent;
        let writer = writer.write_tuple(name.clone(), |d| {
            let (writer, _) = d.into_parent_split();
            let mut reconstructed_self = Self::from_split(writer, remnant);
            let struct_writer = StructWriter::unnamed(reconstructed_self, true);
            let struct_builder = StructBuilder::with(lib, None, struct_writer, false);
            reconstructed_self = inner(struct_builder)?;
            lib_builder = reconstructed_self.parent;
            Ok(reconstructed_self.writer)
        })?;
        clone.parent = lib_builder;
        self = Self::from_split(writer, clone);
        self._write_field(name);
        Ok(self)
    }

    fn write_struct(
        mut self,
        name: FieldName,
        inner: impl FnOnce(Self::StructWriter) -> io::Result<Self>,
    ) -> io::Result<Self> {
        let lib = self.lib.clone();
        let (writer, remnant) = self.into_split();
        let mut clone = remnant._fork();
        let mut lib_builder = clone.parent;
        let writer = writer.write_struct(name.clone(), |d| {
            let (writer, _) = d.into_parent_split();
            let mut reconstructed_self = Self::from_split(writer, remnant);
            let struct_writer = StructWriter::unnamed(reconstructed_self, false);
            let struct_builder = StructBuilder::with(lib, None, struct_writer, false);
            reconstructed_self = inner(struct_builder)?;
            lib_builder = reconstructed_self.parent;
            Ok(reconstructed_self.writer)
        })?;
        clone.parent = lib_builder;
        self = Self::from_split(writer, clone);
        self._write_field(name);
        Ok(self)
    }

    fn complete(self) -> LibBuilder {
        let ty = self._build_union();
        self._complete_write(ty)
    }
}
