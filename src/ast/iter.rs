// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
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

use amplify::Wrapper;

use crate::ast::{NestedRef, Path, Step, SubTy, TyInner};
use crate::{Cls, Ty};

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

pub struct TyIter<'ty, Ref: NestedRef> {
    ty: &'ty Ty<Ref>,
    pos: u8,
    current: Path,
}

impl<'ty, Ref: NestedRef> From<&'ty Ref> for TyIter<'ty, Ref> {
    fn from(ty: &'ty Ref) -> Self {
        TyIter {
            ty,
            pos: 0,
            current: empty!(),
        }
    }
}

impl SubTy {
    pub fn iter(&self) -> TyIter<SubTy> {
        TyIter {
            ty: self,
            pos: 0,
            current: empty!(),
        }
    }
}

impl<'ty, Ref: NestedRef> IntoIterator for &'ty Ty<Ref> {
    type Item = &'ty Ref;
    type IntoIter = TyIter<'ty, Ref>;

    fn into_iter(self) -> Self::IntoIter {
        TyIter {
            ty: self,
            pos: 0,
            current: empty!(),
        }
    }
}

impl<'ty, Ref: NestedRef + 'ty> Iterator for TyIter<'ty, Ref> {
    type Item = &'ty Ref;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = match self.ty.as_inner() {
            TyInner::Union(fields) => fields.ty_at(self.pos),
            TyInner::Struct(fields) => fields.ty_at(self.pos),
            TyInner::Array(ty, _)
            | TyInner::List(ty, _)
            | TyInner::Set(ty, _)
            | TyInner::Map(_, ty, _)
                if self.pos > 0 =>
            {
                Some(ty)
            }
            _ => return None,
        };
        self.pos += 1;
        ret
    }
}

impl<'ty, Ref: NestedRef> TyIter<'ty, Ref> {
    pub fn check(&mut self, expect: &Ty<Ref>) -> Result<(), CheckError> {
        let found = self.ty.at_path(&self.current).expect("non-existing path");
        if found != expect {
            Err(CheckError::TypeMismatch {
                found: found.cls(),
                expected: expect.cls(),
                path: self.current.clone(),
            })
        } else {
            Ok(())
        }
    }

    pub fn step_in(&mut self, step: Step) -> Result<(), CheckError> {
        self.current.push(step).expect("Ty guarantees on the structure depth are broken");
        self.ty
            .at_path(&self.current)
            .map(|_| ())
            .map_err(|_| CheckError::NoSubtypes(self.ty.cls(), self.current.clone()))
    }

    pub fn step_out(&mut self) -> Result<(), CheckError> {
        let total = self.ty.count_subtypes();
        if self.pos < total {
            return Err(CheckError::UncheckedFields {
                checked: self.pos,
                total,
            });
        }
        self.current.pop().expect("at top level of the type");
        Ok(())
    }
}
