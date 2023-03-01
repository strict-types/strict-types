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
use std::fmt::{self, Display, Formatter};

use amplify::confinement;
use amplify::confinement::MediumOrdMap;
use amplify::num::u24;
use blake3::Hasher;
use encoding::{StrictDeserialize, StrictSerialize};
use strict_encoding::STRICT_TYPES_LIB;

use crate::ast::HashId;
use crate::{SemId, Ty, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { EmbeddedRef::SemId(SemId::strict_dumb()) })]
pub enum EmbeddedRef {
    #[from]
    SemId(SemId),

    #[from]
    Inline(Ty<SemId>),
}

impl TypeRef for EmbeddedRef {
    const TYPE_NAME: &'static str = "EmbeddedRef";

    fn id(&self) -> SemId {
        match self {
            EmbeddedRef::SemId(id) => *id,
            EmbeddedRef::Inline(ty) => ty.id(None),
        }
    }
    fn is_compound(&self) -> bool {
        match self {
            EmbeddedRef::SemId(_) => false,
            EmbeddedRef::Inline(ty) => ty.is_compound(),
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

impl HashId for EmbeddedRef {
    fn hash_id(&self, hasher: &mut Hasher) {
        match self {
            EmbeddedRef::Inline(ty) => ty.hash_id(hasher),
            EmbeddedRef::SemId(id) => id.hash_id(hasher),
        }
    }
}

impl Display for EmbeddedRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddedRef::SemId(id) => Display::fmt(id, f),
            EmbeddedRef::Inline(ty) => Display::fmt(ty, f),
        }
    }
}

#[derive(Wrapper, WrapperMut, Clone, Eq, PartialEq, Debug, Default, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct TypeSystem(MediumOrdMap<SemId, Ty<EmbeddedRef>>);

impl StrictSerialize for TypeSystem {}
impl StrictDeserialize for TypeSystem {}

impl TypeSystem {
    pub fn try_from_iter<T: IntoIterator<Item = Ty<EmbeddedRef>>>(
        iter: T,
    ) -> Result<Self, confinement::Error> {
        let mut lib: BTreeMap<SemId, Ty<EmbeddedRef>> = empty!();
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
}
