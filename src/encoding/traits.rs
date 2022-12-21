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

use std::io::{BufRead, Seek};
use std::{fs, io};

use amplify::confinement::Confined;

use super::DecodeError;
use crate::encoding::{DeserializeError, SerializeError, StrictReader, StrictWriter};
use crate::Ident;

pub trait ToIdent {
    fn to_ident(&self) -> Ident;
}
impl ToIdent for &'static str {
    fn to_ident(&self) -> Ident { Ident::from(*self) }
}
impl ToIdent for String {
    fn to_ident(&self) -> Ident { Ident::try_from(self.to_owned()).expect("invalid identifier") }
}
pub trait ToMaybeIdent {
    fn to_maybe_ident(&self) -> Option<Ident>;
}
impl<T> ToMaybeIdent for Option<T>
where T: ToIdent
{
    fn to_maybe_ident(&self) -> Option<Ident> { self.as_ref().map(|n| n.to_ident()) }
}

pub trait TypedWrite: Sized {
    type TupleWriter: WriteTuple<Self>;
    type StructWriter: WriteStruct<Self>;
    type UnionWriter: WriteUnion<Self>;
    type EnumWriter: WriteEnum<Self>;

    // TODO: Remove optionals
    fn write_tuple(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::TupleWriter;
    fn write_type(
        self,
        ns: impl ToIdent,
        name: Option<impl ToIdent>,
        value: &impl StrictEncode,
    ) -> io::Result<Self> {
        Ok(self.write_tuple(ns, name).write_field(value)?.complete())
    }
    fn write_struct(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::StructWriter;
    fn write_union(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::UnionWriter;
    fn write_enum(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::EnumWriter;

    unsafe fn write_raw<const LEN: usize>(self, raw: [u8; LEN]) -> io::Result<Self>;
}

pub trait DefineTuple<P: Sized>: Sized {
    fn define_field<T: StrictEncode>(self) -> Self;
    fn define_field_ord<T: StrictEncode>(self, ord: u8) -> Self;
    fn complete(self) -> P;
}

pub trait WriteTuple<P: Sized>: Sized {
    fn write_field(self, value: &impl StrictEncode) -> io::Result<Self>;
    fn write_field_ord(self, ord: u8, value: &impl StrictEncode) -> io::Result<Self>;
    fn complete(self) -> P;
}

pub trait DefineStruct<P: Sized>: Sized {
    fn define_field<T: StrictEncode>(self, name: impl ToIdent) -> Self;
    fn define_field_ord<T: StrictEncode>(self, name: impl ToIdent, ord: u8) -> Self;
    fn complete(self) -> P;
}

pub trait WriteStruct<P: Sized>: Sized {
    fn write_field(self, name: impl ToIdent, value: &impl StrictEncode) -> io::Result<Self>;
    fn write_field_ord(
        self,
        name: impl ToIdent,
        ord: u8,
        value: &impl StrictEncode,
    ) -> io::Result<Self>;
    fn complete(self) -> P;
}

pub trait WriteEnum<P: Sized>: Sized {
    fn define_variant(self, name: impl ToIdent, value: u8) -> Self;
    fn write_variant(self, name: impl ToIdent) -> io::Result<Self>;
    fn complete(self) -> P;
}

pub trait WriteUnion<P: Sized>: Sized {
    type TupleDefiner: DefineTuple<Self>;
    type StructDefiner: DefineStruct<Self>;
    type TupleWriter: WriteTuple<Self>;
    type StructWriter: WriteStruct<Self>;

    fn define_unit(self, name: impl ToIdent) -> Self;
    fn define_type<T: StrictEncode>(self, name: impl ToIdent) -> Self {
        self.define_tuple(name).define_field::<T>().complete()
    }
    fn define_tuple(self, name: impl ToIdent) -> Self::TupleDefiner;
    fn define_struct(self, name: impl ToIdent) -> Self::StructDefiner;

    fn write_unit(self, name: impl ToIdent) -> io::Result<Self>;
    fn write_type(self, name: impl ToIdent, value: &impl StrictEncode) -> io::Result<Self> {
        Ok(self.write_tuple(name).write_field(value)?.complete())
    }
    fn write_tuple(self, name: impl ToIdent) -> Self::TupleWriter;
    fn write_struct(self, name: impl ToIdent) -> Self::StructWriter;

    fn complete(self) -> P;
}

pub trait TypedRead: Sized {}

pub trait StrictEncode {
    fn strict_encode_dumb() -> Self;
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W>;
}

pub trait StrictDecode: Sized {
    fn strict_decode(reader: &impl TypedRead) -> Result<Self, DecodeError>;
}

pub trait Serialize: StrictEncode {
    fn strict_serialized_len(&self) -> io::Result<usize> {
        let counter = StrictWriter::counter();
        Ok(self.strict_encode(counter)?.unbox().count)
    }

    fn to_strict_serialized<const MAX: usize>(
        &self,
    ) -> Result<Confined<Vec<u8>, 0, MAX>, SerializeError> {
        let ast_data = StrictWriter::in_memory(MAX);
        let data = self.strict_encode(ast_data)?.unbox();
        Confined::<Vec<u8>, 0, MAX>::try_from(data).map_err(SerializeError::from)
    }

    fn strict_serialize_to_file<const MAX: usize>(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), SerializeError> {
        let file = StrictWriter::with(MAX, fs::File::create(path)?);
        self.strict_encode(file)?;
        Ok(())
    }
}

pub trait Deserialize: StrictDecode {
    fn from_strict_serialized<const MAX: usize>(
        ast_data: Confined<Vec<u8>, 0, MAX>,
    ) -> Result<Self, DeserializeError> {
        let cursor = io::Cursor::new(ast_data.into_inner());
        let mut reader = StrictReader::with(MAX, cursor);
        let me = Self::strict_decode(&mut reader)?;
        let mut cursor = reader.unbox();
        if !cursor.fill_buf()?.is_empty() {
            return Err(DeserializeError::DataNotEntirelyConsumed);
        }
        Ok(me)
    }

    fn strict_deserialize_from_file<const MAX: usize>(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, DeserializeError> {
        let file = fs::File::open(path)?;
        let mut reader = StrictReader::with(MAX, file);
        let me = Self::strict_decode(&mut reader)?;
        let mut file = reader.unbox();
        if file.stream_position()? != file.seek(io::SeekFrom::End(0))? {
            return Err(DeserializeError::DataNotEntirelyConsumed);
        }
        Ok(me)
    }
}
