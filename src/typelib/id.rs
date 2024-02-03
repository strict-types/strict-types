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

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use amplify::{ByteArray, Bytes32};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};
use encoding::StrictEncode;
use sha2::{Digest, Sha256};
use strict_encoding::{StrictDumb, STRICT_TYPES_LIB};

use crate::ast::HashId;
use crate::typelib::{ExternRef, InlineRef, InlineRef1, InlineRef2, TypeLib};
use crate::{Dependency, LibRef, SymbolRef, TranspileRef};

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

impl HashId for TypeLibId {
    fn hash_id(&self, hasher: &mut Sha256) { hasher.update(self.as_slice()); }
}

impl HashId for TypeLib {
    fn hash_id(&self, hasher: &mut Sha256) {
        self.name.hash_id(hasher);
        hasher.update([self.dependencies.len_u8()]);
        for dep in &self.dependencies {
            dep.hash_id(hasher);
        }
        hasher.update(self.types.len_u16().to_le_bytes());
        for (name, ty) in &self.types {
            let sem_id = ty.id(Some(name));
            sem_id.hash_id(hasher);
        }
    }
}

impl HashId for Dependency {
    fn hash_id(&self, hasher: &mut Sha256) { self.id.hash_id(hasher); }
}

impl HashId for TranspileRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        match self {
            TranspileRef::Embedded(ty) => ty.hash_id(hasher),
            TranspileRef::Named(name) => name.hash_id(hasher),
            TranspileRef::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl HashId for SymbolRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) { self.sem_id.hash_id(hasher); }
}

impl HashId for LibRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        match self {
            LibRef::Inline(ty) => ty.hash_id(hasher),
            LibRef::Named(id) => id.hash_id(hasher),
            LibRef::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl HashId for InlineRef2 {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        match self {
            InlineRef2::Named(sem_id) => sem_id.hash_id(hasher),
            InlineRef2::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl HashId for InlineRef1 {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        match self {
            InlineRef1::Inline(ty) => ty.hash_id(hasher),
            InlineRef1::Named(id) => id.hash_id(hasher),
            InlineRef1::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl HashId for InlineRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        match self {
            InlineRef::Inline(ty) => ty.hash_id(hasher),
            InlineRef::Named(id) => id.hash_id(hasher),
            InlineRef::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl HashId for ExternRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) { self.sem_id.hash_id(hasher); }
}

impl TypeLib {
    pub fn id(&self) -> TypeLibId {
        let tag = Sha256::new_with_prefix(LIB_ID_TAG).finalize();
        let mut hasher = Sha256::new();
        hasher.update(tag);
        hasher.update(tag);
        self.hash_id(&mut hasher);
        TypeLibId::from_byte_array(hasher.finalize())
    }
}
