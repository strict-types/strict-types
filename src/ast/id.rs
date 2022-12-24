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

use amplify::Wrapper;
use bitcoin_hashes::{sha256, Hash, HashEngine};

use crate::ast::{Field, Fields, Variants};
use crate::util::Sizing;
use crate::{Cls, KeyTy, Ty, TypeName, TypeRef};

/// Semantic type id, which commits to the type memory layout, name and field/variant names.
#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref)]
#[display(inner)]
pub struct SemId(sha256::Hash);

impl Ord for SemId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.into_inner().cmp(&other.0.into_inner()) }
}

impl PartialOrd for SemId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

pub const SEM_ID_TAG: [u8; 32] = [0u8; 32];

impl<Ref: TypeRef> Ty<Ref> {
    pub fn id(&self, name: Option<&TypeName>) -> SemId {
        let mut hasher = sha256::HashEngine::default();
        if let Some(ref name) = name {
            hasher.input(name.as_bytes());
        }
        self.hash(&mut hasher);
        SemId(sha256::Hash::from_engine(hasher))
    }

    fn hash(&self, hasher: &mut sha256::HashEngine) {
        self.cls().hash(hasher);
        match self {
            Ty::Primitive(prim) => {
                hasher.input(&[prim.into_code()]);
            }
            Ty::Enum(vars) => vars.hash(hasher),
            Ty::Union(fields) => fields.hash(hasher),
            Ty::Struct(fields) => fields.hash(hasher),
            Ty::Array(ty, len) => {
                hasher.input(&ty.id().into_inner());
                hasher.input(&len.to_le_bytes());
            }
            Ty::UnicodeChar => {}
            Ty::List(ty, sizing) => {
                hasher.input(&ty.id().into_inner());
                sizing.hash(hasher);
            }
            Ty::Set(ty, sizing) => {
                hasher.input(&ty.id().into_inner());
                sizing.hash(hasher);
            }
            Ty::Map(key, ty, sizing) => {
                key.hash(hasher);
                hasher.input(&ty.id().into_inner());
                sizing.hash(hasher);
            }
        };
    }
}

impl KeyTy {
    pub fn id(&self) -> SemId {
        let mut hasher = sha256::HashEngine::default();
        self.hash(&mut hasher);
        SemId(sha256::Hash::from_engine(hasher))
    }

    fn hash(&self, hasher: &mut sha256::HashEngine) {
        self.cls().hash(hasher);
        match self {
            KeyTy::Primitive(prim) => {
                hasher.input(&[prim.into_code()]);
            }
            KeyTy::Enum(vars) => vars.hash(hasher),
            KeyTy::Array(len) => {
                hasher.input(&len.to_le_bytes());
            }
            KeyTy::AsciiStr(sizing) | KeyTy::UnicodeStr(sizing) | KeyTy::Bytes(sizing) => {
                sizing.hash(hasher)
            }
        };
    }
}

impl Cls {
    fn hash(&self, hasher: &mut sha256::HashEngine) { hasher.input(&[*self as u8]); }
}

impl Variants {
    fn hash(&self, hasher: &mut sha256::HashEngine) {
        for field in self {
            field.hash(hasher);
        }
    }
}

impl<Ref: TypeRef, const OP: bool> Fields<Ref, OP> {
    fn hash(&self, hasher: &mut sha256::HashEngine) {
        for (field, ty) in self {
            field.hash(hasher);
            hasher.input(&ty.id().into_inner());
        }
    }
}

impl Field {
    fn hash(&self, hasher: &mut sha256::HashEngine) {
        match &self.name {
            None => hasher.input(&[0u8]),
            Some(name) => {
                hasher.input(&[1u8]);
                hasher.input(name.as_bytes())
            }
        };
        hasher.input(&[self.ord]);
    }
}

impl Sizing {
    fn hash(&self, hasher: &mut sha256::HashEngine) {
        let mut data = self.min.to_le_bytes().to_vec();
        data.extend(self.max.to_le_bytes());
        hasher.input(&data);
    }
}
