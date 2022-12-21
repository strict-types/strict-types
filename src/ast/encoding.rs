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

use crate::ast::Step;
use crate::encoding::{StrictEncode, TypedWrite, WriteUnion};
use crate::{FieldName, Ident};

impl StrictEncode for Step {
    fn strict_encode_dumb() -> Self { Step::Index }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        let u = writer
            .write_union("StEn", Some("Step"))
            .define_type::<FieldName>("NamedField")
            .define_type::<u8>("UnnamedField")
            .define_unit("Index")
            .define_unit("List")
            .define_unit("Set")
            .define_unit("Map");

        let u = match self {
            Step::NamedField(name) => u.write_type("NamedField", name),
            Step::UnnamedField(ord) => u.write_type("UnnamedField", ord),
            Step::Index => u.write_unit("Index"),
            Step::List => u.write_unit("List"),
            Step::Set => u.write_unit("Set"),
            Step::Map => u.write_unit("Map"),
        }?;

        Ok(u.complete())
    }
}

impl StrictEncode for Ident {
    fn strict_encode_dumb() -> Self { Ident::from("Dumb") }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_type("StEn", Some("Ident"), self.as_inner())
    }
}

impl StrictEncode for u8 {
    fn strict_encode_dumb() -> Self { 0 }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        unsafe { writer.write_raw([*self]) }
    }
}
