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

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_types;

use std::io;

use strict_encoding::stl::{AlphaLodash, Bool};
use strict_encoding::{
    DecodeError, Ident, StrictDecode, StrictDumb, StrictEncode, StrictType, TypedRead, TypedWrite,
    LIB_NAME_STD, STRICT_TYPES_LIB,
};
use strict_types::stl::{std_stl, strict_types_stl};
use strict_types::{CompileError, LibBuilder, SystemBuilder, TranspileError, TypeLib};

const LIB: &str = "Test";

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Void;

impl StrictType for Void {
    const STRICT_LIB_NAME: &'static str = LIB;
}
impl StrictEncode for Void {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> { Ok(writer) }
}
impl StrictDecode for Void {
    fn strict_decode(_reader: &mut impl TypedRead) -> Result<Self, DecodeError> { Ok(Void) }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB, tags = repr, into_u8, try_from_u8)]
#[repr(u8)]
pub enum Prim {
    #[default]
    A = 1,
    B = 2,
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB, tags = order)]
pub enum Message {
    Init(u8),
    Ping,
    Pong { len: u8, nonce: Void },
    Connect { host: Option<u8>, port: u16 },
}

impl Default for Message {
    fn default() -> Self {
        Message::Pong {
            len: 0,
            nonce: Void,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB)]
pub struct TypeA(u8, u16);

#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB)]
pub struct TypeB {
    pub one: TypeA,
    pub two: TypeA,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB)]
pub struct Complex {
    pub a: TypeA,
    pub b: TypeB,
    pub prim: Prim,
    pub msg: Message,
}

#[test]
fn serialize() {
    let std = std_stl();
    let builder = LibBuilder::with(libname!(STRICT_TYPES_LIB), [std.to_dependency_types()])
        .transpile::<TypeLib>();
    let lib = builder.compile().unwrap();

    let builder =
        LibBuilder::with(libname!(LIB), [lib.to_dependency_types()]).transpile::<Complex>();
    let lib = builder.compile_symbols().unwrap();

    println!("{}", lib);
}

#[test]
fn dependency_misses_type() {
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
    #[derive(StrictType, StrictEncode, StrictDecode)]
    #[strict_type(lib = LIB_NAME_STD)]
    struct Fake(());

    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
    #[derive(StrictType, StrictEncode, StrictDecode)]
    #[strict_type(lib = LIB)]
    struct FakeUser(Fake);

    let std = std_stl();
    let builder =
        LibBuilder::with(libname!(LIB), [std.to_dependency_types()]).transpile::<FakeUser>();
    let err = builder.compile().unwrap_err();
    eprintln!("{err}");
    assert!(
        matches!(err, CompileError::DependencyMissesType(lib, _, name) if lib == libname!(LIB_NAME_STD) && name == tn!("Fake"))
    );

    let builder =
        LibBuilder::with(libname!(LIB), [std.to_dependency_types()]).transpile::<FakeUser>();
    let err = builder.compile_symbols().unwrap_err();
    assert!(
        matches!(err, TranspileError::DependencyMissesType(lib, _, name) if lib == libname!(LIB_NAME_STD) && name == tn!("Fake"))
    );
}

#[test]
#[should_panic(
    expected = r#"DependencyMissesType(LibName("Std"), SemId(Array<32>(831bcb0c328608f3f9cd16633c16a8e6a52ac31c79a61042be9d864bc9f4a0f7)), TypeName("AlphaLodash"))"#
)]
fn type_lib_missing_type() {
    // construct the original SymbolicSys
    let libs_orig = [strict_types_stl(), std_stl()];
    let mut builder_orig = SystemBuilder::new();
    for lib in libs_orig.into_iter() {
        builder_orig = builder_orig.import(lib).unwrap();
    }
    let sys_orig = builder_orig.finalize().unwrap();

    // construct a copy of std_stl that uses only Bool
    let std_stl_mod = LibBuilder::with(libname!(LIB_NAME_STD), None)
        .transpile::<Bool>()
        .compile_symbols()
        .unwrap()
        .compile()
        .unwrap();

    // construct a library that uses the original Ident but depends on the modified std_stl
    let strict_types_stl_mod =
        LibBuilder::with(libname!(STRICT_TYPES_LIB), [std_stl_mod.to_dependency_types()])
            .transpile::<Ident>()
            .compile_symbols()
            .unwrap()
            .compile()
            .unwrap();

    // construct a modified SymbolicSys
    let libs_mod = [strict_types_stl_mod, std_stl_mod];
    let mut builder_mod = SystemBuilder::new();
    for lib in libs_mod.into_iter() {
        builder_mod = builder_mod.import(lib).unwrap();
    }
    let sys_mod = builder_mod.finalize().unwrap();

    // get the sem ID for AlphaLodash from the modified SymbolicSys
    let alphalodash_semid_orig = sys_orig.resolve("Std.AlphaLodash").unwrap();

    // getting the AlphaLodash type from the original TypeSystem succeeds, as expected
    sys_orig.clone().into_type_system().find(*alphalodash_semid_orig).unwrap();
    // getting the AlphaLodash type from the modified TypeSystem fails
    sys_mod.clone().into_type_system().find(*alphalodash_semid_orig).unwrap();
}

