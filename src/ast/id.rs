// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
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

use amplify::Wrapper;

use crate::ast::{Field, Fields, TyInner, Variants};
use crate::util::Sizing;
use crate::{Cls, KeyTy, Ty, TypeRef};

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref)]
#[display("urn:ubideco:sten:{0}")]
pub struct TyId(blake3::Hash);

impl Ord for TyId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for TyId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

pub const STEN_ID_TAG: [u8; 32] = [0u8; 32];

impl<Ref: TypeRef> Ty<Ref> {
    pub fn id(&self) -> TyId {
        let mut hasher = blake3::Hasher::new_keyed(&STEN_ID_TAG);
        self.hash(&mut hasher);
        TyId(hasher.finalize())
    }

    fn hash(&self, hasher: &mut blake3::Hasher) {
        self.cls().hash(hasher);
        match self.as_inner() {
            TyInner::Primitive(prim) => {
                hasher.update(&[prim.into_code()]);
            }
            TyInner::Enum(vars) => vars.hash(hasher),
            TyInner::Union(fields) => fields.hash(hasher),
            TyInner::Struct(fields) => fields.hash(hasher),
            TyInner::Array(ty, len) => {
                hasher.update(ty.id().as_bytes());
                hasher.update(&len.to_le_bytes());
            }
            TyInner::UnicodeChar => {}
            TyInner::List(ty, sizing) => {
                hasher.update(ty.id().as_bytes());
                sizing.hash(hasher);
            }
            TyInner::Set(ty, sizing) => {
                hasher.update(ty.id().as_bytes());
                sizing.hash(hasher);
            }
            TyInner::Map(key, ty, sizing) => {
                key.hash(hasher);
                hasher.update(ty.id().as_bytes());
                sizing.hash(hasher);
            }
        };
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
            KeyTy::UnicodeStr(sizing) | KeyTy::Bytes(sizing) => sizing.hash(hasher),
        };
    }
}

impl Cls {
    fn hash(&self, hasher: &mut blake3::Hasher) { hasher.update(&[*self as u8]); }
}

impl Variants {
    fn hash(&self, hasher: &mut blake3::Hasher) {
        for (field, val) in self {
            field.hash(hasher);
            hasher.update(&[*val]);
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
