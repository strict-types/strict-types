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

use std::io;

use amplify::WriteCounter;

use crate::encoding::{
    StrictEncode, ToIdent, TypedWrite, WriteEnum, WritePrimitive, WriteStruct, WriteTuple,
    WriteUnion,
};

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
    type PrimitiveWriter = PrimitiveWriter<W>;
    type TupleWriter = TupleWriter<W>;
    type StructWriter = StructWriter<W>;
    type UnionWriter = UnionWriter<W>;
    type EnumWriter = EnumWriter<W>;

    fn write_primitive(self) -> Self::PrimitiveWriter { todo!() }

    fn write_tuple(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::TupleWriter {
        todo!()
    }

    fn write_struct(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::StructWriter {
        StructWriter {
            ns: ns.to_owned(),
            name: name.map(|n| n.to_owned()),
            writer: self,
        }
    }

    fn write_union(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::UnionWriter {
        todo!()
    }

    fn write_enum(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::EnumWriter {
        todo!()
    }
}

pub struct StructWriter<W: io::Write> {
    ns: String,
    name: Option<String>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WriteStruct<P> for StructWriter<W> {
    fn write_field(self, _name: impl ToIdent, value: &impl StrictEncode) -> io::Result<Self> {
        value.strict_encode(&self.writer).map(|_| self)
    }

    fn write_field_ord(
        self,
        _name: impl ToIdent,
        _ord: u8,
        value: &impl StrictEncode,
    ) -> io::Result<Self> {
        value.strict_encode(&self.writer).map(|_| self)
    }

    fn complete(self) -> P { P::from(self.writer) }
}

pub struct PrimitiveWriter<W: io::Write> {
    ns: String,
    name: Option<String>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WritePrimitive<P> for PrimitiveWriter<W> {
    fn write__(self, value: &impl StrictEncode) -> io::Result<Self> { todo!() }

    fn complete(self) -> P { P::from(self.writer) }
}

pub struct TupleWriter<W: io::Write> {
    ns: String,
    name: Option<String>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WriteTuple<P> for TupleWriter<W> {
    fn write_field(self, value: &impl StrictEncode) -> io::Result<Self> { todo!() }

    fn write_field_ord(self, ord: u8, value: &impl StrictEncode) -> io::Result<Self> { todo!() }

    fn complete(self) -> P { P::from(self.writer) }
}

pub struct UnionWriter<W: io::Write> {
    ns: String,
    name: Option<String>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WriteUnion<P> for UnionWriter<W> {
    type TupleWriter = TupleWriter<W>;
    type StructWriter = StructWriter<W>;

    fn write_unit(self, name: impl ToIdent) -> io::Result<Self> { todo!() }

    fn write_tuple(self, name: impl ToIdent) -> Self::TupleWriter { todo!() }

    fn write_struct(self, name: impl ToIdent) -> Self::StructWriter { todo!() }

    fn complete(self) -> P { P::from(self.writer) }
}

pub struct EnumWriter<W: io::Write> {
    ns: String,
    name: Option<String>,
    writer: StrictWriter<W>,
}

impl<W: io::Write, P: Sized + From<StrictWriter<W>>> WriteEnum<P> for EnumWriter<W> {
    fn write_variant(self, name: impl ToIdent, value: u8) -> io::Result<Self> { todo!() }

    fn complete(self) -> P { P::from(self.writer) }
}

impl<W: io::Write> From<StrictWriter<W>> for UnionWriter<W> {
    fn from(writer: StrictWriter<W>) -> Self {
        UnionWriter {
            ns: "".to_string(),
            name: None,
            writer,
        }
    }
}
