// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 Ubideco Project
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

use std::cmp::Ordering;
use std::hash::Hash;

use amplify::Wrapper;

use crate::ast::{Field, Fields, Variants};
use crate::util::Sizing;
use crate::{Cls, KeyTy, StenSchema, StenType, Ty, TypeRef};

/// Semantic type id, which commits to the type memory layout, name and field/variant names.
#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref)]
#[display(inner)]
pub struct SemId(blake3::Hash);

impl StenSchema for SemId {
    const STEN_TYPE_NAME: &'static str = "SemId";

    fn sten_ty() -> Ty<StenType> { Ty::<StenType>::byte_array(32) }
}

impl Ord for SemId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for SemId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

pub const SEM_ID_TAG: [u8; 32] = [0u8; 32];

impl<Ref: TypeRef> Ty<Ref> {
    pub fn id(&self) -> SemId {
        let mut hasher = blake3::Hasher::new_keyed(&SEM_ID_TAG);
        self.hash(&mut hasher);
        SemId(hasher.finalize())
    }

    fn hash(&self, hasher: &mut blake3::Hasher) {
        self.cls().hash(hasher);
        match self {
            Ty::Primitive(prim) => {
                hasher.update(&[prim.into_code()]);
            }
            Ty::Enum(vars) => vars.hash(hasher),
            Ty::Union(fields) => fields.hash(hasher),
            Ty::Struct(fields) => fields.hash(hasher),
            Ty::Array(ty, len) => {
                hasher.update(ty.id().as_bytes());
                hasher.update(&len.to_le_bytes());
            }
            Ty::UnicodeChar => {}
            Ty::List(ty, sizing) => {
                hasher.update(ty.id().as_bytes());
                sizing.hash(hasher);
            }
            Ty::Set(ty, sizing) => {
                hasher.update(ty.id().as_bytes());
                sizing.hash(hasher);
            }
            Ty::Map(key, ty, sizing) => {
                key.hash(hasher);
                hasher.update(ty.id().as_bytes());
                sizing.hash(hasher);
            }
        };
    }
}

impl StenType {
    pub fn id(&self) -> SemId {
        let mut hasher = blake3::Hasher::new_keyed(&SEM_ID_TAG);
        self.hash(&mut hasher);
        SemId(hasher.finalize())
    }

    fn hash(&self, hasher: &mut blake3::Hasher) {
        if let Some(ref name) = self.name {
            hasher.update(name.as_bytes());
        }
        self.ty.hash(hasher);
    }
}

impl KeyTy {
    fn hash(&self, hasher: &mut blake3::Hasher) {
        self.cls().hash(hasher);
        match self {
            KeyTy::Primitive(prim) => {
                hasher.update(&[prim.into_code()]);
            }
            KeyTy::Enum(vars) => vars.hash(hasher),
            KeyTy::Array(len) => {
                hasher.update(&len.to_le_bytes());
            }
            KeyTy::AsciiStr(sizing) | KeyTy::UnicodeStr(sizing) | KeyTy::Bytes(sizing) => {
                sizing.hash(hasher)
            }
        };
    }
}

impl Cls {
    fn hash(&self, hasher: &mut blake3::Hasher) { hasher.update(&[*self as u8]); }
}

impl Variants {
    fn hash(&self, hasher: &mut blake3::Hasher) {
        for field in self {
            field.hash(hasher);
        }
    }
}

impl<Ref: TypeRef, const OP: bool> Fields<Ref, OP> {
    fn hash(&self, hasher: &mut blake3::Hasher) {
        for (field, ty) in self {
            field.hash(hasher);
            hasher.update(ty.id().as_bytes());
        }
    }
}

impl Field {
    fn hash(&self, hasher: &mut blake3::Hasher) {
        match &self.name {
            None => hasher.update(&[0u8]),
            Some(name) => {
                hasher.update(&[1u8]);
                hasher.update(name.as_bytes())
            }
        };
        hasher.update(&[self.ord]);
    }
}

impl Sizing {
    fn hash(&self, hasher: &mut blake3::Hasher) {
        let mut data = self.min.to_le_bytes().to_vec();
        data.extend(self.max.to_le_bytes());
        hasher.update(&data);
    }
}
