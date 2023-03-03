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

//! Strict value core types.

use amplify::num::{i1024, u1024};
use indexmap::IndexMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
#[non_exhaustive]
pub enum StrictNum {
    Uint(u128),
    BigUint(u1024),
    Int(i128),
    BitInt(i1024),
    // float
    // non-zero
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(inner)]
pub enum EnumTag {
    Name(String),
    Ord(u8),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum StrictVal {
    Unit,
    Number(StrictNum),
    String(String),
    Tuple(Vec<StrictVal>),
    Struct(IndexMap<String, StrictVal>),
    Enum(EnumTag),
    Union(EnumTag, Box<StrictVal>),
    List(Vec<StrictVal>),
    Table(IndexMap<String, StrictVal>),
}
