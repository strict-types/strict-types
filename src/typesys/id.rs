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

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use amplify::{ByteArray, Bytes32};
use baid64::{Baid64ParseError, DisplayBaid64, FromBaid64Str};
use encoding::StrictEncode;
use sha2::{Digest, Sha256};
use strict_encoding::STRICT_TYPES_LIB;

use crate::ast::SemCommit;
use crate::{CommitConsume, TypeSystem};

pub const TYPESYS_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:sys:v01";

#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct TypeSysId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl DisplayBaid64 for TypeSysId {
    const HRI: &'static str = "sts";
    const CHUNKING: bool = true;
    const PREFIX: bool = true;
    const EMBED_CHECKSUM: bool = false;
    const MNEMONIC: bool = true;
    fn to_baid64_payload(&self) -> [u8; 32] { self.to_byte_array() }
}
impl FromBaid64Str for TypeSysId {}
impl FromStr for TypeSysId {
    type Err = Baid64ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_baid64_str(s) }
}
impl Display for TypeSysId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { self.fmt_baid64(f) }
}

impl SemCommit for TypeSystem {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        hasher.commit_consume(self.len_u24().to_le_bytes());
        for sem_id in self.keys() {
            sem_id.sem_commit(hasher);
        }
    }
}

impl TypeSystem {
    pub fn id(&self) -> TypeSysId {
        let tag = Sha256::new_with_prefix(TYPESYS_ID_TAG).finalize();
        let mut hasher = Sha256::new();
        hasher.commit_consume(tag);
        hasher.commit_consume(tag);
        self.sem_commit(&mut hasher);
        TypeSysId::from_byte_array(hasher.finalize())
    }
}
