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

use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

use baid58::ToBaid58;
use strict_encoding::StrictDumb;

use crate::typelib::TypeLib;

pub const LIB_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:lib:v01";

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref)]
pub struct TypeLibId(
    #[from]
    #[from([u8; 32])]
    blake3::Hash,
);

impl Ord for TypeLibId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for TypeLibId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl StrictDumb for TypeLibId {
    fn strict_dumb() -> Self { TypeLibId(blake3::Hash::from([0u8; 32])) }
}

impl ToBaid58<32> for TypeLibId {
    const HRP: &'static str = "stl";

    fn to_baid58_payload(&self) -> [u8; 32] { *self.0.as_bytes() }
}

impl Display for TypeLibId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let baid58 = self.to_baid58();
        Display::fmt(&baid58, f)
    }
}

impl TypeLib {
    pub fn id(&self) -> TypeLibId {
        let mut hasher = blake3::Hasher::new_keyed(&LIB_ID_TAG);
        for ty in self.types.values() {
            hasher.update(ty.id().as_bytes());
        }
        TypeLibId(hasher.finalize())
    }
}
