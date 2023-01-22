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

use std::fmt::{self, Alignment, Display, Formatter};

use amplify::confinement::TinyVec;
use base58::ToBase58;
use strict_encoding::Ident;

use crate::typelib::TypeLibId;
use crate::SemId;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Baid58Case {
    Pascal,
    Kebab,
    Snake,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Baid58<const LEN: usize> {
    hrp: &'static str,
    payload: [u8; LEN],
    checksum: u32,
}

impl<const LEN: usize> Baid58<LEN> {
    pub fn with(hrp: &'static str, payload: [u8; LEN]) -> Self {
        let key = blake3::Hasher::new().update(hrp.as_bytes()).finalize();
        let mut hasher = blake3::Hasher::new_keyed(key.as_bytes());
        hasher.update(&payload);
        let hash = *hasher.finalize().as_bytes();
        let checksum = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        Self {
            hrp,
            payload,
            checksum,
        }
    }

    pub fn checksum(&self) -> u32 { self.checksum }

    pub fn mnemonic(&self) -> String { self.mnemonic_with_case(Baid58Case::Kebab) }

    pub fn mnemonic_with_case(&self, case: Baid58Case) -> String {
        let mn = mnemonic::to_string(self.checksum.to_le_bytes());
        match case {
            Baid58Case::Pascal => {
                let mut res = String::with_capacity(mn.len());
                for s in mn.split('-') {
                    res.push_str((s[0..1].to_uppercase() + &s[1..]).as_str());
                }
                res
            }
            Baid58Case::Kebab => mn,
            Baid58Case::Snake => mn.replace('-', "_"),
        }
    }
}

/// Formatting flags:
/// - Nothing: do not add HRP and mnemonic
/// - `#` - add kebab-case mnemonic as a suffix separated with `#`;
/// - `0` - prefix with capitalized mnemonic separated with zero from the main code;
/// - `-` - prefix with dashed separated mnemonic;
/// - `+` - prefix with underscore separated mnemonic;
/// - `.*` - suffix the string with HRP as a file extension;
/// - `<` - prefix the string with HRP. Requires mnemonic prefix flag or defaults it to `0`.
///   Separates from the mnemonic using fill character and width;
/// - `^` - prefix with HRP without mnemonic, using fill character as separator or defaulting to
///   `_^` otherwise, width value implies number of character replications;
/// - `>` - suffix the string with HRP, using fill character as a separator. If width is given, use
///   multiple fill characters up to a width.
impl<const LEN: usize> Display for Baid58<LEN> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
        enum Mnemo {
            None,
            Prefix(Baid58Case),
            Suffix,
        }
        #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
        enum Hrp {
            None,
            Prefix(String),
            Suffix(String),
            Ext,
        }

        let mut mnemo = if f.alternate() {
            Mnemo::Suffix
        } else if f.sign_aware_zero_pad() {
            Mnemo::Prefix(Baid58Case::Pascal)
        } else if f.sign_minus() {
            Mnemo::Prefix(Baid58Case::Kebab)
        } else if f.sign_plus() {
            Mnemo::Prefix(Baid58Case::Snake)
        } else {
            Mnemo::None
        };

        let fill = (0..=f.width().unwrap_or_default()).map(|_| f.fill()).collect();
        let hrp = match f.align() {
            None if f.precision().is_some() => Hrp::Ext,
            None => Hrp::None,
            Some(Alignment::Left) if mnemo == Mnemo::None => {
                mnemo = Mnemo::Prefix(Baid58Case::Pascal);
                Hrp::Prefix(fill)
            }
            Some(Alignment::Left) | Some(Alignment::Center) => Hrp::Prefix(fill),
            Some(Alignment::Right) => Hrp::Suffix(fill),
        };

        if let Hrp::Prefix(ref prefix) = hrp {
            f.write_str(&self.hrp)?;
            f.write_str(prefix)?;
        }

        if let Mnemo::Prefix(prefix) = mnemo {
            f.write_str(&self.mnemonic_with_case(prefix))?;
            match prefix {
                Baid58Case::Pascal => f.write_str("0")?,
                Baid58Case::Kebab => f.write_str("-")?,
                Baid58Case::Snake => f.write_str("_")?,
            }
        }

        f.write_str(&self.payload.to_base58())?;

        if let Mnemo::Suffix = mnemo {
            write!(f, "#{}", &self.mnemonic_with_case(Baid58Case::Kebab))?;
        }

        if let Hrp::Suffix(ref suffix) = hrp {
            f.write_str(suffix)?;
            f.write_str(&self.hrp)?;
        } else if let Hrp::Ext = hrp {
            write!(f, ".{}", self.hrp)?;
        }

        Ok(())
    }
}

pub trait ToBaid58<const LEN: usize>: Display /* TODO: + FromStr */ {
    const HRP: &'static str;
    // TODO: Uncomment once generic_const_exprs is out
    // const LEN: usize;

    fn to_baid58_payload(&self) -> [u8; LEN];
    fn to_baid58(&self) -> Baid58<LEN> { Baid58::with(Self::HRP, self.to_baid58_payload()) }
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
#[display(inner)]
pub enum PreFragment {
    #[from]
    Ident(Ident),
    #[from]
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
    #[display("urn:sten:lib:{0}", alt = "urn:sten:lib:{0:#}")]
    Lib(TypeLibId),

    #[from]
    #[display("urn:sten:id:{0}", alt = "urn:sten:id:{0:#}")]
    Type(SemId),
}
