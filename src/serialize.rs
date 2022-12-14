// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Project.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.

use std::io::{Error, Read, Write};
use std::{fs, io};

use amplify::ascii::AsciiString;
use amplify::confinement::{MediumVec, TinyVec};
use amplify::num::u24;
use amplify::{confinement, IoError, WriteCounter};

use crate::util::{BuildFragment, InvalidIdent, PreFragment, Sizing};
use crate::{FieldName, Ident, SemVer};

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum DecodeError {
    #[display(inner)]
    #[from(io::Error)]
    Io(IoError),

    /// unknown type class value {0}
    WrongTyCls(u8),

    /// invalid type class {0} for map keys
    InvalidTyCls(Cls),

    /// unknown variant id {0} for inline type reference
    WrongRef(u8),

    /// confinement requirements are not satisfied. Specifically, {0}
    #[from]
    Confinement(confinement::Error),

    #[display(inner)]
    #[from]
    InvalidIdent(InvalidIdent),

    /// invalid value {1} for {0} enum
    WrongEnumId(&'static str, u8),
}

pub trait Deserialize: Decode {
    fn from_serialized(ast_data: MediumVec<u8>) -> Result<Self, DecodeError> {
        let mut cursor = io::Cursor::new(ast_data.into_inner());
        Self::decode(&mut cursor)
    }

    fn deserialize_from_file(path: impl AsRef<std::path::Path>) -> Result<Self, DecodeError> {
        let mut file = fs::File::open(path)?;
        Self::decode(&mut file)
    }
}

pub trait Serialize: Encode {
    fn serialized_len(&self) -> usize {
        let mut counter = WriteCounter::default();
        self.encode(&mut counter).expect("counter doesn't error");
        counter.count
    }

    fn to_serialized(&self) -> MediumVec<u8> {
        let len = self.serialized_len();
        debug_assert!(
            len <= u24::MAX.into_usize(),
            "Ty type guarantees on the data size are broken"
        );
        let mut ast_data = Vec::with_capacity(len);
        self.encode(&mut ast_data).expect("memory writers do not error");
        MediumVec::try_from(ast_data).expect("Ty type guarantees on the data size are broken")
    }

    fn serialize_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), io::Error> {
        let mut file = fs::File::create(path)?;
        self.encode(&mut file)
    }
}

pub trait Encode {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error>;
}

pub trait Decode: Sized {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError>;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
#[display(lowercase)]
#[repr(u8)]
pub enum Cls {
    Primitive = 0,
    Enum = 1,
    Union = 2,
    Struct = 3,
    Array = 4,
    Unicode = 5,
    List = 6,
    Set = 7,
    Map = 8,
}

impl Cls {
    pub const ALL: [Cls; 9] = [
        Cls::Primitive,
        Cls::Enum,
        Cls::Union,
        Cls::Struct,
        Cls::Array,
        Cls::Unicode,
        Cls::List,
        Cls::Set,
        Cls::Map,
    ];
}

impl TryFrom<u8> for Cls {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        for cls in Cls::ALL {
            if cls as u8 == value {
                return Ok(cls);
            }
        }
        return Err(value);
    }
}

impl Encode for Cls {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), Error> {
        writer.write_all(&[*self as u8])
    }
}

impl Decode for Cls {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Cls::try_from(buf[0]).map_err(DecodeError::WrongTyCls)
    }
}

impl Encode for u8 {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), Error> {
        writer.write_all(&[*self])
    }
}

impl Decode for u8 {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Encode for u16 {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), Error> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Decode for u16 {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
}

impl Encode for u24 {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), Error> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Decode for u24 {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf)?;
        Ok(u24::from_le_bytes(buf))
    }
}

impl Encode for u128 {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), Error> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Decode for u128 {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 16];
        reader.read_exact(&mut buf)?;
        Ok(u128::from_le_bytes(buf))
    }
}

impl Encode for FieldName {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write_all(self.as_bytes())
    }
}

impl Decode for FieldName {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let len = u8::decode(reader)?;
        let mut bytes = vec![0u8; len as usize];
        reader.read_exact(&mut bytes)?;
        let ascii = AsciiString::from_ascii(bytes)
            .map_err(|err| err.ascii_error())
            .map_err(InvalidIdent::from)?;
        FieldName::try_from(ascii).map_err(DecodeError::from)
    }
}

impl Encode for Sizing {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.min.encode(writer)?;
        self.max.encode(writer)
    }
}

impl Decode for Sizing {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        Ok(Sizing::new(Decode::decode(reader)?, Decode::decode(reader)?))
    }
}

impl Encode for SemVer {
    fn encode(&self, writer: &mut impl Write) -> Result<(), Error> {
        self.major.encode(writer)?;
        self.minor.encode(writer)?;
        self.patch.encode(writer)?;
        self.pre.len_u8().encode(writer)?;
        for fragment in &self.pre {
            fragment.encode(writer)?;
        }
        self.build.len_u8().encode(writer)?;
        for fragment in &self.build {
            fragment.encode(writer)?;
        }
        Ok(())
    }
}

impl Decode for SemVer {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let major = u16::decode(reader)?;
        let minor = u16::decode(reader)?;
        let patch = u16::decode(reader)?;
        let len = u8::decode(reader)?;
        let mut pre = TinyVec::with_capacity(len as usize);
        for _ in 0..len {
            pre.push(PreFragment::decode(reader)?).expect("len is less than u8::MAX");
        }
        let len = u8::decode(reader)?;
        let mut build = TinyVec::with_capacity(len as usize);
        for _ in 0..len {
            build.push(BuildFragment::decode(reader)?).expect("len is less than u8::MAX");
        }
        Ok(SemVer {
            major,
            minor,
            patch,
            build,
            pre,
        })
    }
}

impl Encode for PreFragment {
    fn encode(&self, writer: &mut impl Write) -> Result<(), Error> {
        match self {
            PreFragment::Ident(ident) => {
                0u8.encode(writer)?;
                ident.encode(writer)
            }
            PreFragment::Digits(dig) => {
                1u8.encode(writer)?;
                dig.encode(writer)
            }
        }
    }
}

impl Decode for PreFragment {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Ident::decode(reader).map(PreFragment::Ident),
            1u8 => u128::decode(reader).map(PreFragment::Digits),
            wrong => Err(DecodeError::WrongEnumId("PreFragment", wrong)),
        }
    }
}

impl Encode for BuildFragment {
    fn encode(&self, writer: &mut impl Write) -> Result<(), Error> {
        match self {
            BuildFragment::Ident(ident) => {
                0u8.encode(writer)?;
                ident.encode(writer)
            }
            BuildFragment::Digits(ident) => {
                1u8.encode(writer)?;
                ident.encode(writer)
            }
        }
    }
}

impl Decode for BuildFragment {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Ident::decode(reader).map(BuildFragment::Ident),
            1u8 => Ident::decode(reader).map(BuildFragment::Digits),
            wrong => Err(DecodeError::WrongEnumId("BuildFragment", wrong)),
        }
    }
}
