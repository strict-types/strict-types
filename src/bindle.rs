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

//! Bindle is a wrapper for ASCII armoring binary data containers, which can be serialized
//! and optionally signed by the creator with certain id and send over to a
//! remote party.

use std::collections::{btree_set, BTreeMap, BTreeSet};
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::str::FromStr;

use amplify::confinement;
use amplify::confinement::{Confined, TinyAscii, TinyBlob, TinyString, TinyVec, U24 as U24MAX};
use baid58::{Baid58ParseError, ToBaid58};
use strict_encoding::{
    StrictDecode, StrictDeserialize, StrictDumb, StrictEncode, StrictSerialize, StrictType,
    STRICT_TYPES_LIB,
};

#[cfg(feature = "fs")]
pub use self::_fs::*;
use crate::{SymbolicSys, TypeLib, TypeLibId, TypeSysId, TypeSystem};

pub const BINDLE_MAX_LEN: usize = U24MAX;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
#[display("{name} <{email}>; using={suite}")]
#[non_exhaustive]
pub struct Identity {
    pub name: TinyString,
    pub email: TinyAscii,
    pub suite: IdSuite,
    pub pk: TinyBlob,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = repr, into_u8, try_from_u8)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
#[non_exhaustive]
#[repr(u8)]
pub enum IdSuite {
    #[strict_type(dumb)]
    #[display("OpenPGP")]
    Pgp,
    #[display("OpenSSH")]
    Ssh,
    #[display("SSI")]
    Ssi,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct Cert {
    pub signer: Identity,
    pub signature: TinyBlob,
}

#[derive(Wrapper, WrapperMut, Clone, PartialEq, Eq, Hash, Debug, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, dumb = Self(confined_bset!(strict_dumb!())))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct ContentSigs(Confined<BTreeSet<Cert>, 1, 10>);

impl IntoIterator for ContentSigs {
    type Item = Cert;
    type IntoIter = btree_set::IntoIter<Cert>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

pub trait BindleContent: StrictSerialize + StrictDeserialize + StrictDumb {
    /// Magic bytes used in saving/restoring container from a file.
    const MAGIC: [u8; 4];
    /// String used in ASCII armored blocks
    const PLATE_TITLE: &'static str;

    type Id: Copy
        + Eq
        + Debug
        + Display
        + FromStr<Err = Baid58ParseError>
        + StrictType
        + StrictDumb
        + StrictEncode
        + StrictDecode;

    fn bindle_id(&self) -> Self::Id;
    fn bindle_headers(&self) -> BTreeMap<&'static str, String> { none!() }
    fn bindle(self) -> Bindle<Self> { Bindle::new(self) }
    fn bindle_mnemonic(&self) -> Option<String> { None }
}

impl BindleContent for TypeLib {
    const MAGIC: [u8; 4] = *b"USTL";
    const PLATE_TITLE: &'static str = "STRICT TYPE LIBRARY";
    type Id = TypeLibId;
    fn bindle_id(&self) -> Self::Id { self.id() }
}

impl BindleContent for TypeSystem {
    const MAGIC: [u8; 4] = *b"USTS";
    const PLATE_TITLE: &'static str = "STRICT TYPE SYSTEM";
    type Id = TypeSysId;
    fn bindle_id(&self) -> Self::Id { self.id() }
    fn bindle_mnemonic(&self) -> Option<String> { Some(self.id().to_baid58().mnemonic()) }
}

impl BindleContent for SymbolicSys {
    const MAGIC: [u8; 4] = *b"USSS";
    const PLATE_TITLE: &'static str = "STRICT SYMBOL SYSTEM";
    type Id = TypeSysId;
    fn bindle_id(&self) -> Self::Id { self.id() }
    fn bindle_mnemonic(&self) -> Option<String> { Some(self.id().to_baid58().mnemonic()) }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct Bindle<C: BindleContent> {
    id: C::Id,
    data: C,
    sigs: TinyVec<Cert>,
}

impl<C: BindleContent> Deref for Bindle<C> {
    type Target = C;
    fn deref(&self) -> &Self::Target { &self.data }
}

impl<C: BindleContent> From<C> for Bindle<C> {
    fn from(data: C) -> Self { Bindle::new(data) }
}

impl<C: BindleContent> Bindle<C> {
    pub fn new(data: C) -> Self {
        Bindle {
            id: data.bindle_id(),
            data,
            sigs: empty!(),
        }
    }

    pub fn id(&self) -> C::Id { self.id }

