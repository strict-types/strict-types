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

use crate::dtl::{GravelTy, MonolithTy};
use crate::{Decode, DecodeError, Encode};

impl Encode for MonolithTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self {
            MonolithTy::Name(name) => {
                0u8.encode(writer)?;
                name.encode(writer)
            }
            MonolithTy::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
        }
    }
}

impl Decode for MonolithTy {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Decode::decode(reader).map(MonolithTy::Name),
            1u8 => Decode::decode(reader).map(Box::new).map(MonolithTy::Inline),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}

impl Encode for GravelTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self {
            GravelTy::Name(name) => {
                0u8.encode(writer)?;
                name.encode(writer)
            }
            GravelTy::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
            GravelTy::Extern(name, dep) => {
                2u8.encode(writer)?;
                name.encode(writer)?;
                dep.encode(writer)
            }
        }
    }
}

impl Decode for GravelTy {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Decode::decode(reader).map(GravelTy::Name),
            1u8 => Decode::decode(reader).map(Box::new).map(GravelTy::Inline),
            2u8 => Ok(GravelTy::Extern(Decode::decode(reader)?, Decode::decode(reader)?)),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}
