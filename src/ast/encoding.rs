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
use std::ops::Deref;

use amplify::Wrapper;

use crate::ast::{Field, Fields, Step, Variants};
use crate::encoding::{
    DefineTuple, DefineUnion, StrictEncode, TypedWrite, WriteStruct, WriteTuple, WriteUnion,
};
use crate::util::Sizing;
use crate::{FieldName, Ident, KeyTy, SemId, Ty, TypeRef};

impl StrictEncode for SemId {
    fn strict_encode_dumb() -> Self { SemId::from(blake3::Hash::from([5u8; 32])) }
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_type(Some("SemId"), self.as_bytes())
    }
}

impl StrictEncode for Step {
    fn strict_encode_dumb() -> Self { Step::Index }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        let u = writer
            .define_union(Some("Step"))
            .define_type::<FieldName>("namedField")
            .define_type::<u8>("unnamedField")
            .define_unit("index")
            .define_unit("list")
            .define_unit("set")
            .define_unit("map")
            .complete();

        let u = match self {
            Step::NamedField(name) => u.write_type("namedField", name),
            Step::UnnamedField(ord) => u.write_type("unnamedField", ord),
            Step::Index => u.write_unit("index"),
            Step::List => u.write_unit("list"),
            Step::Set => u.write_unit("set"),
            Step::Map => u.write_unit("map"),
        }?;

        Ok(u.complete())
    }
}

impl StrictEncode for Sizing {
    fn strict_encode_dumb() -> Self { Sizing::U16_NONEMPTY }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        Ok(writer
            .write_struct(Some("Sizing"))
            .write_field("min", &self.min)?
            .write_field("max", &self.max)?
            .complete())
    }
}

impl StrictEncode for Field {
    fn strict_encode_dumb() -> Self { Field::unnamed(0) }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        Ok(writer
            .write_struct(Some("Field"))
            .write_field("name", &self.name)?
            .write_field("ord", &self.ord)?
            .complete())
    }
}

impl<Ref: TypeRef, const OP: bool> StrictEncode for Fields<Ref, OP> {
    fn strict_encode_dumb() -> Self {
        fields! {
            "a" => Ref::strict_encode_dumb()
        }
    }
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_type(Some("Fields"), self.deref())
    }
}

impl StrictEncode for Variants {
    fn strict_encode_dumb() -> Self {
        variants! { 0..=5 }
    }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_type(Some("Variants"), self.deref())
    }
}

impl<Ref: TypeRef> StrictEncode for Ty<Ref> {
    fn strict_encode_dumb() -> Self { Ty::UnicodeChar }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        let u = writer
            .define_union(Some("Ty"))
            .define_type::<u8>("primitive")
            .define_unit("unicode")
            .define_type::<Variants>("enum")
            .define_type::<Fields<Ref, false>>("union")
            .define_type::<Fields<Ref, true>>("struct");
        let u = u.define_tuple("array").define_field::<Ref>().define_field::<u16>().complete();
        let u = u.define_tuple("list").define_field::<Ref>().define_field::<Sizing>().complete();
        let u = u.define_tuple("Set").define_field::<Ref>().define_field::<Sizing>().complete();
        let u = u
            .define_tuple("map")
            .define_field::<KeyTy>()
            .define_field::<Ref>()
            .define_field::<Sizing>()
            .complete();

        let u = u.complete();

        let u = match self {
            Ty::Primitive(prim) => u.write_type("primitive", &prim.into_code())?,
            Ty::UnicodeChar => u.write_unit("unicode")?,
            Ty::Enum(vars) => u.write_type("enum", vars)?,
            Ty::Union(fields) => u.write_type("union", fields)?,
            Ty::Struct(fields) => u.write_type("struct", fields)?,
            Ty::Array(ty, len) => {
                u.write_tuple("array")?.write_field(ty)?.write_field(len)?.complete()
            }
            Ty::List(ty, sizing) => {
                u.write_tuple("list")?.write_field(ty)?.write_field(sizing)?.complete()
            }
            Ty::Set(ty, sizing) => {
                u.write_tuple("set")?.write_field(ty)?.write_field(sizing)?.complete()
            }
            Ty::Map(key, ty, sizing) => u
                .write_tuple("map")?
                .write_field(key)?
                .write_field(ty)?
                .write_field(sizing)?
                .complete(),
        };

        Ok(u.complete())
    }
}

impl StrictEncode for KeyTy {
    fn strict_encode_dumb() -> Self { KeyTy::Array(767) }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        let u = writer
            .define_union(Some("KeyTy"))
            .define_type::<u8>("primitive")
            .define_type::<Variants>("enum")
            .define_type::<u16>("array")
            .define_type::<Sizing>("unicode")
            .define_type::<Sizing>("ascii")
            .define_type::<Sizing>("bytes")
            .complete();

        let u = match self {
            KeyTy::Primitive(prim) => u.write_type("primitive", &prim.into_code())?,
            KeyTy::Enum(vars) => u.write_type("enum", vars)?,
            KeyTy::Array(len) => u.write_type("array", len)?,
            KeyTy::UnicodeStr(sizing) => u.write_type("unicode", sizing)?,
            KeyTy::AsciiStr(sizing) => u.write_type("ascii", sizing)?,
            KeyTy::Bytes(sizing) => u.write_type("bytes", sizing)?,
        };

        Ok(u.complete())
    }
}

impl StrictEncode for Ident {
    fn strict_encode_dumb() -> Self { Ident::from("Dumb") }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_type(Some("Ident"), Wrapper::as_inner(self))
    }
}
