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

use std::fmt::{self, Display, Formatter};

use amplify::confinement::TinyVec;
use strict_encoding::{Ident, STRICT_TYPES_LIB};

use crate::typelib::TypeLibId;
use crate::SemId;

/* TODO: Move into layout mod
/// Measure of a type size in bytes
#[derive(Copy, Clone, PartialEq, Eq, Debug, Display)]
pub enum Size {
    /// Type has a fixed size known at compile time
    #[display(inner)]
    Fixed(u16),

    /// Type has variable size
    #[display("variable")]
    Variable,
}

impl PartialOrd for Size {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Size {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Size::Variable, Size::Variable) => Ordering::Equal,
            (Size::Variable, _) => Ordering::Greater,
            (_, Size::Variable) => Ordering::Less,
            (Size::Fixed(a), Size::Fixed(b)) => a.cmp(b),
        }
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Size::Fixed(a), Size::Fixed(b)) => Size::Fixed(a + b),
            _ => Size::Variable,
        }
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
}

impl Sum for Size {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Size::Fixed(0);
        for item in iter {
            acc += item;
        }
        acc
    }
}
 */

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { PreFragment::Digits(1) })]
#[display(inner)]
pub enum PreFragment {
    #[from]
    Ident(Ident),
    #[from]
    Digits(u128),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { BuildFragment::Ident(Ident::from("alpha")) })]
#[display(inner)]
pub enum BuildFragment {
    Ident(Ident),
    Digits(Ident),
}

// TODO: Manually implement Ord, PartialOrd
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub pre: TinyVec<PreFragment>,
    pub build: TinyVec<BuildFragment>,
}

impl Display for SemVer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if !self.build.is_empty() {
            f.write_str("-")?;
        }
        let mut len = self.build.len();
        for item in &self.build {
            Display::fmt(item, f)?;
            len -= 1;
            if len > 0 {
                f.write_str(".")?;
            }
        }

        if !self.pre.is_empty() {
            f.write_str("+")?;
        }
        let mut len = self.pre.len();
        for item in &self.pre {
            Display::fmt(item, f)?;
            len -= 1;
            if len > 0 {
                f.write_str(".")?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
pub enum Urn {
    #[from]
    #[display("urn:sten:lib:{0}", alt = "urn:sten:lib:{0:#}")]
    Lib(TypeLibId),

    #[from]
    #[display("urn:sten:id:{0}", alt = "urn:sten:id:{0:#}")]
    Type(SemId),
}
