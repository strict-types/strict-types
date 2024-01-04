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

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use amplify::{ByteArray, Bytes32};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};
use encoding::StrictEncode;
use sha2::{Digest, Sha256};
use strict_encoding::{StrictDumb, STRICT_TYPES_LIB};

use crate::ast::SemCommit;
use crate::typelib::{ExternRef, InlineRef, InlineRef1, InlineRef2, TypeLib};
use crate::{CommitConsume, Dependency, LibRef, SymbolRef, TranspileRef};

pub const LIB_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:lib:v01";

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
pub struct TypeLibId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl ToBaid58<32> for TypeLibId {
    const HRI: &'static str = "stl";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_byte_array() }
}
impl FromBaid58<32> for TypeLibId {}
impl FromStr for TypeLibId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_baid58_str(s.trim_start_matches("urn:ubideco:"))
    }
}
impl Display for TypeLibId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.sign_minus() {
            write!(f, "urn:ubideco:{::<}", self.to_baid58())
        } else {
            write!(f, "urn:ubideco:{::<#}", self.to_baid58())
        }
    }
}

impl SemCommit for TypeLibId {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        hasher.commit_consume(self.as_slice());
    }
}

impl SemCommit for TypeLib {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        self.name.sem_commit(hasher);
        hasher.commit_consume([self.dependencies.len_u8()]);
        for dep in &self.dependencies {
            dep.sem_commit(hasher);
        }
        hasher.commit_consume(self.types.len_u16().to_le_bytes());
        for (name, ty) in &self.types {
            let sem_id = ty.id(Some(name));
            sem_id.sem_commit(hasher);
        }
    }
}

impl SemCommit for Dependency {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) { self.id.sem_commit(hasher); }
}

impl SemCommit for TranspileRef {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        match self {
            TranspileRef::Embedded(ty) => ty.sem_commit(hasher),
            TranspileRef::Named(name) => name.sem_commit(hasher),
            TranspileRef::Extern(ext) => ext.sem_commit(hasher),
        }
    }
}

impl SemCommit for SymbolRef {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) { self.sem_id.sem_commit(hasher); }
}

impl SemCommit for LibRef {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        match self {
            LibRef::Inline(ty) => ty.sem_commit(hasher),
            LibRef::Named(id) => id.sem_commit(hasher),
            LibRef::Extern(ext) => ext.sem_commit(hasher),
        }
    }
}

impl SemCommit for InlineRef2 {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        match self {
            InlineRef2::Named(sem_id) => sem_id.sem_commit(hasher),
            InlineRef2::Extern(ext) => ext.sem_commit(hasher),
        }
    }
}

impl SemCommit for InlineRef1 {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        match self {
            InlineRef1::Inline(ty) => ty.sem_commit(hasher),
            InlineRef1::Named(id) => id.sem_commit(hasher),
            InlineRef1::Extern(ext) => ext.sem_commit(hasher),
        }
    }
}

impl SemCommit for InlineRef {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        match self {
            InlineRef::Inline(ty) => ty.sem_commit(hasher),
            InlineRef::Named(id) => id.sem_commit(hasher),
            InlineRef::Extern(ext) => ext.sem_commit(hasher),
        }
    }
}

impl SemCommit for ExternRef {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) { self.sem_id.sem_commit(hasher); }
}

impl TypeLib {
    pub fn id(&self) -> TypeLibId {
        let tag = Sha256::new_with_prefix(LIB_ID_TAG).finalize();
        let mut hasher = Sha256::new();
        hasher.commit_consume(tag);
        hasher.commit_consume(tag);
        self.sem_commit(&mut hasher);
        TypeLibId::from_byte_array(hasher.finalize())
    }
}
