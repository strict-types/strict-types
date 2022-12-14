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

use std::collections::BTreeMap;
use std::io;

use amplify::num::u24;

use crate::dtl::{EmbeddedTy, LibTy, TypeSystem};
use crate::{Decode, DecodeError, Deserialize, Encode, Serialize, Ty, TyId};

// TODO: Serialize TypeLib

impl Serialize for TypeSystem {}
impl Deserialize for TypeSystem {}

impl Encode for TypeSystem {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.count_types().encode(writer)?;
        for (id, ty) in self.iter() {
            id.encode(writer)?;
            ty.encode(writer)?;
        }
        Ok(())
    }
}

impl Decode for TypeSystem {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        let count = u24::decode(reader)?;
        let mut lib: BTreeMap<TyId, Ty<EmbeddedTy>> = empty!();
        for _ in 0..count.into_usize() {
            lib.insert(Decode::decode(reader)?, Decode::decode(reader)?);
        }
        TypeSystem::try_from_iter(lib).map_err(DecodeError::from)
    }
}

impl Encode for EmbeddedTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self {
            EmbeddedTy::Name(lib_name, ty_name, id) => {
                0u8.encode(writer)?;
                lib_name.encode(writer)?;
                ty_name.encode(writer)?;
                id.encode(writer)
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
            0u8 => Ok(EmbeddedTy::Name(
                Decode::decode(reader)?,
                Decode::decode(reader)?,
                Decode::decode(reader)?,
            )),
            1u8 => Decode::decode(reader).map(Box::new).map(EmbeddedTy::Inline),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}

impl Encode for LibTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self {
            LibTy::Named(name, id) => {
                0u8.encode(writer)?;
                name.encode(writer)?;
                id.encode(writer)
            }
            LibTy::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
            LibTy::Extern(name, lib_alias, id) => {
                2u8.encode(writer)?;
                name.encode(writer)?;
                lib_alias.encode(writer)?;
                id.encode(writer)
            }
        }
    }
}

impl Decode for LibTy {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Ok(LibTy::Named(Decode::decode(reader)?, Decode::decode(reader)?)),
            1u8 => Decode::decode(reader).map(Box::new).map(LibTy::Inline),
            2u8 => Ok(LibTy::Extern(
                Decode::decode(reader)?,
                Decode::decode(reader)?,
                Decode::decode(reader)?,
            )),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}
