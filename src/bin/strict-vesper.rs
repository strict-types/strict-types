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

use std::fs::File;
use std::io::Write;

use strict_encoding::{libname, STRICT_TYPES_LIB};
use strict_types::stl::std_stl;
use strict_types::{LibBuilder, SystemBuilder, TypeLib};

fn main() {
    let std = std_stl();
    let builder =
        LibBuilder::new(libname!(STRICT_TYPES_LIB), [std.to_dependency()]).transpile::<TypeLib>();
    let lib = builder.compile().unwrap();
    let builder = SystemBuilder::new().import(lib).unwrap().import(std).unwrap();
    let sys = builder.finalize().unwrap_or_else(|errors| {
        for err in errors {
            eprintln!("Error: {err}");
        }
        panic!()
    });

    let tt = sys.type_tree("StrictTypes.TypeLib").unwrap();

    let mut file = File::create("stl/TypeLib.vesper").expect("unable to create file");
    writeln!(
        file,
        "{{-
  Description: Data type layout in Vesper language
  Author: Dr Maxim Orlovsky <orlovsky@ubideco.org>
  Copyright (C) 2024 UBIDECO Institute. All rights reserved.
  License: Apache-2.0
-}}

{} vesper lexicon=types
",
        STRICT_TYPES_LIB
    )
    .unwrap();
    write!(file, "{tt}").unwrap();
}
