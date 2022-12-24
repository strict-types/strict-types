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

use bitcoin_hashes::{sha256, Hash, HashEngine};

use crate::typelib::TypeLib;

// TODO: Use real tag
pub const LIB_ID_TAG: [u8; 32] = [0u8; 32];

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref)]
pub struct TypeLibId(sha256::Hash);

impl Ord for TypeLibId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.into_inner().cmp(&other.0.into_inner()) }
}

impl PartialOrd for TypeLibId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Display for TypeLibId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let m = mnemonic::to_string(&self.into_inner()[14..18]);
            write!(f, "{}#{}", self.0, m)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl TypeLib {
    pub fn id(&self) -> TypeLibId {
        let mut hasher = sha256::HashEngine::default();
        for ty in self.types.values() {
            hasher.input(&ty.id().into_inner());
        }
        TypeLibId(sha256::Hash::from_engine(hasher))
    }
}
