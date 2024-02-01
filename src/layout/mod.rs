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

use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use amplify::num::u24;
use encoding::Ident;
use vesper::{AttributeValue, Predicate};

use crate::typesys::{TypeInfo, TypeTree};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TypeLayout {
    items: Vec<TypeInfo>,
}

impl From<TypeTree<'_>> for TypeLayout {
    fn from(tree: TypeTree) -> Self {
        let mut layout = TypeLayout::new();
        layout.items.extend(&tree);
        layout
    }
}

impl<'a> From<&'a TypeTree<'_>> for TypeLayout {
    fn from(tree: &'a TypeTree) -> Self {
        let mut layout = TypeLayout::new();
        layout.items.extend(tree);
        layout
    }
}

impl TypeLayout {
    fn new() -> Self { Self { items: vec![] } }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(lowercase)]
pub enum Pred {
    /// Type alias
    As,
    Tuple,
    Rec,
    Enum,
    Union,
    // Composites
    Char,
    Str,
    Bytes,
    // Collections:
    Array,
    List,
    Set,
    Map,
}

impl Predicate for Pred {
    type AttrVal = AttrVal;
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
pub enum AttrVal {
    Ident(Ident),
    Int(u64),
    LenRange(LenRange),
}

impl AttributeValue for AttrVal {}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct LenRange(Range<u64>);

impl Display for LenRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match (self.0.start, self.0.end) {
            (min, max) if max == u8::MAX as u64 => write!(f, "{min}..MAX8"),
            (min, max) if max == u16::MAX as u64 => write!(f, "{min}..MAX16"),
            (min, max) if max == u24::MAX.into_u64() => write!(f, "{min}..MAX24"),
            (min, max) if max == u32::MAX as u64 => write!(f, "{min}..MAX32"),
            // TODO: Add more numbers
            (min, u64::MAX) => write!(f, "{min}..MAX64"),
            (min, max) => write!(f, "{min}..{max}"),
        }
    }
}
