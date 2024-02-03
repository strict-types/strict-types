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

use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use amplify::{ByteArray, Bytes32};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};
use sha2::{Digest, Sha256};
use strict_encoding::STRICT_TYPES_LIB;

use crate::layout::TypeLayout;
use crate::{CommitConsume, SemId, Ty};

pub const MEM_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:mem:v02";

/// Semantic type id, which commits to the type memory layout, name and field/variant names.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct MemId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl ToBaid58<32> for MemId {
    const HRI: &'static str = "mem";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_byte_array() }
}
impl FromBaid58<32> for MemId {}
impl FromStr for MemId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_baid58_str(s.trim_start_matches("urn:ubideco:"))
    }
}
impl Display for MemId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.to_baid58().mnemonic())
        } else if f.sign_minus() {
            write!(f, "{:#}", self.to_baid58())
        } else if f.sign_aware_zero_pad() {
            write!(f, "urn:ubideco:{::<}", self.to_baid58())
        } else {
            write!(f, "urn:ubideco:{::<#}", self.to_baid58())
        }
    }
}

pub trait MemCommit {
    fn mem_commit(&self, hasher: &mut impl CommitConsume);
}

impl MemCommit for Ty<SemId> {
    fn mem_commit(&self, hasher: &mut impl CommitConsume) {
        hasher.commit_consume(&[self.cls() as u8]);
        match self {
            Ty::Primitive(_) => {}
            Ty::UnicodeChar => {}
            Ty::Enum(_) => {}
            Ty::Union(_) => {}
            Ty::Tuple(_) => {}
            Ty::Struct(_) => {}
            Ty::Array(_, _) => {}
            Ty::List(_, _) => {}
            Ty::Set(_, _) => {}
            Ty::Map(_, _, _) => {}
        }
    }
}

impl MemCommit for TypeLayout {
    fn mem_commit(&self, hasher: &mut impl CommitConsume) {
        for item in self {
            item.ty.mem_commit(hasher);
        }
    }
}

impl TypeLayout {
    pub fn mem_id(&self) -> MemId {
        let tag = Sha256::new_with_prefix(MEM_ID_TAG).finalize();
        let mut hasher = Sha256::new();
        hasher.commit_consume(tag);
        hasher.commit_consume(tag);
        self.mem_commit(&mut hasher);
        MemId::from_byte_array(hasher.finalize())
    }
}
