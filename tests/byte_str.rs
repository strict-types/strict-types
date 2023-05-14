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

use amplify::confinement::SmallVec;
use strict_types::typelib::LibBuilder;

#[test]
fn reflect() {
    #[derive(Clone, Debug, Default)]
    #[derive(StrictType, StrictEncode, StrictDecode)]
    #[strict_type(lib = "Test")]
    struct ByteStr(SmallVec<u8>);

    let builder = LibBuilder::new(libname!("Test")).transpile::<ByteStr>();
    let lib = builder.compile(none!()).unwrap();

    assert_eq!(
        lib.to_string(),
        "typelib Test -- break_senior_ventura_HRmiM5TfA2egrTigBRuxVX192TNqJYNvtk17hu1b1hyf

-- no dependencies

data ByteStr          :: [Byte]
"
    );
}
