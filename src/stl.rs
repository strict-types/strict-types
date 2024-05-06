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

use encoding::stl::{
    Alpha, AlphaCaps, AlphaCapsLodash, AlphaCapsNum, AlphaLodash, AlphaNum, AlphaNumDash,
    AlphaNumLodash, AlphaSmall, AlphaSmallLodash, AsciiPrintable, AsciiSym, Bool, Dec, HexDecCaps,
    HexDecSmall, U2, U3, U4, U6, U7,
};
use encoding::{
    FieldName, Ident, LibName, TypeName, VariantName, LIB_NAME_STD, STRICT_TYPES_LIB, U1, U5,
};

use crate::layout::MemoryLayout;
use crate::{
    CompileError, LibBuilder, SymbolRef, SymbolicLib, SymbolicSys, TranspileError, TypeLib,
    TypeSymbol, TypeSysId,
};

pub const LIB_ID_STD: &str =
    "stl:yiweb4OZ-3TAMPm@-eUS$XRw-iMgF32K-DbZZJX5-xmwCVCc#ralph-blue-lucky";
pub const LIB_ID_STRICT_TYPES: &str =
    "stl:ReqjX9v2-45ABOvH-i7YYKfx-30V2lgT-owwpkNk-E$v5ENk#century-comrade-chess";

fn _std_sym() -> Result<SymbolicLib, TranspileError> {
    LibBuilder::new(libname!(LIB_NAME_STD), None)
        .transpile::<Bool>()
        .transpile::<U1>()
        .transpile::<U2>()
        .transpile::<U3>()
        .transpile::<U4>()
        .transpile::<U5>()
        .transpile::<U6>()
        .transpile::<U7>()
        .transpile::<AsciiSym>()
        .transpile::<AsciiPrintable>()
        .transpile::<Alpha>()
        .transpile::<AlphaLodash>()
        .transpile::<AlphaCaps>()
        .transpile::<AlphaCapsLodash>()
        .transpile::<AlphaSmall>()
        .transpile::<AlphaSmallLodash>()
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
    LibBuilder::new(libname!(STRICT_TYPES_LIB), [std_stl().to_dependency()])
        .transpile::<Ident>()
        .transpile::<TypeName>()
        .transpile::<FieldName>()
        .transpile::<VariantName>()
        .transpile::<LibName>()
        .transpile::<SymbolRef>()
        .transpile::<TypeLib>()
        .transpile::<TypeSysId>()
        .transpile::<TypeSymbol>()
        .transpile::<SymbolicSys>()
        .transpile::<MemoryLayout>()
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
