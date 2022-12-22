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

use std::fmt::{self, Display, Formatter};

use crate::{LibAlias, SemId, Ty, TypeName, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum NestedRef {
    Inline(Box<Ty<NestedRef>>),
    Named(TypeName, SemId),
    Extern(TypeName, LibAlias, SemId),
}

impl TypeRef for NestedRef {
    fn id(&self) -> SemId {
        match self {
            NestedRef::Inline(ty) => ty.id(None),
            NestedRef::Named(_, id) | NestedRef::Extern(_, _, id) => *id,
        }
    }
}

impl Display for NestedRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NestedRef::Inline(ty) => Display::fmt(ty, f),
            NestedRef::Named(name, _) => write!(f, "{}", name),
            NestedRef::Extern(name, lib, _) => write!(f, "{}.{}", lib, name),
        }
    }
}