    pub fn into_split(self) -> (C, TinyVec<Cert>) { (self.data, self.sigs) }
    pub fn unbindle(self) -> C { self.data }
}

#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum BindleParseError<Id: Copy + Eq + Debug + Display> {
    /// the provided text doesn't represent a recognizable ASCII-armored RGB
    /// bindle encoding.
    WrongStructure,

    /// Id header of the bindle contains unparsable information. Details: {0}
    InvalidId(Baid58ParseError),

    /// the actual data doesn't match the provided id.
    ///
    /// Actual id: {actual}.
    ///
    /// Expected id: {expected}.
    MismatchedId { actual: Id, expected: Id },

    /// bindle data has invalid Base85 encoding (ASCII armoring).
    #[from(base85::Error)]
    Base85,

    /// unable to decode the provided bindle data. Details: {0}
    #[from]
    Deserialize(strict_encoding::DeserializeError),

    /// bindle contains more than 16MB of data.
    #[from(confinement::Error)]
    TooLarge,
}

impl<C: BindleContent> FromStr for Bindle<C> {
    type Err = BindleParseError<C::Id>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let first = format!("-----BEGIN {}-----", C::PLATE_TITLE);
        let last = format!("-----END {}-----", C::PLATE_TITLE);
        if (lines.next(), lines.next_back()) != (Some(&first), Some(&last)) {
            return Err(BindleParseError::WrongStructure);
        }
        let mut header_id = None;
        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }
            if let Some(id_str) = line.strip_prefix("Id: ") {
                header_id = Some(C::Id::from_str(id_str).map_err(BindleParseError::InvalidId)?);
            }
        }
        let armor = lines.filter(|l| !l.is_empty()).collect::<String>();
        let data = base85::decode(&armor)?;
        let data = C::from_strict_serialized::<BINDLE_MAX_LEN>(Confined::try_from(data)?)?;
        let id = data.bindle_id();
        if let Some(header_id) = header_id {
            if header_id != id {
                return Err(BindleParseError::MismatchedId {
                    actual: id,
                    expected: header_id,
                });
            }
        }
        // TODO: check mnemonic
        // TODO: parse and validate sigs
        Ok(Self {
            id,
            data,
            sigs: none!(),
        })
    }
}

impl<C: BindleContent> Display for Bindle<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "-----BEGIN {}-----", C::PLATE_TITLE)?;
        writeln!(f, "Id: {:-#}", self.id)?;
        if let Some(mnemonic) = self.bindle_mnemonic() {
            writeln!(f, "Mnemonic: {}", mnemonic)?;
        }
        for (header, value) in self.bindle_headers() {
            writeln!(f, "{header}: {value}")?;
        }
        for cert in &self.sigs {
            writeln!(f, "Signed-By: {}", cert.signer)?;
        }
        writeln!(f)?;

        // TODO: Replace with streamed writer
        let data = self.data.to_strict_serialized::<BINDLE_MAX_LEN>().expect("in-memory");
        let data = base85::encode(&data);
        let mut data = data.as_str();
        while data.len() >= 64 {
            let (line, rest) = data.split_at(64);
            writeln!(f, "{}", line)?;
            data = rest;
        }
        writeln!(f, "{}", data)?;

        writeln!(f, "\n-----END {}-----", C::PLATE_TITLE)?;
        Ok(())
    }
}

#[cfg(feature = "fs")]
mod _fs {
    use std::io::{Read, Write};
    use std::path::Path;
    use std::{fs, io};

    use encoding::{StreamReader, StreamWriter};
    use strict_encoding::DecodeError;

    use super::*;

    #[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
    #[display(doc_comments)]
    pub enum LoadError {
        /// invalid file data.
        InvalidMagic,

        #[display(inner)]
        #[from]
        #[from(io::Error)]
        Decode(DecodeError),
    }

    impl<C: BindleContent> Bindle<C> {
        pub fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
            let mut magic = [0u8; 4];
            let mut file = fs::File::open(path)?;
            file.read_exact(&mut magic)?;
            if magic != C::MAGIC {
                return Err(LoadError::InvalidMagic);
            }
            let reader = StreamReader::new::<BINDLE_MAX_LEN>(file);
            let me = Self::strict_read(reader)?;
            Ok(me)
        }

        pub fn save(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
            let mut file = fs::File::create(path)?;
            file.write_all(&C::MAGIC)?;
            let writer = StreamWriter::new::<BINDLE_MAX_LEN>(file);
            self.strict_write(writer)?;
            Ok(())
        }
    }
}
