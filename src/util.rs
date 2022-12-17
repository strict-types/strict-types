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

use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign};

use amplify::ascii::{AsAsciiStrError, AsciiChar, AsciiString};
use amplify::confinement;
use amplify::confinement::{Confined, TinyVec};

use crate::dtl::TypeLibId;
use crate::TyId;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum InvalidIdent {
    /// ident must contain at least one character
    Empty,

    /// identifier name must start with alphabetic character and not `{0}`
    NonAlphabetic(AsciiChar),

    /// identifier name contains invalid character `{0}`
    InvalidChar(AsciiChar),

    #[from(AsAsciiStrError)]
    /// identifier name contains non-ASCII character(s)
    NonAsciiChar,

    /// identifier name has invalid length
    #[from]
    Confinement(confinement::Error),
}

/// Identifier (field or type name).
#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, Display)]
#[wrapper_mut(DerefMut)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Ident(Confined<AsciiString, 1, 32>);

impl From<&'static str> for Ident {
    fn from(s: &'static str) -> Self {
        let ascii = AsciiString::from_ascii(s).expect("invalid identifier name");
        Ident::try_from(ascii).expect("invalid identifier name")
    }
}

impl From<TyId> for Ident {
    fn from(id: TyId) -> Self {
        let mut s = s!("Auto");
        s.extend(id.to_hex()[..8].to_uppercase().chars().take(8));
        let s = AsciiString::from_ascii(s).expect("invalid identifier name");
        Ident::try_from(s).expect("invalid identifier name")
    }
}

impl Ident {
    pub fn try_from(ascii: AsciiString) -> Result<Self, InvalidIdent> {
        if ascii.is_empty() {
            return Err(InvalidIdent::Empty);
        }
        let first = ascii[0];
        if !first.is_alphabetic() {
            return Err(InvalidIdent::NonAlphabetic(first));
        }
        if let Some(ch) =
            ascii.as_slice().iter().copied().find(|ch| !ch.is_ascii_alphanumeric() && *ch != b'_')
        {
            return Err(InvalidIdent::InvalidChar(ch));
        }
        let s = Confined::try_from(ascii)?;
        Ok(Self(s))
    }
}

pub type TypeName = Ident;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Sizing {
    pub min: u16,
    pub max: u16,
}

impl Sizing {
    pub const U8: Sizing = Sizing {
        min: 0,
        max: u8::MAX as u16,
    };

    pub const U16: Sizing = Sizing {
        min: 0,
        max: u16::MAX,
    };

    pub const U8_NONEMPTY: Sizing = Sizing {
        min: 1,
        max: u8::MAX as u16,
    };

    pub const fn new(min: u16, max: u16) -> Self { Sizing { min, max } }
}

impl Display for Sizing {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (0, u16::MAX) => Ok(()),
            (min, u16::MAX) => write!(f, " ^ {}..", min),
            (min, max) => write!(f, " ^ {}..{:#04x}", min, max),
        }
    }
}

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
#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
pub enum PreFragment {
    Ident(Ident),
    Digits(u128),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
pub enum BuildFragment {
    Ident(Ident),
    Digits(Ident),
}

// TODO: Manually implement Ord, PartialOrd
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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
    #[display("urn:ubideco:sten:lib:{0}", alt = "urn:ubideco:sten:lib:{0:#}")]
    Lib(TypeLibId),

    #[from]
    #[display("urn:ubideco:sten:{0}", alt = "urn:ubideco:sten:{0:#}")]
    Type(TyId),
}
