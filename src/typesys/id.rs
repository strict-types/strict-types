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

use std::str::FromStr;

use amplify::{Bytes32, RawArray};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};
use strict_encoding::STRICT_TYPES_LIB;

use crate::TypeSystem;

pub const TYPESYS_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:sys:v01";

#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[display(Self::to_baid58)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct TypeSysId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl ToBaid58<32> for TypeSysId {
    const HRI: &'static str = "sts";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_raw_array() }
}
impl FromBaid58<32> for TypeSysId {}
impl FromStr for TypeSysId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("sts") {
            Self::from_baid58_str(s)
        } else {
            Self::from_baid58_str(&format!("sts:{s}"))
        }
    }
}

impl TypeSystem {
    pub fn id(&self) -> TypeSysId {
        let mut hasher = blake3::Hasher::new_keyed(&TYPESYS_ID_TAG);
        for ty in self.values() {
            hasher.update(ty.id(None).as_slice());
        }
        TypeSysId::from_raw_array(hasher.finalize())
    }
}
