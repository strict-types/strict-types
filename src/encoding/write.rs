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

use std::collections::BTreeSet;
use std::io;

use amplify::WriteCounter;

use crate::ast::Field;
use crate::encoding::{
    DefineEnum, DefineStruct, DefineTuple, DefineUnion, StrictEncode, ToIdent, ToMaybeIdent,
    TypedWrite, WriteEnum, WriteStruct, WriteTuple, WriteUnion,
};
use crate::Ident;

// TODO: Move to amplify crate
#[derive(Debug)]
pub struct CountingWriter<W: io::Write> {
    count: usize,
    limit: usize,
    writer: W,
}

impl<W: io::Write> From<W> for CountingWriter<W> {
    fn from(writer: W) -> Self {
        Self {
            count: 0,
            limit: usize::MAX,
            writer,
        }
    }
}

impl<W: io::Write> CountingWriter<W> {
    pub fn with(limit: usize, writer: W) -> Self {
        Self {
            count: 0,
            limit,
            writer,
        }
    }

    pub fn unbox(self) -> W { self.writer }
}

impl<W: io::Write> io::Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.count + buf.len() > self.limit {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        let count = self.writer.write(buf)?;
        self.count += count;
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> { self.writer.flush() }
}

#[derive(Debug, From)]
pub struct StrictWriter<W: io::Write>(CountingWriter<W>);

impl StrictWriter<Vec<u8>> {
    pub fn in_memory(limit: usize) -> Self { StrictWriter(CountingWriter::with(limit, vec![])) }
}

impl StrictWriter<WriteCounter> {
    pub fn counter() -> Self { StrictWriter(CountingWriter::from(WriteCounter::default())) }
}

impl<W: io::Write> StrictWriter<W> {
    pub fn with(limit: usize, writer: W) -> Self {
        StrictWriter(CountingWriter::with(limit, writer))
    }
    pub fn unbox(self) -> W { self.0.unbox() }
}

impl<W: io::Write> TypedWrite for StrictWriter<W> {
    type TupleWriter = StructWriter<W>;
    type StructWriter = StructWriter<W>;
    type UnionDefiner = UnionDefiner<W>;
    type EnumDefiner = EnumDefiner<W>;

