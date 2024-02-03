// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2024 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2024 UBIDECO Institute
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

use std::fmt;
use std::fmt::{Display, Formatter};
use std::mem::swap;

use amplify::confinement::{Confined, TinyVec};
use strict_encoding::STRICT_TYPES_LIB;

use crate::ast::ItemCase;
use crate::layout::TypeLayout;
use crate::typesys::TypeFqn;
use crate::{ast, SemId, SymbolicSys, Ty};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TypeTree<'sys> {
    sem_id: SemId,
    sys: &'sys SymbolicSys,
}

impl<'sys> TypeTree<'sys> {
    pub(super) fn new(sem_id: SemId, sys: &'sys SymbolicSys) -> Self { Self { sem_id, sys } }

    pub fn get(&self) -> &Ty<SemId> { self.sys.get(self.sem_id).expect("inconsistent type tree") }

    pub fn iter(&'sys self) -> TypeTreeIter<'sys> {
        TypeTreeIter {
            sem_id: self.sem_id,
            ty: Some(self.get()),
            item: None,
            depth: 0,
            path: vec![],
            sys: &self.sys,
            nested: vec![],
        }
    }

    #[inline]
    pub fn to_layout(&self) -> TypeLayout { TypeLayout::from(self) }
}

/*
impl<'sys> IntoIterator for TypeTree<'sys> {
    type Item = (usize, &'sys Ty<SemId>, Option<&'sys TypeFqn>);
    type IntoIter = TypeTreeIter<'sys>;

    fn into_iter(self) -> Self::IntoIter { self.iter() }
}
 */

impl<'tree, 'sys> IntoIterator for &'tree TypeTree<'sys>
where 'tree: 'sys
{
    type Item = TypeInfo;
    type IntoIter = TypeTreeIter<'sys>;

    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl Display for TypeTree<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Display::fmt(&self.to_layout(), f) }
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom)]
pub enum NestedCase {
    #[strict_type(tag = 0x0)]
    NewType(Option<TypeFqn>),

    #[strict_type(tag = 0x1, dumb)]
    Option,

    #[strict_type(tag = 0x10)]
    ByteStr,

    #[strict_type(tag = 0x11)]
    AsciiStr(Option<TypeFqn>),

    #[strict_type(tag = 0x12)]
    UniStr,
}

/*
pub struct NestedInfo<'sys> {
    pub inner: Option<&'sys TypeFqn>,
    pub case: NestedCase,
}
 */

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct TypeInfo {
    pub depth: u32,
    pub ty: Ty<SemId>,
    pub fqn: Option<TypeFqn>,
    pub item: Option<ItemCase>,
    pub nested: TinyVec<NestedCase>,
}

/*
impl<'sys> TypeInfo<'sys> {
    pub fn with(depth: usize, ty: &'sys Ty<SemId>, fqn: Option<&'sys TypeFqn>) -> Self {
        Self {
            depth,
            ty,
            fqn,
            wrapped: false,
            optional: false,
        }
    }
}
 */

pub struct TypeTreeIter<'sys> {
    sem_id: SemId,
    ty: Option<&'sys Ty<SemId>>,
    item: Option<ItemCase>,
    depth: u32,
    path: Vec<(u32, ast::Iter<'sys, SemId>)>,
    sys: &'sys SymbolicSys,
    nested: Vec<NestedCase>,
}

impl<'sys> Iterator for TypeTreeIter<'sys> {
    type Item = TypeInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ty) = self.ty {
            let fqn = self.sys.symbols.lookup(self.sem_id);
            self.ty = None;

            let mut nested = vec![];
            let mut dive = true;
            let mut push = true;
            let mut ret = true;
            let mut iter = ty.iter();

            if ty.is_newtype() {
                nested.push(NestedCase::NewType(fqn.cloned()));
                dive = false;
                ret = false;
            } else if ty.is_option() {
                nested.push(NestedCase::Option);
                let _ = iter.next(); // skipping none
                ret = false;
            } else if let Ty::List(inner_id, _) = ty {
                let inner_ty = self.sys.get(*inner_id).expect("incomplete type system");
                if inner_ty.is_char_enum() {
                    let fqn = self.sys.symbols.lookup(*inner_id);
                    nested.push(NestedCase::AsciiStr(fqn.cloned()));
                } else if inner_ty.is_byte() {
                    nested.push(NestedCase::ByteStr);
                } else if inner_ty.is_unicode_char() {
                    nested.push(NestedCase::UniStr);
                }
                if !nested.is_empty() {
                    push = false;
                    dive = false;
                }
            } else if ty.is_byte_array() {
                push = false;
            }

            let depth = self.depth;
            if dive {
                self.depth += 1;
            }
            if push {
                self.path.push((self.depth, iter));
            }
            self.nested.extend(nested);
            if ret {
                let mut item = None;
                swap(&mut item, &mut self.item);
                let info = TypeInfo {
                    depth,
                    ty: ty.clone(),
                    fqn: fqn.cloned(),
                    item,
                    nested: Confined::from_collection_unsafe(self.nested.clone()),
                };
                self.nested = vec![];
                return Some(info);
            }
        }
        loop {
            let (depth, iter) = self.path.last_mut()?;
            self.depth = *depth;
            match iter.next() {
                None => {
                    self.path.pop();
                    self.nested = vec![];
                    continue;
                }
                Some((id, item)) => {
                    self.sem_id = *id;
                    if !matches!(self.nested.last(), Some(NestedCase::NewType(_))) {
                        self.item = item;
                    }
                    self.ty = self.sys.get(*id);
                    return self.next();
                }
            }
        }
    }
}
