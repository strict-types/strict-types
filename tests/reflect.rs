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

#[macro_use]
extern crate amplify;

use amplify::hex::ToHex;
use stens::encoding::{StrictEncode, StrictWriter};
use stens::typelib::TypeLib;
use stens::{tn, LibRef, Ty, Urn};

fn pp(data: impl AsRef<[u8]>) {
    let data = base64::encode(data);
    let mut data = data.as_str();
    while data.len() > 80 {
        let (line, rest) = data.split_at(80);
        println!("{}", line);
        data = rest;
    }
    println!("{}", data);
}

#[test]
fn reflect() {
    let ty = Ty::<LibRef>::UnicodeChar;
    let writer = StrictWriter::in_memory(u16::MAX as usize);
    let writer = ty.strict_encode(writer).expect("memory encoding");
    println!("----- BEGIN STEN TYPE -----");
    println!("Id: {}\n", ty.id(Some(&tn!("Ty"))));
    println!("{}", writer.unbox().to_hex());
    println!("\n----- END STEN TYPE -----\n");

    /*
    let root = TypeLib::sten_type();
    let root_id = root.id();
    let lib = TypeLib::with(s!("StEn"), root).unwrap();

    println!("{:#}", Urn::from(lib.id()));
    println!("{:#}", Urn::from(root_id));

    println!();
    println!("{}", lib);
    println!("----- BEGIN STEN TYPE LIB -----");
    println!("Id: {}\n", lib.id());
    pp(lib.to_serialized());
    println!("\n----- END STEN TYPE LIB -----\n");
    */
    /*
    let mut builder = SystemBuilder::new();
    builder.import(lib);
    match builder.finalize() {
        Ok((sys, warnings)) => {
            for warning in warnings {
                eprintln!("Warning: {}", warning);
            }
            println!("----- BEGIN STEN TYPE SYSTEM -----");
            println!("Id: {}\n", sys.id());
            pp(sys.to_serialized());
            println!("\n----- END STEN TYPE SYSTEM -----\n");
        }
        Err(errors) => {
            for error in errors {
                eprintln!("Error: {}", error);
            }
            panic!()
        }
    }

     */
}
