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
use std::hash::Hash;
use std::str::FromStr;

use amplify::{ByteArray, Bytes32, Wrapper};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};
use encoding::{FieldName, LibName, VariantName};
use sha2::Digest;
use strict_encoding::{Sizing, TypeName, Variant, STRICT_TYPES_LIB};

use crate::ast::ty::{Field, UnionVariants, UnnamedFields};
use crate::ast::{EnumVariants, NamedFields, PrimitiveRef};
use crate::{Cls, Ty, TypeRef};

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
    fn default() -> Self { Ty::<SemId>::UNIT.id(None) }
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
    fn is_byte(&self) -> bool { Self::byte() == *self || Ty::<Self>::U8.id(None) == *self }
}

impl PrimitiveRef for SemId {
    fn byte() -> Self { Ty::<Self>::BYTE.id(None) }
    fn unicode_char() -> Self { Ty::<Self>::UNICODE.id(None) }
}

impl<Ref: TypeRef> Ty<Ref> {
    pub fn id(&self, name: Option<&TypeName>) -> SemId {
        let tag = sha2::Sha256::new_with_prefix(SEM_ID_TAG).finalize();
        let mut hasher = sha2::Sha256::new();
        hasher.update(tag);
        hasher.update(tag);
        if let Some(name) = name {
            name.hash_id(&mut hasher);
        }
        self.hash_id(&mut hasher);
        SemId::from_byte_array(hasher.finalize())
    }
}

pub trait HashId {
    fn hash_id(&self, hasher: &mut sha2::Sha256);
}

impl HashId for LibName {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        hasher.update([self.len() as u8]);
        hasher.update(self.as_bytes());
    }
}

impl HashId for TypeName {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        hasher.update([self.len() as u8]);
        hasher.update(self.as_bytes());
    }
}

impl HashId for FieldName {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        hasher.update([self.len() as u8]);
        hasher.update(self.as_bytes());
    }
}

impl HashId for VariantName {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        hasher.update([self.len() as u8]);
        hasher.update(self.as_bytes());
    }
}

impl HashId for SemId {
    fn hash_id(&self, hasher: &mut sha2::Sha256) { hasher.update(self.as_slice()); }
}

impl<Ref: TypeRef> HashId for Ty<Ref> {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        self.cls().hash_id(hasher);
        match self {
            Ty::Primitive(prim) => {
                hasher.update([prim.into_code()]);
            }
            Ty::Enum(vars) => vars.hash_id(hasher),
            Ty::Union(fields) => fields.hash_id(hasher),
            Ty::Tuple(fields) => fields.hash_id(hasher),
            Ty::Struct(fields) => fields.hash_id(hasher),
            Ty::Array(ty, len) => {
                ty.hash_id(hasher);
                hasher.update(len.to_le_bytes());
            }
            Ty::UnicodeChar => {}
            Ty::List(ty, sizing) => {
                ty.hash_id(hasher);
                sizing.hash_id(hasher);
            }
            Ty::Set(ty, sizing) => {
                ty.hash_id(hasher);
                sizing.hash_id(hasher);
            }
            Ty::Map(key, ty, sizing) => {
                key.hash_id(hasher);
                ty.hash_id(hasher);
                sizing.hash_id(hasher);
            }
        };
    }
}

impl HashId for Cls {
    fn hash_id(&self, hasher: &mut sha2::Sha256) { hasher.update([*self as u8]); }
}

impl<Ref: TypeRef> HashId for Field<Ref> {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        self.name.hash_id(hasher);
        self.ty.hash_id(hasher);
    }
}

impl HashId for EnumVariants {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        for variant in self {
            variant.hash_id(hasher);
        }
    }
}

impl<Ref: TypeRef> HashId for UnionVariants<Ref> {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        for (variant, ty) in self {
            variant.hash_id(hasher);
            ty.hash_id(hasher);
        }
    }
}

impl<Ref: TypeRef> HashId for NamedFields<Ref> {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        for field in self {
            field.hash_id(hasher);
        }
    }
}

impl<Ref: TypeRef> HashId for UnnamedFields<Ref> {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        for ty in self {
            ty.hash_id(hasher);
        }
    }
}

impl HashId for Variant {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        self.name.hash_id(hasher);
        hasher.update([self.tag]);
    }
}

impl HashId for Sizing {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        let mut data = self.min.to_le_bytes().to_vec();
        data.extend(self.max.to_le_bytes());
        hasher.update(&data);
    }
}
