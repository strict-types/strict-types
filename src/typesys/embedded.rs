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
use std::ops::Deref;

use amplify::confinement;
use amplify::confinement::MediumOrdMap;
use amplify::num::u24;

use crate::{SemId, Serialize, StenSchema, StenType, Ty, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum EmbeddedRef {
    SemId(SemId),

    #[from]
    Inline(Ty<SemId>),
}

impl StenSchema for EmbeddedRef {
    const STEN_TYPE_NAME: &'static str = "EmbeddedRef";

    fn sten_ty() -> Ty<StenType> { todo!() }
}

impl TypeRef for EmbeddedRef {
    fn id(&self) -> SemId {
        match self {
            EmbeddedRef::SemId(id) => *id,
            EmbeddedRef::Inline(ty) => ty.id(None),
        }
    }
    fn is_byte(&self) -> bool {
        match self {
            EmbeddedRef::SemId(_) => false,
            EmbeddedRef::Inline(ty) => ty.is_byte(),
        }
    }
    fn is_unicode_char(&self) -> bool {
        match self {
            EmbeddedRef::SemId(_) => false,
            EmbeddedRef::Inline(ty) => ty.is_unicode_char(),
        }
    }
    fn is_ascii_char(&self) -> bool {
        match self {
            EmbeddedRef::SemId(_) => false,
            EmbeddedRef::Inline(ty) => ty.is_ascii_char(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub struct TypeSystem(MediumOrdMap<SemId, Ty<EmbeddedRef>>);

impl Deref for TypeSystem {
    type Target = BTreeMap<SemId, Ty<EmbeddedRef>>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl IntoIterator for TypeSystem {
    type Item = (SemId, Ty<EmbeddedRef>);
    type IntoIter = std::collections::btree_map::IntoIter<SemId, Ty<EmbeddedRef>>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'lib> IntoIterator for &'lib TypeSystem {
    type Item = (&'lib SemId, &'lib Ty<EmbeddedRef>);
    type IntoIter = std::collections::btree_map::Iter<'lib, SemId, Ty<EmbeddedRef>>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl TypeSystem {
    pub fn try_from_iter<T: IntoIterator<Item = Ty<EmbeddedRef>>>(
        iter: T,
    ) -> Result<Self, confinement::Error> {
        let mut lib: BTreeMap<SemId, Ty<EmbeddedRef>> = empty!();
        for ty in iter {
            lib.insert(ty.id(), ty);
        }

        let lib = TypeSystem(MediumOrdMap::try_from_iter(lib)?);
        let len = lib.serialized_len();
        let max_len = u24::MAX.into_usize();
        if len > max_len {
            return Err(confinement::Error::Oversize { len, max_len }.into());
        }
        Ok(lib)
    }

    pub fn count_types(&self) -> u24 { self.0.len_u24() }
}
