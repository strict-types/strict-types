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

//! Embedded lib is a set of compiled type libraries having no external
//! dependencies

use std::collections::BTreeMap;
use std::ops::Deref;

use amplify::confinement;
use amplify::confinement::MediumOrdMap;
use amplify::num::u24;

use crate::ast::NestedRef;
use crate::{Serialize, StenSchema, StenType, Ty, TyId, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum EmbeddedTy {
    Ref(TyId),

    #[from]
    Inline(Box<Ty<EmbeddedTy>>),
}

impl StenSchema for EmbeddedTy {
    const STEN_TYPE_NAME: &'static str = "EmbeddedTy";

    fn sten_ty() -> Ty<StenType> { todo!() }
}

impl Deref for EmbeddedTy {
    type Target = Ty<EmbeddedTy>;

    fn deref(&self) -> &Self::Target {
        match self {
            EmbeddedTy::Ref(_) => &Ty::UNIT,
            EmbeddedTy::Inline(ty) => ty.as_ref(),
        }
    }
}

impl TypeRef for EmbeddedTy {
    fn id(&self) -> TyId {
        match self {
            EmbeddedTy::Ref(id) => *id,
            EmbeddedTy::Inline(ty) => ty.id(),
        }
    }
}

impl NestedRef for EmbeddedTy {
    fn as_ty(&self) -> &Ty<Self> { self.deref() }

    fn into_ty(self) -> Ty<Self> {
        match self {
            EmbeddedTy::Ref(_) => Ty::UNIT,
            EmbeddedTy::Inline(ty) => *ty,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub struct TypeSystem(MediumOrdMap<TyId, Ty<EmbeddedTy>>);

impl Deref for TypeSystem {
    type Target = BTreeMap<TyId, Ty<EmbeddedTy>>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl IntoIterator for TypeSystem {
    type Item = (TyId, Ty<EmbeddedTy>);
    type IntoIter = std::collections::btree_map::IntoIter<TyId, Ty<EmbeddedTy>>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'lib> IntoIterator for &'lib TypeSystem {
    type Item = (&'lib TyId, &'lib Ty<EmbeddedTy>);
    type IntoIter = std::collections::btree_map::Iter<'lib, TyId, Ty<EmbeddedTy>>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl TypeSystem {
    pub fn try_from_iter<T: IntoIterator<Item = Ty<EmbeddedTy>>>(
        iter: T,
    ) -> Result<Self, confinement::Error> {
        let mut lib: BTreeMap<TyId, Ty<EmbeddedTy>> = empty!();
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
