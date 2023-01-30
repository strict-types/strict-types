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

use std::fmt::{Display, Formatter};

use amplify::confinement::SmallVec;
use amplify::Wrapper;
use strict_encoding::{FieldName, STRICT_TYPES_LIB};

use crate::ast::NestedRef;
use crate::Ty;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order)]
pub enum Step {
    #[display(".{0}")]
    #[from]
    NamedField(FieldName),

    #[display(".{0}")]
    #[from]
    UnnamedField(u8),

    #[strict_type(dumb)]
    #[display("#")]
    Index,

    #[display("[]")]
    List,

    #[display("{}")]
    Set,

    #[display("->")]
    Map,
}

#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
pub struct Path(SmallVec<Step>);

impl Path {
    pub fn new() -> Path { Path::default() }

    pub fn with(step: Step) -> Path { Path(small_vec!(step)) }

    pub fn iter(&self) -> std::slice::Iter<Step> { self.0.iter() }
}

impl<'path> IntoIterator for &'path Path {
    type Item = &'path Step;
    type IntoIter = std::slice::Iter<'path, Step>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for step in self {
            Display::fmt(step, f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Display, Error)]
#[display("no type path {path} exists within type {ty:?}")]
pub struct PathError<'ty, Ref: NestedRef> {
    pub ty: &'ty Ty<Ref>,
    pub path: Path,
}

impl<'ty, Ref: NestedRef> PathError<'ty, Ref> {
    pub fn new(ty: &'ty Ty<Ref>, path: Path) -> Self { PathError { ty, path } }
}

impl<Ref: NestedRef> Ty<Ref> {
    pub fn at_path(&self, path: &Path) -> Result<&Self, PathError<Ref>> {
        let mut ty = self;
        let mut path = path.clone();
        let mut path_so_far = Path::new();
        while let Some(step) = path.pop() {
            let res = match (self, &step) {
                (Ty::Struct(fields), Step::NamedField(name)) => fields.ty_by_name(name),
                (Ty::Union(variants), Step::NamedField(name)) => variants.ty_by_name(name),
                (Ty::Struct(fields), Step::UnnamedField(tag)) => fields.ty_by_pos(*tag),
                (Ty::Union(variants), Step::UnnamedField(tag)) => variants.ty_by_ord(*tag),
                (Ty::Array(ty, _), Step::Index) => Some(ty),
                (Ty::List(ty, _), Step::List) => Some(ty),
                (Ty::Set(ty, _), Step::Set) => Some(ty),
                (Ty::Map(_, ty, _), Step::Map) => Some(ty),
                (_, _) => None,
            };
            path_so_far.push(step).expect("confinement collection guarantees");
            ty = res
                .and_then(|r| r.as_ty())
                .ok_or_else(|| PathError::new(self, path_so_far.clone()))?
        }
        Ok(ty)
    }

    pub fn count_type_refs(&self) -> u8 {
        match self {
            Ty::Primitive(_) => 0,
            Ty::Enum(_) => 0,
            Ty::Union(fields) => fields.len_u8(),
            Ty::Struct(fields) => fields.len_u8(),
            Ty::Tuple(fields) => fields.len_u8(),
            Ty::Array(_, _) => 1,
            Ty::UnicodeChar => 0,
            Ty::List(_, _) | Ty::Set(_, _) | Ty::Map(_, _, _) => 1,
        }
    }
}
