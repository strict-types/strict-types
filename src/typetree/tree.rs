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

use std::ops::Deref;

use encoding::{LibName, StrictDumb, TypeName, STRICT_TYPES_LIB};

use crate::ast::SemCommit;
use crate::{CommitConsume, Ty, TypeRef};

impl SemCommit for () {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) { hasher.commit_consume([]) }
}

impl TypeRef for () {}

#[derive(Clone, PartialEq, Eq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct TyInner {
    pub lib: LibName,
    pub name: Option<TypeName>,
    pub ty: Box<Ty<TyInner>>,
}

impl SemCommit for TyInner {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        if let Some(name) = &self.name {
            name.sem_commit(hasher);
        }
        self.ty.sem_commit(hasher)
    }
}

impl TypeRef for TyInner {
    fn as_ty(&self) -> Option<&Ty<Self>> { Some(self.ty.deref()) }

    fn is_compound(&self) -> bool { self.ty.is_compound() }
    fn is_byte(&self) -> bool { self.ty.is_byte() }
    fn is_unicode_char(&self) -> bool { self.ty.is_unicode_char() }
}

#[derive(Clone, PartialEq, Eq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct TypeTree {
    pub name: Option<TypeName>,
    pub ty: Ty<TyInner>,
}
