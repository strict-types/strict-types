// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2024 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2024 UBIDECO Institute
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
extern crate strict_encoding;

use armor::AsciiArmor;
use strict_encoding::STRICT_TYPES_LIB;
use strict_types::stl::std_stl;
use strict_types::typesys::SystemBuilder;
use strict_types::{LibBuilder, SymbolicSys, TypeLib};

fn lib() -> TypeLib {
    let std = std_stl();
    let builder = LibBuilder::with(libname!(STRICT_TYPES_LIB), [std.to_dependency_types()])
        .transpile::<TypeLib>();
    builder.compile().unwrap()
}

fn sys() -> SymbolicSys {
    let std = std_stl();
    let lib = lib();
    let builder = SystemBuilder::new().import(lib).unwrap().import(std).unwrap();
    builder.finalize().unwrap_or_else(|errors| {
        for err in errors {
            eprintln!("Error: {err}");
        }
        panic!()
    })
}

#[test]
fn library() {
    let lib = lib();
    println!("{lib}");

    let s = lib.to_ascii_armored_string();
    println!("{s}");
    let lib2 = TypeLib::from_ascii_armored_str(&s).unwrap();
    assert_eq!(lib, lib2);
}

#[test]
fn symbols() {
    let sys = sys();
    println!("{sys}");

    let s = sys.to_ascii_armored_string();
    println!("{s}");
    let sys2 = SymbolicSys::from_ascii_armored_str(&s).unwrap();
    assert_eq!(sys, sys2);
}

#[test]
fn type_tree() {
    let sys = sys();
    let tt = sys.type_tree("StrictTypes.TypeLib").unwrap();
    let _ = tt.to_string();
}
