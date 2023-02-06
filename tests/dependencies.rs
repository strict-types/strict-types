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

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
extern crate strict_types;

use strict_encoding::{libname, StrictDumb, STRICT_TYPES_LIB};
use strict_types::typelib::build::LibBuilder;
use strict_types::{Dependency, KeyTy, TypeLib};

const LIB: &str = "Test";

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB, tags = repr, into_u8, try_from_u8)]
#[repr(u8)]
pub enum Prim {
    #[default]
    A = 1,
    B = 2,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB, tags = order)]
pub enum Message {
    Init(u8),
    #[default]
    Ping,
    Pong,
    Connect {
        host: Option<u8>,
    },
    Dependency(KeyTy),
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB)]
pub struct TypeA(u8, u16);

#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB)]
pub struct TypeB {
    pub one: TypeA,
    pub two: TypeA,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB)]
pub struct Complex {
    pub a: TypeA,
    pub b: TypeB,
    pub prim: Prim,
    pub msg: Message,
}

#[test]
fn serialize() {
    let root = TypeLib::strict_dumb();
    let builder = LibBuilder::new(libname!(STRICT_TYPES_LIB)).process(&root).unwrap();
    let lib = builder.compile(none!()).unwrap();
    let id = lib.id();

    let root = Complex::strict_dumb();
    let imports = bmap! {
        libname!(STRICT_TYPES_LIB) => (libname!(STRICT_TYPES_LIB), Dependency::with(id, libname!(STRICT_TYPES_LIB), (0,1,0)))
    };
    let builder = LibBuilder::new(libname!(LIB)).process(&root).unwrap();
    let lib = builder.compile(imports).unwrap();

    println!("{}", lib);
}
