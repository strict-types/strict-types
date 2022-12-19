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

use stens::dtl::{SystemBuilder, TypeLib};
use stens::{Serialize, StenSchema, Urn};

#[test]
fn reflect() {
    let lib = TypeLib::with(s!("TypeLib"), TypeLib::sten_type()).unwrap();

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
