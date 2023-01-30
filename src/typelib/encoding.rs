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

use std::io;

use strict_encoding::{
    DecodeError, ReadTuple, StrictDecode, StrictDeserialize, StrictEncode, StrictProduct,
    StrictSerialize, StrictTuple, StrictType, TypedRead, TypedWrite, STEN_LIB,
};

use crate::{TypeLib, TypeLibId};

impl StrictType for TypeLibId {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictProduct for TypeLibId {}
impl StrictTuple for TypeLibId {
    const FIELD_COUNT: u8 = 1;
}
impl StrictEncode for TypeLibId {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_newtype::<Self>(self.as_bytes())
    }
}
impl StrictDecode for TypeLibId {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_tuple(|r| r.read_field::<[u8; 32]>().map(TypeLibId::from))
    }
}

impl StrictSerialize for TypeLib {}
impl StrictDeserialize for TypeLib {}
