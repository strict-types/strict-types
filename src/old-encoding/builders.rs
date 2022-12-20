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

use std::io::Error;
use std::ops::DerefMut;

use crate::ast::{Field, Fields};
use crate::{Encode, FieldName, LibName, LibRef, StenWrite, Ty, TypeName};

pub trait TypedWrite: Sized {
    type TupleWriter: WriteTuple<Self>;
    type StructWriter: WriteStruct<Self>;
    type UnionWriter: WriteUnion<Self>;
    type EnumWriter: WriteEnum<Self>;

    fn write_primitive(self) -> Self::PrimitiveWriter;
    fn write_tuple(self, lib: LibName, name: Option<TypeName>) -> Self::TupleWriter;
    fn write_struct(self, lib: LibName, name: Option<TypeName>) -> Self::StructWriter;
    fn write_union(self, lib: LibName, name: Option<TypeName>) -> Self::UnionWriter;
    fn write_enum(self, lib: LibName, name: Option<TypeName>) -> Self::EnumWriter;
}

pub trait WriteTuple<P: Sized>: Sized {
    fn write_field(self, value: &impl Encode) -> Self;
    fn complete(self) -> P;
}

pub trait WriteStruct<P: Sized>: Sized {
    fn write_field(self, name: FieldName, value: &impl Encode) -> Self;
    fn complete(self) -> P;
}

pub trait WriteUnion<P: Sized>: Sized {
    type TupleWriter: WriteTuple<Self>;
    type StructWriter: WriteStruct<Self>;

    fn write_unit(self, name: FieldName) -> Self;
    fn write_type(self, name: FieldName, value: &impl Encode) -> Self {
        self.write_tuple(name).write_field(value).complete()
    }
    fn write_tuple(self, name: FieldName) -> Self::TupleWriter;
    fn write_struct(self, name: FieldName) -> Self::StructWriter;
    fn complete(self) -> P;
}

pub struct TypeBuilder {}

impl TypeBuilder {
    pub fn process(&mut self, value: &impl Encode) -> LibRef { todo!() }
    pub fn complete(self, lib: LibName, name: Option<TypeName>, ty: Ty<LibRef>) { todo!() }
}

pub trait BuilderParent: Sized + DerefMut<Target = TypeBuilder> {}

pub struct StructBuilder<P: BuilderParent> {
    lib: LibName,
    name: Option<TypeName>,
    fields: Fields<LibRef, true>,
    ord: u8,
    parent: P,
}

impl<P: BuilderParent> WriteStruct<P> for StructBuilder<P> {
    fn write_field(mut self, name: FieldName, value: &impl Encode) -> Self {
        let ty = self.parent.process(value);
        self.fields
            .insert(Field::named(name, self.ord), ty)
            .expect("type has too many fields")
            .expect("repeated field name");
        self.ord += 1;
        self
    }

    fn complete(self) -> P {
        self.parent.complete(self.lib, self.name, Ty::Struct(self.fields));
        self.parent
    }
}

enum Example {
    Init(u8),
    Ping,
    Connect { host: Option<Vec<u8>> },
}

impl Encode for Example {
    fn sten_dumb() -> Self { Example::Ping }

    fn encode(&self, writer: impl TypedWrite) -> Result<(), Error> {
        let union = writer
            .write_union("Test", Some("Example"))
            .define_type("init", u8::sten_dumb())
            .define_unit("ping")
            .define_struct("connect")
            .define_field("host", Some(Vec::<u8>::sten_dumb()))
            .complete();
        match self {
            Example::Init(val) => union.write_value("init", val),
            Example::Ping => union.write_unit("ping"),
            Example::Connect { host } => union.write_struct("connect").write_field(host).complete(),
        }
        Ok(());

        reader.read_union("Test", Some("Example"), |field, r| match field {
            f!(0u8, "init") => Example::Init(r.read_type()),
            f!(2u8, "connect") => Example::Connect {
                host: r.read_struct().read_field("host").complete(),
            },
        })
    }
}
