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
use amplify::confinement::SmallOrdMap;

use super::compile::{CompileRef, CompileType};
use crate::ast::{Fields, Variants};
use crate::encoding::{
    DefineEnum, DefineStruct, DefineTuple, DefineUnion, SplitParent, StrictEncode, StrictParent,
    StrictWriter, StructWriter, ToIdent, ToMaybeIdent, TypedParent, TypedWrite, UnionWriter,
    WriteEnum, WriteStruct, WriteTuple, WriteUnion,
};
use crate::{LibName, Ty, TypeLib, TypeName};

#[derive(Default)]
pub struct LibBuilder {
    types: SmallOrdMap<TypeName, CompileType>,
    pub(self) last_compiled: Option<CompileRef>,
}

impl LibBuilder {
    pub(crate) fn new() -> LibBuilder {
        LibBuilder {
            types: default!(),
            last_compiled: None,
        }
    }

    pub(crate) fn finalize(self, name: LibName) -> Result<TypeLib, confinement::Error> {
        todo!()
        // TODO: translate from CompileType to LibRef
        /*
        let types = Confined::try_from(self.types.into_inner())?;
        Ok(TypeLib {
            name,
            dependencies: none!(),
            types,
        })*/
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
    types: BTreeMap<u8, CompileRef>,
}

impl<P: BuilderParent> StructBuilder<P> {
    pub fn with(name: Option<TypeName>, parent: P) -> Self {
        StructBuilder {
            name: name.clone(),
            writer: StructWriter::with(name, parent),
            types: empty!(),
        }
    }

    pub fn name(&self) -> &str { self.name.as_ref().map(|n| n.as_str()).unwrap_or("<unnamed>") }

    fn _define_field<T: StrictEncode>(mut self, ord: u8) -> Self {
        let (parent, remnant) = self.writer.into_parent_split();
        let (parent, ty) = parent.compile_type(&T::strict_encode_dumb());
        self.writer = StructWriter::from_parent_split(parent, remnant);
        self.types.insert(ord, ty).expect("checked by self.writer");
        self
    }

    fn _write_field(mut self, ord: u8, value: &impl StrictEncode) -> io::Result<Self> {
        let (parent, remnant) = self.writer.into_parent_split();
        let (parent, ty) = parent.compile_type(value);
        self.writer = StructWriter::from_parent_split(parent, remnant);
        let expect_ty = self.types.get(&ord).expect("type guarantees");
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
            .expect(&format!("structure {} has invalid number of fields", self.name()));
        WriteStruct::complete(self.writer).report_compiled(self.name.clone(), Ty::Struct(fields))
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
    types: BTreeMap<u8, CompileRef>,
    parent: LibBuilder,
    writer: UnionWriter<Sink>,
    current_ord: u8,
}

impl UnionBuilder {
    pub fn with(name: Option<TypeName>, parent: LibBuilder) -> Self {
        UnionBuilder {
            name: name.clone(),
            types: empty!(),
            parent,
            writer: UnionWriter::with(name, StrictWriter::sink()),
            current_ord: 0,
        }
    }

    pub fn name(&self) -> &str { self.name.as_ref().map(|n| n.as_str()).unwrap_or("<unnamed>") }

    fn _complete_definition(mut self) -> UnionBuilder {
        self.writer = DefineUnion::complete(self.writer);
        self
    }

    fn _complete_write(self, ty: Ty<CompileRef>) -> LibBuilder {
        let _ = WriteUnion::complete(self.writer);
        self.parent.report_compiled(self.name, ty)
    }
}

impl DefineEnum for UnionBuilder {
    type Parent = LibBuilder;
    type EnumWriter = Self;

    fn define_variant(mut self, name: impl ToIdent, value: u8) -> Self {
        self.current_ord = value;
        self.writer = DefineEnum::define_variant(self.writer, name, value);
        self
    }

    fn complete(self) -> Self::EnumWriter { self._complete_definition() }
}

impl WriteEnum for UnionBuilder {
    type Parent = LibBuilder;

    fn write_variant(mut self, name: impl ToIdent) -> io::Result<Self> {
        self.current_ord = self.writer.next_ord();
        self.writer = WriteEnum::write_variant(self.writer, name)?;
        Ok(self)
    }

    fn complete(self) -> LibBuilder {
        let fields = self.writer.variants().keys().cloned().collect::<BTreeSet<_>>();
        let fields = Variants::try_from(fields)
            .expect(&format!("enum {} has invalid number of variants", self.name()));
        self._complete_write(Ty::Enum(fields))
    }
}

impl DefineUnion for UnionBuilder {
    type Parent = LibBuilder;
    type TupleDefiner = StructBuilder<Self>;
    type StructDefiner = StructBuilder<Self>;
    type UnionWriter = Self;

    fn define_unit(mut self, name: impl ToIdent) -> Self {
        self.current_ord = self.writer.next_ord();
        self.writer = DefineUnion::define_unit(self.writer, name);
        self
    }

    fn define_tuple(mut self, name: impl ToIdent) -> Self::TupleDefiner {
        self.current_ord = self.writer.next_ord();
        let sink = DefineUnion::define_tuple(self.writer, name);
        self.writer = sink.into_parent();
        StructBuilder::with(None, self)
    }

    fn define_struct(mut self, name: impl ToIdent) -> Self::StructDefiner {
        self.current_ord = self.writer.next_ord();
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
        self.current_ord = self.writer.next_ord();
        self.writer = WriteUnion::write_unit(self.writer, name)?;
        Ok(self)
    }

    fn write_tuple(mut self, name: impl ToIdent) -> io::Result<Self::TupleWriter> {
        self.current_ord = self.writer.next_ord();
        let sink = WriteUnion::write_tuple(self.writer, name)?;
        self.writer = sink.into_parent();
        Ok(StructBuilder::with(None, self))
    }

    fn write_struct(mut self, name: impl ToIdent) -> io::Result<Self::StructWriter> {
        self.current_ord = self.writer.next_ord();
        let sink = WriteUnion::write_struct(self.writer, name)?;
        self.writer = sink.into_parent();
        Ok(StructBuilder::with(None, self))
    }

    fn complete(self) -> LibBuilder {
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
            .expect(&format!("union {} has invalid number of variants", self.name()));
        self._complete_write(Ty::Union(fields))
    }
}

pub trait BuilderParent: StrictParent<Sink> {
    /// Converts strict-encodable value into a type information. Must be propagated back to the
    /// lib builder which does the TypedWrite implementation to call strict encode on the type
    fn compile_type(self, value: &impl StrictEncode) -> (Self, CompileRef);
    /// Notifies lib builder about complete type built, even for unnamed inline types, such that it
    /// can register last compiled type for the `compile_type` procedure.
    fn report_compiled(self, name: Option<TypeName>, ty: Ty<CompileRef>) -> Self;
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
        let (mut writer, remnant) = self.into_write_split();
        writer = value.strict_encode(writer).expect("too many types in the library");
        self = Self::from_write_split(writer, remnant);
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
        self.types.insert(self.current_ord, CompileRef::Inline(Box::new(ty.clone())));
        self.parent = self.parent.report_compiled(name, ty);
        self
    }
}
