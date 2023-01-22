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

use amplify::confinement::SmallVec;

#[repr(u8)]
pub enum Prim {
    A = 1,
    B = 2,
}

pub enum Message {
    Init(u8),
    Ping,
    Pong,
    Connect { host: Option<SmallVec<u8>> },
}

pub struct TypeA(u8, u16);

pub struct TypeB {
    pub one: TypeA,
    pub two: TypeA,
}

pub struct Complex {
    pub a: TypeA,
    pub b: TypeB,
    pub prim: Prim,
    pub msg: Message,
}

#[test]
fn serialize() {
    /*
    let root = Complex::strict_dumb();

    let builder = LibBuilder::new(libname!("")).process(&root).unwrap();
    let lib = builder.compile().unwrap();

    println!("{}", lib);

    println!("----- BEGIN STRICT TYPE LIB -----");
    println!("Id: {:#}\n", lib.id());
    pp(lib.to_strict_serialized::<{ u16::MAX as usize }>().expect("in-memory"));
    println!("\n----- END STRICT TYPE LIB -----\n");
     */
}
