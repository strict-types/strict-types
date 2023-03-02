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

use std::collections::BTreeMap;

use amplify::confinement;
use amplify::confinement::MediumOrdMap;
use amplify::num::u24;
use encoding::{StrictDeserialize, StrictSerialize};
use strict_encoding::STRICT_TYPES_LIB;

use crate::{SemId, Ty};

#[derive(Wrapper, WrapperMut, Clone, Eq, PartialEq, Debug, Default, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct TypeSystem(MediumOrdMap<SemId, Ty<SemId>>);

impl StrictSerialize for TypeSystem {}
impl StrictDeserialize for TypeSystem {}

impl TypeSystem {
    pub fn try_from_iter<T: IntoIterator<Item = Ty<SemId>>>(
        iter: T,
    ) -> Result<Self, confinement::Error> {
        let mut lib: BTreeMap<SemId, Ty<SemId>> = empty!();
        for ty in iter {
            lib.insert(ty.id(None), ty);
        }

        let lib = TypeSystem(MediumOrdMap::try_from_iter(lib)?);
        let len = lib.strict_serialized_len().expect("in-memory writer");
        let max_len = u24::MAX.into_usize();
        if len > max_len {
            return Err(confinement::Error::Oversize { len, max_len }.into());
        }
        Ok(lib)
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
