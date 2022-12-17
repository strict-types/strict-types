// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
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

#[macro_use]
extern crate stens;
#[macro_use]
extern crate amplify;

use amplify::confinement::SmallVec;
use stens::ast::Ty;
use stens::dtl::{SystemBuilder, TypeLib};
use stens::{Serialize, StenSchema, StenType, Urn};

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

impl StenSchema for Prim {
    const STEN_TYPE_NAME: &'static str = "Prim";

    fn sten_ty() -> Ty<StenType> {
        Ty::enumerate(variants![
            "a" => 10 => Prim::A as u8,
            "b" => 20 => Prim::B as u8,
        ])
    }
}

impl StenSchema for Message {
    const STEN_TYPE_NAME: &'static str = "Message";

    fn sten_ty() -> Ty<StenType> {
        Ty::union(fields! [
            "init" => u8::sten_type(),
            "ping" => <()>::sten_type(),
            "pong" => <()>::sten_type(),
            "connect" => StenType::new("", Ty::composition(fields![
                "host" => Option::<SmallVec<u8>>::sten_type(),
            ])),
        ])
    }
}

impl StenSchema for TypeA {
    const STEN_TYPE_NAME: &'static str = "TypeA";

    fn sten_ty() -> Ty<StenType> { Ty::composition(fields![u8::sten_type(), u16::sten_type(),]) }
}

impl StenSchema for TypeB {
    const STEN_TYPE_NAME: &'static str = "TypeB";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields![
            "one" => TypeA::sten_type(),
            "two" => TypeA::sten_type(),
        ])
    }
}

impl StenSchema for Complex {
    const STEN_TYPE_NAME: &'static str = "Complex";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields![
            "a" => TypeA::sten_type(),
            "b" => TypeB::sten_type(),
            "prim" => Prim::sten_type(),
            "msg" => Message::sten_type(),
        ])
    }
}

#[test]
fn serialize() {
    let root = Complex::sten_type();
    let lib = TypeLib::with(s!("Test"), root).unwrap();

    println!("{:#}", Urn::from(lib.id()));

    println!();
    println!("{}", lib);

    let mut builder = SystemBuilder::new();
    builder.import(lib);
    match builder.finalize() {
        Ok((sys, warnings)) => {
            for warning in warnings {
                eprintln!("Warning: {}", warning);
            }
            let data = sys.to_serialized();
            let data = base64::encode(data.as_inner());
            let mut data = data.as_str();
            println!("----- BEGIN STEN TYPE SYSTEM -----");
            println!("Id: {}\n", sys.id());
            while data.len() > 80 {
                let (line, rest) = data.split_at(80);
                println!("{}", line);
                data = rest;
            }
            println!("{}", data);
            println!("\n----- END STEN TYPE SYSTEM -----\n");
        }
        Err(errors) => {
            for error in errors {
                eprintln!("Error: {}", error);
            }
            panic!()
        }
    }
}
