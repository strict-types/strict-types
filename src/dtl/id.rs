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
use std::fmt::{self, Display, Formatter};

use crate::dtl::{TypeLib, TypeSystem};
use crate::{StenSchema, StenType, Ty};

// TODO: Use real tag
pub const LIB_ID_TAG: [u8; 32] = [0u8; 32];

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref)]
pub struct TypeLibId(blake3::Hash);

impl StenSchema for TypeLibId {
    const STEN_TYPE_NAME: &'static str = "TypeLibId";

    fn sten_ty() -> Ty<StenType> { Ty::<StenType>::byte_array(32) }
}

impl Ord for TypeLibId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for TypeLibId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Display for TypeLibId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let m = mnemonic::to_string(&self.as_bytes()[14..18]);
            write!(f, "{}#{}", self.0, m)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

// TODO: Use real tag
pub const TYPESYS_ID_TAG: [u8; 32] = [0u8; 32];

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref)]
pub struct TypeSysId(blake3::Hash);

impl Ord for TypeSysId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for TypeSysId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Display for TypeSysId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let m = mnemonic::to_string(&self.as_bytes()[14..18]);
            write!(f, "{}#{}", self.0, m)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl TypeLib {
    pub fn id(&self) -> TypeLibId {
        let mut hasher = blake3::Hasher::new_keyed(&TYPESYS_ID_TAG);
        for (name, ty) in self.types.iter() {
            hasher.update(name.as_bytes());
            hasher.update(ty.id().as_bytes());
        }
        TypeLibId(hasher.finalize())
    }
}

impl TypeSystem {
    pub fn id(&self) -> TypeSysId {
        let mut hasher = blake3::Hasher::new_keyed(&LIB_ID_TAG);
        for ty in self.values() {
            hasher.update(ty.id().as_bytes());
        }
        TypeSysId(hasher.finalize())
    }
}
