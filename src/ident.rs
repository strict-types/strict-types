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

use amplify::ascii::{AsAsciiStrError, AsciiChar, AsciiString, FromAsciiError};
use amplify::confinement;
use amplify::confinement::Confined;

use crate::SemId;

#[macro_export]
macro_rules! tn {
    ($name:literal) => {
        $crate::TypeName::from($name).into()
    };
    ($name:ident) => {
        $crate::TypeName::from($name).into()
    };
    ($name:literal, $($arg:expr),+) => {
        $crate::TypeName::try_from(format!($name, $($arg),+))
            .expect("invalid type name from formatter")
            .into()
    };
}

#[macro_export]
macro_rules! fname {
    ($name:literal) => {
        $crate::FieldName::from($name).into()
    };
    ($name:ident) => {
        $crate::FieldName::from($name).into()
    };
}

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

impl<O> From<FromAsciiError<O>> for InvalidIdent {
    fn from(_: FromAsciiError<O>) -> Self { InvalidIdent::NonAsciiChar }
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
// TODO: Use alphanumeric filter instead; promote type to the amplify library
pub struct Ident(Confined<AsciiString, 1, 32>);

impl From<&'static str> for Ident {
    fn from(s: &'static str) -> Self {
        let ascii = AsciiString::from_ascii(s).expect("invalid identifier name");
        Ident::try_from(ascii).expect("invalid identifier name")
    }
}

impl From<SemId> for Ident {
    fn from(id: SemId) -> Self {
        let mut s = s!("Auto");
        s.extend(id.to_hex()[..8].to_uppercase().chars().take(8));
        let s = AsciiString::from_ascii(s).expect("invalid identifier name");
        Ident::try_from(s).expect("invalid identifier name")
    }
}

impl TryFrom<String> for Ident {
    type Error = InvalidIdent;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let s = AsciiString::from_ascii(s.as_bytes())?;
        Ident::try_from(s)
    }
}

impl TryFrom<AsciiString> for Ident {
    type Error = InvalidIdent;

    fn try_from(ascii: AsciiString) -> Result<Self, InvalidIdent> {
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
