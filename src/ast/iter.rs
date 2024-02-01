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

use crate::ast::{ItemCase, Path};
use crate::{Cls, Ty, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(doc_comments)]
pub enum CheckError {
    /// the type {0} at the current path {1} doesn't have subtypes
    NoSubtypes(Cls, Path),

    /// type {found} found when {expected} was expected at path {path}
    TypeMismatch {
        expected: Cls,
        found: Cls,
        path: Path,
    },

    /// only {checked} fields were checked out of {total} fields in total
    UncheckedFields { checked: u8, total: u8 },
}
pub struct IntoIter<Ref: TypeRef> {
    ty: Ty<Ref>,
    pos: u8,
}

impl<Ref: TypeRef> From<Ty<Ref>> for IntoIter<Ref> {
    fn from(ty: Ty<Ref>) -> Self { IntoIter { ty, pos: 0 } }
}

impl<Ref: TypeRef> IntoIterator for Ty<Ref> {
    type Item = Ref;
    type IntoIter = IntoIter<Ref>;

    fn into_iter(self) -> Self::IntoIter { IntoIter::from(self) }
}

impl<Ref: TypeRef> Iterator for IntoIter<Ref> {
    type Item = Ref;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.ty.ty_at(self.pos);
        self.pos += 1;
        ret.cloned()
    }
}

pub struct Iter<'ty, Ref: TypeRef> {
    ty: &'ty Ty<Ref>,
    pos: u8,
}

impl<Ref: TypeRef> Ty<Ref> {
    pub fn type_refs(&self) -> Iter<Ref> { Iter::from(self) }
}

impl<Ref: TypeRef> Ty<Ref> {
    pub fn iter(&self) -> Iter<'_, Ref> { self.into_iter() }
}

impl<'ty, Ref: TypeRef> From<&'ty Ty<Ref>> for Iter<'ty, Ref> {
    fn from(ty: &'ty Ty<Ref>) -> Self { Iter { ty, pos: 0 } }
}

impl<'ty, Ref: TypeRef> From<&'ty Ref> for Iter<'ty, Ref> {
    fn from(ty: &'ty Ref) -> Self { Self::from(ty.as_ty().unwrap_or(&Ty::UNIT)) }
}

impl<'ty, Ref: TypeRef> IntoIterator for &'ty Ty<Ref> {
    type Item = (&'ty Ref, Option<ItemCase>);
    type IntoIter = Iter<'ty, Ref>;

    fn into_iter(self) -> Self::IntoIter { Iter::from(self) }
}

impl<'ty, Ref: TypeRef + 'ty> Iterator for Iter<'ty, Ref> {
    type Item = (&'ty Ref, Option<ItemCase>);

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.ty.ty_at(self.pos);
        let item = self.ty.case_at(self.pos);
        self.pos += 1;
        r.map(|r| (r, item))
    }
}
