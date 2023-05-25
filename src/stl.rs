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

use crate::{
    CompileError, LibBuilder, SymbolRef, SymbolicLib, SymbolicSys, TranspileError, TypeLib,
    TypeSymbol, TypeSysId,
};

pub const LIB_ID_STD: &str = "left_pierre_food_5cmoZctpx98FbTzWTArm3G53pMzUXuRPXmMhuCd8zyXb";
pub const LIB_ID_STRICT_TYPES: &str =
    "figure_radical_liquid_8pj6Q4MDsaQW97omrv5DhAX1yid5Hjmbi8uoBzLQayMh";

fn _std_sym() -> Result<SymbolicLib, TranspileError> {
    LibBuilder::new(libname!(LIB_NAME_STD), None)
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

pub fn std_sym() -> SymbolicLib { _std_sym().expect("invalid strict type Std library") }

pub fn std_stl() -> TypeLib { _std_stl().expect("invalid strict type Std library") }

fn _strict_types_sym() -> Result<SymbolicLib, TranspileError> {
    LibBuilder::new(libname!(STRICT_TYPES_LIB), None)
        .transpile::<SymbolRef>()
        .transpile::<TypeLib>()
        .transpile::<TypeSysId>()
        .transpile::<TypeSymbol>()
        .transpile::<SymbolicSys>()
        .compile_symbols()
}
fn _strict_types_stl() -> Result<TypeLib, CompileError> { _strict_types_sym()?.compile() }

pub fn strict_types_sym() -> SymbolicLib {
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
