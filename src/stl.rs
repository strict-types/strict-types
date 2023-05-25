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

use encoding::stl::{
    Alpha, AlphaCaps, AlphaCapsNum, AlphaNum, AlphaNumDash, AlphaNumLodash, AlphaSmall,
    AsciiPrintable, Bool, Dec, HexDecCaps, HexDecSmall, U4,
};
use encoding::{LIB_NAME_STD, STRICT_TYPES_LIB};

use crate::typeobj::{LibBuilder, SymbolRef};
use crate::typesys::{TypeSymbol, TypeSysId};
use crate::{CompileError, TranspileError, TypeLib, TypeObjects, TypeSystem};

pub const LIB_ID_STD: &str = "regard_light_ninja_9fsgNpJdXxPiJztBbJVZQKcRbxF8DtpBcYQduxUgXKps";
pub const LIB_ID_STRICT_TYPES: &str =
    "darwin_popular_welcome_4fFBHxWbs1ysSL59Nc4xFExBVAucfKbTvujACbZy8ATc";

fn _std_sym() -> Result<TypeObjects, TranspileError> {
    LibBuilder::new(libname!(LIB_NAME_STD), none!())
        .transpile::<Bool>()
        .transpile::<U4>()
        .transpile::<AsciiPrintable>()
        .transpile::<Alpha>()
        .transpile::<AlphaCaps>()
        .transpile::<AlphaSmall>()
        .transpile::<Dec>()
        .transpile::<HexDecCaps>()
        .transpile::<HexDecSmall>()
        .transpile::<AlphaNum>()
        .transpile::<AlphaCapsNum>()
        .transpile::<AlphaNumDash>()
        .transpile::<AlphaNumLodash>()
        .compile_symbols()
}

fn _std_stl() -> Result<TypeLib, CompileError> { _std_sym()?.compile() }

pub fn std_sym() -> TypeObjects { _std_sym().expect("invalid strict type Std library") }

pub fn std_stl() -> TypeLib { _std_stl().expect("invalid strict type Std library") }

fn _strict_types_sym() -> Result<TypeObjects, TranspileError> {
    LibBuilder::new(libname!(STRICT_TYPES_LIB), none!())
        .transpile::<SymbolRef>()
        .transpile::<TypeLib>()
        .transpile::<TypeSystem>()
        .transpile::<TypeSysId>()
        .transpile::<TypeSymbol>()
        .compile_symbols()
}
fn _strict_types_stl() -> Result<TypeLib, CompileError> { _strict_types_sym()?.compile() }

pub fn strict_types_sym() -> TypeObjects {
    _strict_types_sym().expect("invalid strict type StrictTypes library")
}

pub fn strict_types_stl() -> TypeLib {
    _strict_types_stl().expect("invalid strict type StrictTypes library")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn std_lib_id() {
        let lib = std_stl();
        assert_eq!(lib.id().to_string(), LIB_ID_STD);
    }

    #[test]
    fn strict_types_lib_id() {
        let lib = strict_types_stl();
        assert_eq!(lib.id().to_string(), LIB_ID_STRICT_TYPES);
    }
}
