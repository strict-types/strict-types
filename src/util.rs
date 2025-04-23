// Strict encoding schema library, implementing validation and parsing of strict encoded data
// against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2019-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2024-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2022-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use std::env;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use amplify::confinement::TinyVec;
use strict_encoding::{Ident, STRICT_TYPES_LIB};

use crate::typelib::TypeLibId;
use crate::SemId;

#[derive(Clone, Eq, PartialEq, Debug, Display, Error)]
#[display("unknown name for the file format '{0}'")]
pub struct UnknownFormat(String);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
pub enum StlFormat {
    #[display("sty")]
    Source,
    #[display("stl")]
    Binary,
    #[cfg(feature = "armor")]
    #[display("sta")]
    Armored,
}

impl FromStr for StlFormat {
    type Err = UnknownFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stl" => Ok(StlFormat::Binary),
            #[cfg(feature = "armor")]
            "sta" => Ok(StlFormat::Armored),
            "sty" => Ok(StlFormat::Source),
            invalid => Err(UnknownFormat(invalid.to_owned())),
        }
    }
}

pub fn parse_args() -> (StlFormat, Option<String>) {
    let args: Vec<String> = env::args().collect();
    let ext = args.get(1).map(String::as_str).map(|s| s.trim_start_matches("--")).unwrap_or("sty");
    let format = StlFormat::from_str(ext).expect("unrecognized file format argument");
    let dir = match args.len() {
        1 => None,
        2 | 3 => Some(args.get(2).cloned().unwrap_or_else(|| s!("stl"))),
        _ => panic!("invalid argument count"),
    };
    (format, dir)
}

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

impl SemVer {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        SemVer {
            major,
            minor,
            patch,
            pre: none!(),
            build: none!(),
        }
    }
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
