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

use std::hash::Hash;
use std::str::FromStr;

use amplify::{Bytes32, RawArray, Wrapper};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};
use blake3::Hasher;
use strict_encoding::{Sizing, StrictDumb, TypeName, Variant, STRICT_TYPES_LIB};

use crate::ast::ty::{Field, UnionVariants, UnnamedFields};
use crate::ast::{EnumVariants, NamedFields};
use crate::{Cls, KeyTy, Ty, TypeRef};

/// Semantic type id, which commits to the type memory layout, name and field/variant names.
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
pub struct SemId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl ToBaid58<32> for SemId {
    const HRI: &'static str = "sty";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_raw_array() }
}
impl FromBaid58<32> for SemId {}
impl FromStr for SemId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_baid58_str(s) }
}

pub const SEM_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:typ:v01";

pub trait HashId {
    fn hash_id(&self, hasher: &mut blake3::Hasher);
}

impl<Ref: TypeRef> Ty<Ref> {
    pub fn id(&self, name: Option<&TypeName>) -> SemId {
        let mut hasher = blake3::Hasher::new_keyed(&SEM_ID_TAG);
        if let Some(name) = name {
            hasher.update(name.as_bytes());
        }
        self.hash_id(&mut hasher);
        SemId::from_raw_array(hasher.finalize())
    }
}

impl HashId for SemId {
    fn hash_id(&self, hasher: &mut Hasher) { hasher.update(self.as_slice()); }
}

impl<Ref: TypeRef> HashId for Ty<Ref> {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        self.cls().hash_id(hasher);
        match self {
            Ty::Primitive(prim) => {
                hasher.update(&[prim.into_code()]);
            }
            Ty::Enum(vars) => vars.hash_id(hasher),
            Ty::Union(fields) => fields.hash_id(hasher),
            Ty::Tuple(fields) => fields.hash_id(hasher),
            Ty::Struct(fields) => fields.hash_id(hasher),
            Ty::Array(ty, len) => {
                ty.hash_id(hasher);
                hasher.update(&len.to_le_bytes());
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

impl HashId for KeyTy {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        self.cls().hash_id(hasher);
        match self {
            KeyTy::Primitive(prim) => {
                hasher.update(&[prim.into_code()]);
            }
            KeyTy::Enum(vars) => vars.hash_id(hasher),
            KeyTy::Array(len) => {
                hasher.update(&len.to_le_bytes());
            }
            KeyTy::AsciiStr(sizing) | KeyTy::UnicodeStr(sizing) | KeyTy::Bytes(sizing) => {
                sizing.hash_id(hasher)
            }
        };
    }
}

impl HashId for Cls {
    fn hash_id(&self, hasher: &mut blake3::Hasher) { hasher.update(&[*self as u8]); }
}

impl<Ref: TypeRef> HashId for Field<Ref> {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.name.as_bytes());
        self.ty.hash_id(hasher);
    }
}

impl HashId for EnumVariants {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        for variant in self {
            variant.hash_id(hasher);
        }
    }
}

impl<Ref: TypeRef> HashId for UnionVariants<Ref> {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        for (variant, ty) in self {
            variant.hash_id(hasher);
            ty.hash_id(hasher);
        }
    }
}

impl<Ref: TypeRef> HashId for NamedFields<Ref> {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        for field in self {
            field.hash_id(hasher);
        }
    }
}

impl<Ref: TypeRef> HashId for UnnamedFields<Ref> {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        for ty in self {
            ty.hash_id(hasher);
        }
    }
}

impl HashId for Variant {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.name.as_bytes());
        hasher.update(&[self.tag]);
    }
}

impl HashId for Sizing {
    fn hash_id(&self, hasher: &mut blake3::Hasher) {
        let mut data = self.min.to_le_bytes().to_vec();
        data.extend(self.max.to_le_bytes());
        hasher.update(&data);
    }
}
