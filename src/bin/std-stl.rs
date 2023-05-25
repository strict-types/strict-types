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

use strict_types::{parse_args, stl, StlFormat};

fn main() {
    let (format, dir) = parse_args();
    match format {
        StlFormat::Source => {
            stl::std_sym()
                .serialize(
                    format,
                    dir,
                    "0.1.0",
                    Some(
                        "
  Description: Strict types standard library
  Author: Dr Maxim Orlovsky <orlovsky@ubideco.org>
  Copyright (C) 2023 UBIDECO Institute. All rights reserved.
  License: Apache-2.0",
                    ),
                )
                .expect("unable to write to the file");
        }
        StlFormat::Binary | StlFormat::Armored => {
            stl::std_stl()
                .serialize(format, dir, "0.1.0", None)
                .expect("unable to write to the file");
        }
    }
}