    fn define_union(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::UnionDefiner {
        todo!()
    }
    fn define_enum(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::EnumDefiner {
        todo!()
    }
    fn write_tuple(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::TupleWriter {
        StructWriter::with(ns, name, self)
    }
    fn write_struct(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::StructWriter {
        StructWriter::with(ns, name, self)
    }
    unsafe fn _write_raw<const LEN: usize>(mut self, bytes: impl AsRef<[u8]>) -> io::Result<Self> {
        use io::Write;
        self.0.write_all(bytes.as_ref())?;
        Ok(self)
    }
}

pub struct StructWriter<W: io::Write> {
    ns: Ident,
    name: Option<Ident>,
    fields: BTreeSet<Field>,
    ords: BTreeSet<u8>,
    writer: StrictWriter<W>,
    defined: bool,
}

impl<W: io::Write> StructWriter<W> {
    fn with(ns: impl ToIdent, name: Option<impl ToIdent>, writer: StrictWriter<W>) -> Self {
        StructWriter {
            ns: ns.to_ident(),
            name: name.to_maybe_ident(),
            fields: empty!(),
            ords: empty!(),
            writer,
            defined: false,
        }
    }

    fn name(&self) -> String {
        format!("{}.{}", &self.ns, self.name.as_ref().unwrap_or(&tn!("<unnamed>")))
    }

    fn next_ord(&self) -> u8 { self.fields.iter().max().map(|f| f.ord + 1).unwrap_or_default() }

    fn _define_field(mut self, field: Field) -> Self {
        assert!(
            self.fields.insert(field.clone()),
            "field {} is already defined as a part of {}",
            &field,
            self.name()
        );
        self.ords.insert(field.ord);
        self
    }

    fn _write_field(mut self, field: Field, value: &impl StrictEncode) -> io::Result<Self> {
        if self.defined {
            assert!(
                !self.fields.contains(&field),
                "field {} was not defined in {}",
                &field,
                self.name()
            )
        } else {
            self = self._define_field(field.clone());
        }
        assert!(
            self.ords.remove(&field.ord),
            "field {} was already written before in {} struct",
            &field,
            self.name()
        );
        self.writer = field.ord.strict_encode(self.writer)?;
        self.writer = value.strict_encode(self.writer)?;
        Ok(self)
    }

    fn _complete_definition<P: Sized + From<StrictWriter<W>>>(mut self) -> P {
        assert!(!self.fields.is_empty(), "struct {} does not have fields defined", self.name());
        self.defined = true;
        P::from(self.writer)
    }

    fn _complete_write<P: Sized + From<StrictWriter<W>>>(self) -> P {
        assert!(self.ords.is_empty(), "not all fields were written for {}", self.name());
        P::from(self.writer)
    }
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> DefineStruct<P> for StructWriter<W> {
    fn define_field<T: StrictEncode>(self, name: impl ToIdent) -> Self {
        let ord = self.next_ord();
        DefineStruct::<P>::define_field_ord::<T>(self, name, ord)
    }
    fn define_field_ord<T: StrictEncode>(self, name: impl ToIdent, ord: u8) -> Self {
        let field = Field::named(name.to_ident(), ord);
        self._define_field(field)
    }
    fn complete(self) -> P { self._complete_definition() }
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WriteStruct<P> for StructWriter<W> {
    fn write_field(self, name: impl ToIdent, value: &impl StrictEncode) -> io::Result<Self> {
        let ord = self.next_ord();
        WriteStruct::<P>::write_field_ord(self, name, ord, value)
    }
    fn write_field_ord(
        self,
        name: impl ToIdent,
        ord: u8,
        value: &impl StrictEncode,
    ) -> io::Result<Self> {
        let field = Field::named(name.to_ident(), ord);
        self._write_field(field, value)
    }
    fn complete(self) -> P { self._complete_write() }
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> DefineTuple<P> for StructWriter<W> {
    fn define_field<T: StrictEncode>(self) -> Self {
        let ord = self.next_ord();
        DefineTuple::<P>::define_field_ord::<T>(self, ord)
    }
    fn define_field_ord<T: StrictEncode>(self, ord: u8) -> Self {
        let field = Field::unnamed(ord);
        self._define_field(field)
    }
    fn complete(self) -> P { self._complete_definition() }
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WriteTuple<P> for StructWriter<W> {
    fn write_field(self, value: &impl StrictEncode) -> io::Result<Self> {
        let ord = self.next_ord();
        WriteTuple::<P>::write_field_ord(self, ord, value)
    }
    fn write_field_ord(self, ord: u8, value: &impl StrictEncode) -> io::Result<Self> {
        let field = Field::unnamed(ord);
        self._write_field(field, value)
    }
    fn complete(self) -> P { self._complete_write() }
}

pub struct UnionDefiner<W: io::Write> {
    ns: Ident,
    name: Option<Ident>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> DefineUnion<P> for UnionDefiner<W> {
    type TupleDefiner = StructWriter<W>;
    type StructDefiner = StructWriter<W>;
    type UnionWriter = UnionWriter<W>;

    fn define_unit(self, name: impl ToIdent) -> Self { todo!() }
    fn define_tuple(self, name: impl ToIdent) -> Self::TupleDefiner { todo!() }
    fn define_struct(self, name: impl ToIdent) -> Self::StructDefiner { todo!() }
    fn complete(self) -> Self::UnionWriter { todo!() }
}

pub struct UnionWriter<W: io::Write> {
    ns: Ident,
    name: Option<Ident>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WriteUnion<P> for UnionWriter<W> {
    type TupleWriter = StructWriter<W>;
    type StructWriter = StructWriter<W>;

    fn write_unit(self, name: impl ToIdent) -> io::Result<Self> { todo!() }
    fn write_tuple(self, name: impl ToIdent) -> Self::TupleWriter { todo!() }
    fn write_struct(self, name: impl ToIdent) -> Self::StructWriter { todo!() }
    fn complete(self) -> P { P::from(self.writer) }
}

pub struct EnumDefiner<W: io::Write> {
    ns: Ident,
    name: Option<Ident>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> DefineEnum<P> for EnumDefiner<W> {
    type EnumWriter = EnumWriter<W>;
    fn define_variant(self, name: impl ToIdent, value: u8) -> Self { todo!() }
    fn complete(self) -> Self::EnumWriter { todo!() }
}

pub struct EnumWriter<W: io::Write> {
    ns: Ident,
    name: Option<Ident>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WriteEnum<P> for EnumWriter<W> {
    fn write_variant(self, name: impl ToIdent) -> io::Result<Self> { todo!() }
    fn complete(self) -> P { P::from(self.writer) }
}

impl<W: io::Write> From<StrictWriter<W>> for UnionDefiner<W> {
    fn from(writer: StrictWriter<W>) -> Self {
        UnionDefiner {
            ns: tn!(""),
            name: None,
            writer,
        }
    }
}

impl<W: io::Write> From<StrictWriter<W>> for UnionWriter<W> {
    fn from(writer: StrictWriter<W>) -> Self {
        UnionWriter {
            ns: tn!(""),
            name: None,
            writer,
        }
    }
}