#[test]
#[should_panic(
    expected = r#"DependencyMissesType(LibName("Std"), SemId(Array<32>(95c3bdc94d0260f9716a113cf6492d5d4e23988e33043005ca36da6d6eee67b4)), TypeName("AlphaNumLodash"))"#
)]
fn type_lib_semid_inconsistency() {
    // construct the original SymbolicSys
    let libs_orig = [strict_types_stl(), std_stl()];
    let mut builder_orig = SystemBuilder::new();
    for lib in libs_orig.into_iter() {
        builder_orig = builder_orig.import(lib).unwrap();
    }
    let sys_orig = builder_orig.finalize().unwrap();

    // define a modified AlphaNumLodash
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
    #[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
    #[strict_type(lib = LIB_NAME_STD, tags = repr, into_u8, try_from_u8)]
    #[display(inner)]
    #[repr(u8)]
    pub enum AlphaNumLodash {
        #[strict_type(dumb)]
        #[display("0")]
        Zero = b'0',
    }

    // construct a copy of std_stl that uses the modified AlphaNumLodash
    let std_stl_mod = LibBuilder::with(libname!(LIB_NAME_STD), None)
        .transpile::<AlphaLodash>()
        .transpile::<AlphaNumLodash>()
        .compile_symbols()
        .unwrap()
        .compile()
        .unwrap();

    // construct a library that uses the original Ident but depends on the modified std_stl
    let strict_types_stl_mod =
        LibBuilder::with(libname!(STRICT_TYPES_LIB), [std_stl_mod.to_dependency_types()])
            .transpile::<Ident>()
            .compile_symbols()
            .unwrap()
            .compile()
            .unwrap();

    // construct a modified SymbolicSys
    let libs_mod = [strict_types_stl_mod, std_stl_mod];
    let mut builder_mod = SystemBuilder::new();
    for lib in libs_mod.into_iter() {
        builder_mod = builder_mod.import(lib).unwrap();
    }
    let sys_mod = builder_mod.finalize().unwrap();

    // get the sem IDs for Ident and related types from the original SymbolicSys
    let ident_semid_orig = sys_orig.resolve("StrictTypes.Ident").unwrap();
    let alphalodash_semid_orig = sys_orig.resolve("Std.AlphaLodash").unwrap();
    let alphanumlodash_semid_orig = sys_orig.resolve("Std.AlphaNumLodash").unwrap();

    // get the sem IDs for Ident and related types from the modified SymbolicSys
    let ident_semid_mod = sys_mod.resolve("StrictTypes.Ident").unwrap();
    let alphalodash_semid_mod = sys_mod.resolve("Std.AlphaLodash").unwrap();
    let alphanumlodash_semid_mod = sys_mod.resolve("Std.AlphaNumLodash").unwrap();

    // check that Ident and related types are available in the modified TypeSystem
    sys_mod.clone().into_type_system().find(*ident_semid_mod).unwrap();
    sys_mod.clone().into_type_system().find(*alphalodash_semid_mod).unwrap();
    sys_mod.clone().into_type_system().find(*alphanumlodash_semid_mod).unwrap();

    // AlphaLodash sem IDs from the 2 SymbolicSys match, as expected
    assert_eq!(alphalodash_semid_orig, alphalodash_semid_mod);
    // AlphaNumLodash sem IDs from the 2 SymbolicSys differ, as expected
    assert_ne!(alphanumlodash_semid_orig, alphanumlodash_semid_mod);
    // Ident sem IDs from the 2 SymbolicSys unexpectedly match
    assert_ne!(ident_semid_orig, ident_semid_mod); // fails
}
