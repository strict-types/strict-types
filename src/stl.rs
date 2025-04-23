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

use encoding::stl::{
    Alpha, AlphaCaps, AlphaCapsDash, AlphaCapsDot, AlphaCapsLodash, AlphaCapsNum, AlphaDash,
    AlphaDot, AlphaLodash, AlphaNum, AlphaNumDash, AlphaNumDot, AlphaNumLodash, AlphaSmall,
    AlphaSmallDash, AlphaSmallDot, AlphaSmallLodash, AsciiPrintable, AsciiSym, Bool, Dec, DecDot,
    HexDecCaps, HexDecSmall, U2, U3, U4, U6, U7,
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
    "stl:gonrTQ8L-cFSvdEs-F6MHXnS-MDplxjy-8_lZ5j5-_lY8MWo#delete-roman-hair";
pub const LIB_ID_STRICT_TYPES: &str =
    "stl:6Z6S5ztA-l3_RfoW-uOIW~K0-04t7R_3-KIiByhE-1W4rPFA#henry-heart-survive";

fn _std_sym() -> Result<SymbolicLib, TranspileError> {
    LibBuilder::with(libname!(LIB_NAME_STD), None)
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
        .transpile::<AlphaDot>()
        .transpile::<AlphaDash>()
        .transpile::<AlphaLodash>()
        .transpile::<AlphaCaps>()
        .transpile::<AlphaCapsDot>()
        .transpile::<AlphaCapsDash>()
        .transpile::<AlphaCapsLodash>()
        .transpile::<AlphaSmall>()
        .transpile::<AlphaSmallDot>()
        .transpile::<AlphaSmallDash>()
        .transpile::<AlphaSmallLodash>()
        .transpile::<Dec>()
        .transpile::<DecDot>()
        .transpile::<HexDecCaps>()
        .transpile::<HexDecSmall>()
        .transpile::<AlphaNum>()
        .transpile::<AlphaCapsNum>()
        .transpile::<AlphaNumDot>()
        .transpile::<AlphaNumDash>()
        .transpile::<AlphaNumLodash>()
        .compile_symbols()
}

fn _std_stl() -> Result<TypeLib, CompileError> { _std_sym()?.compile() }

pub fn std_sym() -> SymbolicLib { _std_sym().expect("invalid strict type Std library") }

pub fn std_stl() -> TypeLib { _std_stl().expect("invalid strict type Std library") }

fn _strict_types_sym() -> Result<SymbolicLib, TranspileError> {
    LibBuilder::with(libname!(STRICT_TYPES_LIB), [std_stl().to_dependency_types()])
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
