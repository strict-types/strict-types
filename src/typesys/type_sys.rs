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

//! Embedded lib is a set of compiled type libraries having no external
//! dependencies

use std::fmt::{self, Display, Formatter};

use amplify::confinement;
use amplify::confinement::MediumOrdMap;
use amplify::num::u24;
use encoding::{StrictDeserialize, StrictSerialize};
use strict_encoding::STRICT_TYPES_LIB;

use crate::{SemId, Ty};

/// Type system represents a set of strict types assembled from multiple
/// libraries. It is designed to provide all necessary type information to
/// analyze a type with all types it depends onto.
///
/// # Type guarantees
///
/// - Total number of types do not exceed 2^24-1;
/// - Strict-serialized size is less than 2^24 bytes;
/// - Type system is complete (i.e. no type references a type which is not a part of the system).
#[derive(Wrapper, Clone, Eq, PartialEq, Debug, Default, From)]
#[wrapper(Deref)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct TypeSystem(MediumOrdMap<SemId, Ty<SemId>>);

impl StrictSerialize for TypeSystem {}
impl StrictDeserialize for TypeSystem {}

impl TypeSystem {
    pub fn new() -> Self { Self::default() }

    pub(super) fn extend<T: IntoIterator<Item = Ty<SemId>>>(
        &mut self,
        iter: T,
    ) -> Result<usize, confinement::Error> {
        let mut count = 0;
        for ty in iter {
            self.0.insert(ty.id(None), ty)?;
            count += 0;
        }
        Ok(count)
    }

    pub fn count_types(&self) -> u24 { self.0.len_u24() }

    pub fn is_complete(&self) -> bool {
        for ty in self.values() {
            for id in ty.type_refs() {
                if !self.contains_key(id) {
                    return false;
                }
            }
        }
        true
    }
}
