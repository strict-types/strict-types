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
extern crate strict_types;

use std::io;

use strict_encoding::{
    DecodeError, StrictDecode, StrictEncode, StrictType, TypedRead, TypedWrite, STRICT_TYPES_LIB,
};
use strict_types::typeobj::LibBuilder;
use strict_types::{Dependency, KeyTy, TypeLib};

const LIB: &str = "Test";

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Void;

impl StrictType for Void {
    const STRICT_LIB_NAME: &'static str = LIB;
}
impl StrictEncode for Void {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> { Ok(writer) }
}
impl StrictDecode for Void {
    fn strict_decode(_reader: &mut impl TypedRead) -> Result<Self, DecodeError> { Ok(Void) }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB, tags = repr, into_u8, try_from_u8)]
#[repr(u8)]
pub enum Prim {
    #[default]
    A = 1,
    B = 2,
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB, tags = order)]
pub enum Message {
    Init(u8),
    Ping,
    Pong { len: u8, nonce: Void },
    Connect { host: Option<u8>, port: u16 },
    Dependency(KeyTy),
}

impl Default for Message {
    fn default() -> Self {
        Message::Pong {
            len: 0,
            nonce: Void,
        }
    }
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
    let builder = LibBuilder::new(libname!(STRICT_TYPES_LIB), none!()).transpile::<TypeLib>();
    let lib = builder.compile().unwrap();

    let imports = tiny_bset! {
        Dependency::with(lib.id(), lib.name)
    };
    let builder = LibBuilder::new(libname!(LIB), imports).transpile::<Complex>();
    let lib = builder.compile().unwrap();

    println!("{}", lib);
}
