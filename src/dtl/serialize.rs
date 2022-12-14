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
use std::io::{Error, Read, Write};

use amplify::confinement::{Confined, TinyOrdMap};
use amplify::num::u24;
use amplify::Wrapper;

use crate::dtl::type_lib::Dependency;
use crate::dtl::{EmbeddedTy, LibName, LibTy, TypeLib, TypeLibId, TypeSystem};
use crate::{Decode, DecodeError, Deserialize, Encode, SemVer, Serialize, Ty, TyId, TypeName};

impl Serialize for TypeSystem {}
impl Deserialize for TypeSystem {}

impl Encode for TypeSystem {
    fn encode(&self, writer: &mut impl Write) -> Result<(), io::Error> {
        self.count_types().encode(writer)?;
        for ty in self.values() {
            ty.encode(writer)?;
        }
        Ok(())
    }
}

impl Decode for TypeSystem {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let count = u24::decode(reader)?;
        let mut lib: BTreeMap<TyId, Ty<EmbeddedTy>> = empty!();
        for _ in 0..count.into_usize() {
            let ty = Ty::decode(reader)?;
            lib.insert(ty.id(), ty);
        }
        TypeSystem::try_from_iter(lib).map_err(DecodeError::from)
    }
}

impl Serialize for TypeLib {}
impl Deserialize for TypeLib {}

impl Encode for TypeLib {
    fn encode(&self, writer: &mut impl Write) -> Result<(), Error> {
        self.name.encode(writer)?;

        self.dependencies.len_u8().encode(writer)?;
        for (alias, dep) in &self.dependencies {
            alias.encode(writer)?;
            dep.encode(writer)?;
        }

        self.types.len_u16().encode(writer)?;
        for (name, ty) in &self.types {
            name.encode(writer)?;
            ty.encode(writer)?
        }
        Ok(())
    }
}

impl Decode for TypeLib {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let name = LibName::decode(reader)?;

        let len = u8::decode(reader)?;
        let mut dependencies: TinyOrdMap<_, _> = empty!();
        for _ in 0..len {
            let name = LibName::decode(reader)?;
            let ty = Dependency::decode(reader)?;
            dependencies.insert(name, ty).expect("under u8::MAX");
        }

        let len = u16::decode(reader)?;
        let mut types: BTreeMap<_, _> = empty!();
        for _ in 0..len {
            let name = TypeName::decode(reader)?;
            let ty = Ty::decode(reader)?;
            types.insert(name, ty);
        }

        Ok(TypeLib {
            name,
            dependencies,
            types: Confined::try_from(types)?,
        })
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

impl Encode for Dependency {
    fn encode(&self, writer: &mut impl Write) -> Result<(), Error> {
        self.id.encode(writer)?;
        self.name.encode(writer)?;
        self.ver.encode(writer)
    }
}

impl Decode for Dependency {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let id = TypeLibId::decode(reader)?;
        let name = LibName::decode(reader)?;
        let ver = SemVer::decode(reader)?;
        Ok(Dependency { id, name, ver })
    }
}

impl Encode for TypeLibId {
    fn encode(&self, writer: &mut impl Write) -> Result<(), Error> {
        writer.write_all(self.as_bytes())
    }
}

impl Decode for TypeLibId {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(TypeLibId::from_inner(blake3::Hash::from(buf)))
    }
}
