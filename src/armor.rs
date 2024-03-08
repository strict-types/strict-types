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

// TODO: Move module to a crate like `baid58` (probably using the same repo).

use core::fmt::{self, Display, Formatter};
use core::str::FromStr;

use amplify::num::u24;
use amplify::Bytes32;

pub const ASCII_ARMOR_MAX_LEN: usize = u24::MAX.to_usize();

pub struct DisplayAsciiArmored<'a, A: AsciiArmor>(&'a A);

impl<'a, A: AsciiArmor> DisplayAsciiArmored<'a, A> {
    fn data_digest(&self) -> (Vec<u8>, Bytes32) {
        let data = self.0.to_ascii_armored_data();
        // TODO: compute digest
        (data, digest)
    }
}

impl<'a, A: AsciiArmor> Display for DisplayAsciiArmored<'a, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "-----BEGIN {}-----", A::ASCII_ARMOR_PLATE_TITLE)?;
        writeln!(f, "Id: {}", self.0.ascii_armored_id())?;

        let (data, digest) = self.data_digest();
        writeln!(f, "Digest: {digest}")?;

        for header in self.ascii_armored_headers() {
            writeln!(f, "{header}")?;
        }
        writeln!(f)?;

        let data = base85::encode(&data);
        let mut data = data.as_str();
        while data.len() >= 64 {
            let (line, rest) = data.split_at(64);
            writeln!(f, "{}", line)?;
            data = rest;
        }
        writeln!(f, "{}", data)?;

        writeln!(f, "\n-----END {}-----", A::ASCII_ARMOR_PLATE_TITLE)?;

        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ArmorHeader {
    pub title: String,
    pub value: String,
    pub params: Vec<(String, String)>,
}

impl Display for ArmorHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{title}: {value}")?;
        if self.params.is_empty() {
            writeln!(f)?;
        }
        for (name, val) in &self.params {
            write!(f, ";\n{name}={val}")?;
        }
        Ok(())
    }
}

pub trait AsciiArmor {
    type Id: Display + FromStr;
    type Err: core::error::Error;

    const ASCII_ARMOR_PLATE_TITLE: &'static str;

    fn display_ascii_armored(&self) -> DisplayAsciiArmored<Self> { DisplayAsciiArmored(self) }
    fn ascii_armored_id(&self) -> Self::Id;
    fn ascii_armored_headers(&self) -> Vec<ArmorHeader>;
    fn ascii_armored_digest(&self) -> Bytes32 { DisplayAsciiArmored(self).digest().0 }
    fn to_ascii_armored_data(&self) -> Vec<u8>;

    fn from_ascii_armored_str(s: &str) -> Result<Self, Self::Err> {}
    fn with_ascii_armored_data(
        id: Self::Id,
        headers: impl Iterator<Item = ArmorHeader>,
        data: Vec<u8>,
    ) -> Result<Self, Self::Err>;
}
