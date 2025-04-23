// Strict encoding schema library, implementing validation and parsing of strict encoded data
// against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2019-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2024-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2022-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use strict_types::{parse_args, stl};

fn main() {
    let (format, dir) = parse_args();
    stl::strict_types_stl()
        .serialize(
            format,
            dir,
            "0.1.0",
            Some(
                "
  Description: Confined generalized algebraic data types (GADT)
  Author: Dr Maxim Orlovsky <orlovsky@ubideco.org>
  Copyright (C) 2023-2024 UBIDECO Institute. All rights reserved.
  License: Apache-2.0",
            ),
        )
        .expect("unable to write to the file");
}
