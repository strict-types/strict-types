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

use amplify::confinement::TinyVec;
use encoding::{Primitive, Sizing};

#[derive(Clone, PartialEq, Eq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub enum MemTy {
    #[strict_type(tag = 0)]
    #[from]
    Primitive(Primitive),

    /// We use separate type since unlike primitive it has variable length.
    /// While unicode character can be expressed as a composite type, it will be very verbose
    /// expression (union with 256 variants), so instead we built it in.
    #[strict_type(tag = 1, rename = "unicode", dumb)]
    UnicodeChar,

    #[strict_type(tag = 3)]
    #[from]
    Enum(EnumMemTy),

    #[strict_type(tag = 4)]
    #[from]
    Union(Vec<(u8, MemTy)>),

    #[strict_type(tag = 7)]
    Array(MemTy, u16),

    #[strict_type(tag = 8)]
    List(MemTy, Sizing),

    #[strict_type(tag = 9)]
    Set(MemTy, Sizing),

    #[strict_type(tag = 10)]
    Map(KeyMemTy, MemTy, Sizing),
}

/// Lexicographically sortable types which may serve as map keys.
///
/// The type is always guaranteed to fit strict encoding AST serialization
/// bounds since it doesn't has a dynamically-sized types.
#[derive(Clone, PartialEq, Eq, Debug, Display, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom, dumb = { KeyTy::Array(1) })]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[display(inner)]
pub enum KeyMemTy {
    #[strict_type(tag = 0)]
    #[from]
    Primitive(Primitive),

    #[strict_type(tag = 3)]
    #[display("({0})")]
    #[from]
    Enum(EnumMemTy),

    /// Fixed-size byte array
    #[strict_type(tag = 7)]
    #[display("[Byte ^ {0}]")]
    #[from]
    Array(u16),

    #[strict_type(tag = 0x10, rename = "unicode")]
    #[display("[Unicode{0}]")]
    UnicodeStr(Sizing),

    #[strict_type(tag = 0x11, rename = "ascii")]
    #[display("[Ascii{0}]")]
    AsciiStr(Sizing),

    #[strict_type(tag = 0x12)]
    #[display("[Byte{0}]")]
    Bytes(Sizing),
}

pub struct EnumMemTy(TinyVec<u8>);
pub struct UnionMemTy(TinyVec<(u8, Box<MemTy>)>);
