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
use std::hash::Hash;
use std::str::FromStr;

use amplify::{ByteArray, Bytes32, Wrapper};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};
use encoding::{FieldName, LibName};
use sha2::Digest;
use strict_encoding::{Sizing, TypeName, Variant, STRICT_TYPES_LIB};

use crate::ast::ty::{Field, UnionVariants, UnnamedFields};
use crate::ast::{EnumVariants, NamedFields, PrimitiveRef};
use crate::{Cls, CommitConsume, Ty, TypeRef};

/// Semantic type id, which commits to the type memory layout, name and field/variant names.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct SemId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl Default for SemId {
    fn default() -> Self { Ty::<SemId>::UNIT.sem_id(None) }
}

impl ToBaid58<32> for SemId {
    const HRI: &'static str = "semid";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_byte_array() }
}
impl FromBaid58<32> for SemId {}
impl FromStr for SemId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_baid58_str(s.trim_start_matches("urn:ubideco:"))
    }
}
impl Display for SemId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.sign_minus() {
            write!(f, "urn:ubideco:{::<}", self.to_baid58())
        } else {
            write!(f, "urn:ubideco:{::<#}", self.to_baid58())
        }
    }
}

pub const SEM_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:typ:v01";

impl TypeRef for SemId {
    fn is_unicode_char(&self) -> bool { Self::unicode_char() == *self }
    fn is_byte(&self) -> bool { Self::byte() == *self || Ty::<Self>::U8.sem_id(None) == *self }
}

impl PrimitiveRef for SemId {
    fn byte() -> Self { Ty::<Self>::BYTE.sem_id(None) }
    fn unicode_char() -> Self { Ty::<Self>::UNICODE.sem_id(None) }
}

impl<Ref: TypeRef> Ty<Ref> {
    pub fn sem_id(&self, name: Option<&TypeName>) -> SemId {
        let tag = sha2::Sha256::new_with_prefix(SEM_ID_TAG).finalize();
        let mut hasher = sha2::Sha256::new();
        hasher.commit_consume(tag);
        hasher.commit_consume(tag);
        if let Some(name) = name {
            name.sem_commit(&mut hasher);
        }
        self.sem_commit(&mut hasher);
        SemId::from_byte_array(hasher.finalize())
    }
}

pub trait SemCommit {
    fn sem_commit(&self, hasher: &mut impl CommitConsume);
}

impl SemCommit for LibName {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        hasher.commit_consume([self.len() as u8]);
        hasher.commit_consume(self.as_bytes());
    }
}

impl SemCommit for TypeName {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        hasher.commit_consume([self.len() as u8]);
        hasher.commit_consume(self.as_bytes());
    }
}

impl SemCommit for FieldName {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        hasher.commit_consume([self.len() as u8]);
        hasher.commit_consume(self.as_bytes());
    }
}

impl SemCommit for SemId {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        hasher.commit_consume(self.as_slice());
    }
}

impl<Ref: TypeRef> SemCommit for Ty<Ref> {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        self.cls().sem_commit(hasher);
        match self {
            Ty::Primitive(prim) => {
                hasher.commit_consume([prim.into_code()]);
            }
            Ty::Enum(vars) => vars.sem_commit(hasher),
            Ty::Union(fields) => fields.sem_commit(hasher),
            Ty::Tuple(fields) => fields.sem_commit(hasher),
            Ty::Struct(fields) => fields.sem_commit(hasher),
            Ty::Array(ty, len) => {
                ty.sem_commit(hasher);
                hasher.commit_consume(len.to_le_bytes());
            }
            Ty::UnicodeChar => {}
            Ty::List(ty, sizing) => {
                ty.sem_commit(hasher);
                sizing.sem_commit(hasher);
            }
            Ty::Set(ty, sizing) => {
                ty.sem_commit(hasher);
                sizing.sem_commit(hasher);
            }
            Ty::Map(key, ty, sizing) => {
                key.sem_commit(hasher);
                ty.sem_commit(hasher);
                sizing.sem_commit(hasher);
            }
        };
    }
}

impl SemCommit for Cls {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) { hasher.commit_consume([*self as u8]); }
}

impl<Ref: TypeRef> SemCommit for Field<Ref> {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        self.name.sem_commit(hasher);
        self.ty.sem_commit(hasher);
    }
}

impl SemCommit for EnumVariants {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        for variant in self {
            variant.sem_commit(hasher);
        }
    }
}

impl<Ref: TypeRef> SemCommit for UnionVariants<Ref> {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        for (variant, ty) in self {
            variant.sem_commit(hasher);
            ty.sem_commit(hasher);
        }
    }
}

impl<Ref: TypeRef> SemCommit for NamedFields<Ref> {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        for field in self {
            field.sem_commit(hasher);
        }
    }
}

impl<Ref: TypeRef> SemCommit for UnnamedFields<Ref> {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        for ty in self {
            ty.sem_commit(hasher);
        }
    }
}

impl SemCommit for Variant {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        self.name.sem_commit(hasher);
        hasher.commit_consume([self.tag]);
    }
}

impl SemCommit for Sizing {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        let mut data = self.min.to_le_bytes().to_vec();
        data.extend(self.max.to_le_bytes());
        hasher.commit_consume(&data);
    }
}
