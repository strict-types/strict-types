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

use std::iter::Sum;
use std::ops::{Add, AddAssign, Neg, Range};

use amplify::confinement::LargeVec;
use amplify::num::u24;
use strict_encoding::STRICT_TYPES_LIB;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display)]
#[display(lowercase)]
pub enum Compat {
    #[display("non-compatible")]
    NonCompatible,
    Alloset,
    Subset,
    Superset,
    Compatible,
}

impl Neg for Compat {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Compat::NonCompatible => Compat::NonCompatible,
            Compat::Subset => Compat::Superset,
            Compat::Superset => Compat::Subset,
            Compat::Alloset => Compat::Alloset,
            Compat::Compatible => Compat::Compatible,
        }
    }
}

impl Add for Compat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Compat::NonCompatible, _) | (_, Compat::NonCompatible) => Compat::NonCompatible,
            (Compat::Subset, Compat::Compatible | Compat::Subset)
            | (Compat::Compatible | Compat::Subset, Compat::Subset) => Compat::Subset,
            (Compat::Superset, Compat::Compatible | Compat::Superset)
            | (Compat::Compatible | Compat::Superset, Compat::Superset) => Compat::Superset,
            (Compat::Superset, Compat::Subset) | (Compat::Subset, Compat::Superset) => {
                Compat::Alloset
            }
            (Compat::Alloset, _) | (_, Compat::Alloset) => Compat::Alloset,
            (Compat::Compatible, Compat::Compatible) => Compat::Compatible,
        }
    }
}

impl AddAssign for Compat {
    fn add_assign(&mut self, rhs: Self) { *self = *self + rhs }
}

impl Sum for Compat {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Compat::Compatible, |acc, item| acc + item)
    }
}

pub trait Compatibility {
    fn compatibility(&self, other: &Self) -> Compat;
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct MemoryLayout(LargeVec<MemoryItem>);

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(StrictType, StrictEncode, StrictDecode, StrictDumb)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = repr, into_u8, try_from_u8)]
#[display(kebabcase)]
#[repr(u8)]
pub enum ItemsSorting {
    OrderedMap = 0x40,
    UnorderedMap = 0x4F,
    OrderedSet = 0x80,
    UnorderedSet = 0x8F,
    #[strict_type(dumb)]
    Unordered = 0xFF,
}

// TODO: Make use of this type throughout the library
#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictType, StrictEncode, StrictDecode, StrictDumb)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom, dumb = )]
pub struct LenRange<T: Number> {
    pub min: T,
    pub max: T,
}

// TODO: Make use of this type throughout the library
#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictType, StrictEncode, StrictDecode, StrictDumb)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom, dumb = {Self::U16(0..0xFFFF)})]
pub enum LenBounds {
    #[strict_type(tag = 1)]
    #[display("{0:X}@U8")]
    U8(Range<u8>),

    #[strict_type(tag = 2)]
    #[display("{0:X}@U16")]
    U16(Range<u16>),

    #[strict_type(tag = 3)]
    #[display("{0:X}@U24")]
    U24(Range<u24>),

    #[strict_type(tag = 4)]
    #[display("{0:X}@U32")]
    U32(Range<u32>),

    #[strict_type(tag = 8)]
    #[display("{0:X}@U64")]
    U64(Range<u64>),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictType, StrictEncode, StrictDecode, StrictDumb)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct AtomicItem {
    pub bytes: u16,
    pub restricted: bool,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictType, StrictEncode, StrictDecode, StrictDumb)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct CollectionItem {
    pub bounds: LenBounds,
    pub sorting: ItemsSorting,
    pub item: MemoryItem,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictType, StrictEncode, StrictDecode, StrictDumb)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = custom)]
pub enum MemoryItem {
    #[strict_type(tag = 0, dumb)]
    UnicodeChar,
    #[strict_type(tag = 1)]
    Atomic(AtomicItem),
    #[strict_type(tag = 0xFF)]
    Collection(Box<CollectionItem>),
}
