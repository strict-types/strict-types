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
extern crate strict_encoding;

use strict_encoding::STRICT_TYPES_LIB;
use strict_types::stl::std_stl;
use strict_types::typesys::SystemBuilder;
use strict_types::{LibBuilder, SymbolicSys, TypeLib};

fn lib() -> TypeLib {
    let std = std_stl();
    let builder =
        LibBuilder::new(libname!(STRICT_TYPES_LIB), [std.to_dependency()]).transpile::<TypeLib>();
    builder.compile().unwrap()
}

fn sys() -> SymbolicSys {
    let std = std_stl();
    let lib = lib();
    let builder = SystemBuilder::new().import(lib).unwrap().import(std).unwrap();
    match builder.finalize() {
        Ok(sys) => sys,
        Err(errors) => {
            for err in errors {
                eprintln!("Error: {err}");
            }
            panic!()
        }
    }
}

#[test]
fn library() {
    let lib = lib();
    println!("{lib}");
    println!("{lib:X}");
}

#[test]
fn symbols() {
    let sys = sys();
    println!("{sys}");
    println!("{sys:X}");
}

#[test]
fn type_tree() {
    use std::io::{stdout, Write};

    let sys = sys();
    let tt = sys.type_tree("StrictTypes.TypeLib").unwrap();
    let mut f = stdout();
    let mut counter = 0;
    for (depth, ty, fqn) in tt {
        write!(f, "{: ^1$}", "", depth * 2).ok();
        if let Some(fqn) = fqn {
            write!(f, "{fqn: <22}: ").ok();
        } else {
            write!(f, "{: >22}: ", "_").ok();
        }
        writeln!(f, "{ty}").ok();

        counter += 1;
        if counter > 10 {
            break;
        }
    }
}
