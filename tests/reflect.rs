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

use baid58::ToBaid58;
use strict_encoding::{StrictSerialize, STRICT_TYPES_LIB};
use strict_types::typelib::build::LibBuilder;
use strict_types::TypeLib;

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
    let builder = LibBuilder::new(libname!(STRICT_TYPES_LIB)).process::<TypeLib>().unwrap();
    let lib = builder.compile(none!()).unwrap();
    let id = lib.id();

    println!("{{-\n-- Import this lib by putting in the file header\n-- import {id:#}\n-}}");
    println!();
    println!("{lib}");

    println!("----- BEGIN STRICT TYPE LIB -----");
    println!("Id: {}", id);
    println!("Checksum: {}", id.to_baid58().mnemonic());
    println!();
    pp(lib.to_strict_serialized::<{ u16::MAX as usize }>().expect("in-memory"));
    println!("\n----- END STRICT TYPE LIB -----\n");

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
