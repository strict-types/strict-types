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

use amplify::Wrapper;

use crate::ast::{Field, Fields, TypeRef, Variants};
use crate::primitive::Primitive;
use crate::{
    Cls, Decode, DecodeError, Deserialize, Encode, FieldName, KeyTy, SemId, Serialize, StenType,
    StenWrite, Ty, TypeName,
};

impl<Ref: TypeRef> Ty<Ref> {
    pub const fn cls(&self) -> Cls {
        match self {
            Ty::Primitive(_) => Cls::Primitive,
            Ty::Enum(_) => Cls::Enum,
            Ty::Union(_) => Cls::Union,
            Ty::Struct(_) => Cls::Struct,
            Ty::Array(_, _) => Cls::Array,
            Ty::UnicodeChar => Cls::UnicodeChar,
            Ty::List(_, _) => Cls::List,
            Ty::Set(_, _) => Cls::Set,
            Ty::Map(_, _, _) => Cls::Map,
        }
    }
}

impl KeyTy {
    pub const fn cls(&self) -> Cls {
        match self {
            KeyTy::Primitive(_) => Cls::Primitive,
            KeyTy::Enum(_) => Cls::Enum,
            KeyTy::Array(_) => Cls::Array,
            KeyTy::UnicodeStr(_) => Cls::UnicodeChar,
            KeyTy::AsciiStr(_) => Cls::AsciiStr,
            KeyTy::Bytes(_) => Cls::List,
        }
    }
}

impl<Ref: TypeRef + Decode> Deserialize for Ty<Ref> {}

impl<Ref: TypeRef + Encode> Serialize for Ty<Ref> {}

impl<Ref: TypeRef + Encode> Encode for Ty<Ref> {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        self.cls().encode(writer)?;
        match self {
            Ty::Primitive(prim) => prim.encode(writer),
            Ty::Enum(vars) => vars.encode(writer),
            Ty::Union(fields) => fields.encode(writer),
            Ty::Struct(fields) => fields.encode(writer),
            Ty::Array(ty, len) => {
                ty.encode(writer)?;
                len.encode(writer)
            }
            Ty::UnicodeChar => Ok(()),
            Ty::List(ty, sizing) => {
                ty.encode(writer)?;
                sizing.encode(writer)
            }
            Ty::Set(ty, sizing) => {
                ty.encode(writer)?;
                sizing.encode(writer)
            }
            Ty::Map(key, ty, sizing) => {
                key.encode(writer)?;
                ty.encode(writer)?;
                sizing.encode(writer)
            }
        }
    }
}

impl<Ref: TypeRef + Decode> Decode for Ty<Ref> {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        Ok(match Cls::decode(reader)? {
            Cls::Primitive => Ty::Primitive(Decode::decode(reader)?),
            Cls::Enum => Ty::Enum(Decode::decode(reader)?),
            Cls::Union => Ty::Union(Decode::decode(reader)?),
            Cls::Struct => Ty::Struct(Decode::decode(reader)?),
            Cls::Array => Ty::Array(Decode::decode(reader)?, Decode::decode(reader)?),
            Cls::UnicodeChar => Ty::UnicodeChar,
            Cls::AsciiStr => return Err(DecodeError::InvalidTyCls(Cls::AsciiStr)),
            Cls::List => Ty::List(Decode::decode(reader)?, Decode::decode(reader)?),
            Cls::Set => Ty::Set(Decode::decode(reader)?, Decode::decode(reader)?),
            Cls::Map => {
                Ty::Map(Decode::decode(reader)?, Decode::decode(reader)?, Decode::decode(reader)?)
            }
        })
    }
}

impl Encode for Primitive {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        self.into_code().encode(writer)
    }
}

impl Decode for Primitive {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        u8::decode(reader).map(Primitive::from_code)
    }
}

impl Encode for StenType {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        self.name.encode(writer)?;
        self.ty.encode(writer)
    }
}

impl Decode for StenType {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let name = Option::<TypeName>::decode(reader)?;
        let ty = Ty::<StenType>::decode(reader)?;
        let id = ty.id(name.as_ref());
        Ok(StenType {
            name,
            ty: Box::new(ty),
            id,
        })
    }
}

impl Encode for KeyTy {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        self.cls().encode(writer)?;
        match self {
            KeyTy::Primitive(prim) => prim.encode(writer),
            KeyTy::Enum(vars) => vars.encode(writer),
            KeyTy::Array(len) => len.encode(writer),
            KeyTy::UnicodeStr(sizing) | KeyTy::AsciiStr(sizing) | KeyTy::Bytes(sizing) => {
                sizing.encode(writer)
            }
        }
    }
}

impl Decode for KeyTy {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        Ok(match Cls::decode(reader)? {
            Cls::Primitive => KeyTy::Primitive(Decode::decode(reader)?),
            Cls::Enum => KeyTy::Enum(Decode::decode(reader)?),
            Cls::Array => KeyTy::Array(Decode::decode(reader)?),
            Cls::UnicodeChar => KeyTy::UnicodeStr(Decode::decode(reader)?),
            Cls::AsciiStr => KeyTy::AsciiStr(Decode::decode(reader)?),
            Cls::List => KeyTy::Bytes(Decode::decode(reader)?),
            wrong => return Err(DecodeError::InvalidTyCls(wrong)),
        })
    }
}

impl Encode for Variants {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        self.len_u8().encode(writer)?;
        for field in self.as_inner() {
            field.encode(writer)?;
        }
        Ok(())
    }
}

impl Decode for Variants {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let len = u8::decode(reader)?;
        let mut map = BTreeSet::new();
        // TODO: Detect wrong order
        for _ in 0..len {
            map.insert(Decode::decode(reader)?);
        }
        Variants::try_from(map).map_err(DecodeError::from)
    }
}

impl Encode for Field {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        self.name.encode(writer)?;
        self.ord.encode(writer)
    }
}

impl Decode for Field {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let name = Option::<FieldName>::decode(reader)?;
        let ord = Decode::decode(reader)?;
        Ok(Field { name, ord })
    }
}

impl<Ref: TypeRef + Encode, const OP: bool> Encode for Fields<Ref, OP> {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        self.len_u8().encode(writer)?;
        for (name, ty) in self {
            name.encode(writer)?;
            ty.encode(writer)?;
        }
        Ok(())
    }
}

impl<Ref: TypeRef + Decode, const OP: bool> Decode for Fields<Ref, OP> {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let len = u8::decode(reader)?;
        let mut map = BTreeMap::new();
        // TODO: Detect wrong order
        for _ in 0..len {
            map.insert(Decode::decode(reader)?, Decode::decode(reader)?);
        }
        Fields::try_from(map).map_err(DecodeError::from)
    }
}

impl Encode for SemId {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), Error> {
        writer.write_byte_array(*self.as_bytes())
    }
}

impl Decode for SemId {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(SemId::from_inner(blake3::Hash::from(buf)))
    }
}
