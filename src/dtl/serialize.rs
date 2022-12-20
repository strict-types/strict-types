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

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::io::{Error, Read};

use amplify::confinement::{Confined, TinyOrdMap};
use amplify::num::u24;
use amplify::Wrapper;

use crate::dtl::type_lib::{Dependency, InlineRef};
use crate::dtl::{LibAlias, LibName, LibRef, TypeLib, TypeLibId, TypeSystem};
use crate::{
    Decode, DecodeError, Deserialize, EmbeddedRef, Encode, SemId, SemVer, Serialize, StenWrite, Ty,
    TypeName,
};

impl Serialize for TypeSystem {}
impl Deserialize for TypeSystem {}

impl Encode for TypeSystem {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
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
        let mut lib: BTreeSet<Ty<EmbeddedRef>> = empty!();
        let mut prev: Option<SemId> = None;
        for _ in 0..count.into_usize() {
            let ty = Ty::decode(reader)?;
            if matches!(prev, Some(id) if id > ty.id()) {
                return Err(DecodeError::WrongTypeOrdering(ty.id()));
            }
            let id = ty.id();
            prev = Some(id);
            if !lib.insert(ty) {
                return Err(DecodeError::RepeatedType(id));
            }
        }
        TypeSystem::try_from_iter(lib).map_err(DecodeError::from)
    }
}

impl Serialize for TypeLib {}
impl Deserialize for TypeLib {}

impl Encode for TypeLib {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), Error> {
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
        let mut prev = None;
        for _ in 0..len {
            let alias = LibAlias::decode(reader)?;
            let dep = Dependency::decode(reader)?;
            if matches!(prev, Some(a) if a > alias) {
                return Err(DecodeError::WrongDependencyOrdering(alias));
            }
            prev = Some(alias.clone());
            let name = dep.name.clone();
            if dependencies.insert(alias, dep).expect("under u8::MAX").is_some() {
                return Err(DecodeError::RepeatedDependency(name));
            }
        }

        let len = u16::decode(reader)?;
        let mut types: BTreeMap<_, _> = empty!();
        let mut prev: Option<SemId> = None;
        for _ in 0..len {
            let name = TypeName::decode(reader)?;
            let ty = Ty::decode(reader)?;
            if matches!(prev, Some(id) if id > ty.id()) {
                return Err(DecodeError::WrongTypeOrdering(ty.id()));
            }
            let id = ty.id();
            prev = Some(id);
            if types.insert(name, ty).is_some() {
                return Err(DecodeError::RepeatedType(id));
            }
        }

        Ok(TypeLib {
            name,
            dependencies,
            types: Confined::try_from(types)?,
        })
    }
}

impl Encode for EmbeddedRef {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        match self {
            EmbeddedRef::SemId(id) => {
                0u8.encode(writer)?;
                id.encode(writer)
            }
            EmbeddedRef::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
        }
    }
}

impl Decode for EmbeddedRef {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Ok(EmbeddedRef::SemId(Decode::decode(reader)?)),
            1u8 => Decode::decode(reader).map(EmbeddedRef::Inline),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}

impl Encode for InlineRef {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        match self {
            InlineRef::Named(name, id) => {
                0u8.encode(writer)?;
                name.encode(writer)?;
                id.encode(writer)
            }
            InlineRef::Extern(name, lib_alias, id) => {
                2u8.encode(writer)?;
                name.encode(writer)?;
                lib_alias.encode(writer)?;
                id.encode(writer)
            }
        }
    }
}

impl Decode for LibRef {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Ok(LibRef::Named(Decode::decode(reader)?, Decode::decode(reader)?)),
            1u8 => Decode::decode(reader).map(LibRef::Inline),
            2u8 => Ok(LibRef::Extern(
                Decode::decode(reader)?,
                Decode::decode(reader)?,
                Decode::decode(reader)?,
            )),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}

impl Encode for LibRef {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        match self {
            LibRef::Named(name, id) => {
                0u8.encode(writer)?;
                name.encode(writer)?;
                id.encode(writer)
            }
            LibRef::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
            LibRef::Extern(name, lib_alias, id) => {
                2u8.encode(writer)?;
                name.encode(writer)?;
                lib_alias.encode(writer)?;
                id.encode(writer)
            }
        }
    }
}

impl Decode for InlineRef {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Ok(InlineRef::Named(Decode::decode(reader)?, Decode::decode(reader)?)),
            2u8 => Ok(InlineRef::Extern(
                Decode::decode(reader)?,
                Decode::decode(reader)?,
                Decode::decode(reader)?,
            )),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}

impl Encode for Dependency {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), Error> {
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
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), Error> {
        writer.write_byte_array(*self.as_bytes())
    }
}

impl Decode for TypeLibId {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(TypeLibId::from_inner(blake3::Hash::from(buf)))
    }
}
