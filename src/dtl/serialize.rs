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

use std::io;

use crate::dtl::{EmbeddedTy, LibTy};
use crate::{Decode, DecodeError, Encode};

impl Encode for EmbeddedTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self {
            EmbeddedTy::Name(name) => {
                0u8.encode(writer)?;
                name.encode(writer)
            }
            EmbeddedTy::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
        }
    }
}

impl Decode for EmbeddedTy {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Decode::decode(reader).map(EmbeddedTy::Name),
            1u8 => Decode::decode(reader).map(Box::new).map(EmbeddedTy::Inline),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}

impl Encode for LibTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self {
            LibTy::Named(name) => {
                0u8.encode(writer)?;
                name.encode(writer)
            }
            LibTy::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
            LibTy::Extern(name, dep) => {
                2u8.encode(writer)?;
                name.encode(writer)?;
                dep.encode(writer)
            }
        }
    }
}

impl Decode for LibTy {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Decode::decode(reader).map(LibTy::Named),
            1u8 => Decode::decode(reader).map(Box::new).map(LibTy::Inline),
            2u8 => Ok(LibTy::Extern(Decode::decode(reader)?, Decode::decode(reader)?)),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}
