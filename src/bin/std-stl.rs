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

use std::io::stdout;
use std::{env, fs, io};

use amplify::num::u24;
use strict_encoding::{StrictEncode, StrictWriter, LIB_NAME_STD};
use strict_types::stl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let lib = stl::std_stl();
    let id = lib.id();

    let ext = match args.get(1).map(String::as_str) {
        Some("--stl") => "stl",
        Some("--asc") => "asc.stl",
        Some("--sty") => "sty",
        _ => "sty",
    };
    let filename =
        args.get(2).cloned().unwrap_or_else(|| format!("stl/{LIB_NAME_STD}@0.1.0.{ext}"));
    let mut file = match args.len() {
        1 => Box::new(stdout()) as Box<dyn io::Write>,
        2 | 3 => Box::new(fs::File::create(filename)?) as Box<dyn io::Write>,
        _ => panic!("invalid argument count"),
    };
    match ext {
        "stl" => {
            lib.strict_encode(StrictWriter::with(u24::MAX.into_usize(), file))?;
        }
        "asc.stl" => {
            writeln!(file, "{lib:X}")?;
        }
        _ => {
            writeln!(
                file,
                "{{-
  Id: {id:+}
  Name: StrictTypes
  Description: Confined generalized algebraic data types (GADT)
  Author: Dr Maxim Orlovsky <orlovsky@ubideco.org>
  Copyright (C) 2023 UBIDECO Institute. All rights reserved.
  License: Apache-2.0
-}}
"
            )?;
            writeln!(file, "{lib}")?;
        }
    }

    Ok(())
}
