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

use crate::ast::ty::{NestedRef, SubTy};
use crate::ast::{Field, Fields, TyInner, TypeRef, Variants};
use crate::primitive::Primitive;
use crate::{
    Cls, Decode, DecodeError, Deserialize, Encode, FieldName, KeyTy, Serialize, StenType,
    StenWrite, Ty, TyId,
};

impl<Ref: TypeRef> TyInner<Ref> {
    pub const fn cls(&self) -> Cls {
        match self {
            TyInner::Primitive(_) => Cls::Primitive,
            TyInner::Enum(_) => Cls::Enum,
            TyInner::Union(_) => Cls::Union,
            TyInner::Struct(_) => Cls::Struct,
            TyInner::Array(_, _) => Cls::Array,
            TyInner::UnicodeChar => Cls::UnicodeChar,
            TyInner::List(_, _) => Cls::List,
            TyInner::Set(_, _) => Cls::Set,
            TyInner::Map(_, _, _) => Cls::Map,
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
        match self.as_inner() {
            TyInner::Primitive(prim) => prim.encode(writer),
            TyInner::Enum(vars) => vars.encode(writer),
            TyInner::Union(fields) => fields.encode(writer),
            TyInner::Struct(fields) => fields.encode(writer),
            TyInner::Array(ty, len) => {
                ty.encode(writer)?;
                len.encode(writer)
            }
            TyInner::UnicodeChar => Ok(()),
            TyInner::List(ty, sizing) => {
                ty.encode(writer)?;
                sizing.encode(writer)
            }
            TyInner::Set(ty, sizing) => {
                ty.encode(writer)?;
                sizing.encode(writer)
            }
            TyInner::Map(key, ty, sizing) => {
                key.encode(writer)?;
                ty.encode(writer)?;
                sizing.encode(writer)
            }
        }
    }
}

impl<Ref: TypeRef + Decode> Decode for Ty<Ref> {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        Ok(Ty::from_inner(match Cls::decode(reader)? {
            Cls::Primitive => TyInner::Primitive(Decode::decode(reader)?),
            Cls::Enum => TyInner::Enum(Decode::decode(reader)?),
            Cls::Union => TyInner::Union(Decode::decode(reader)?),
            Cls::Struct => TyInner::Struct(Decode::decode(reader)?),
            Cls::Array => TyInner::Array(Decode::decode(reader)?, Decode::decode(reader)?),
            Cls::UnicodeChar => TyInner::UnicodeChar,
            Cls::AsciiStr => return Err(DecodeError::InvalidTyCls(Cls::AsciiStr)),
            Cls::List => TyInner::List(Decode::decode(reader)?, Decode::decode(reader)?),
            Cls::Set => TyInner::Set(Decode::decode(reader)?, Decode::decode(reader)?),
            Cls::Map => TyInner::Map(
                Decode::decode(reader)?,
                Decode::decode(reader)?,
                Decode::decode(reader)?,
            ),
        }))
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
        self.as_ty().encode(writer)
    }
}

impl Decode for StenType {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        Ok(StenType {
            name: Decode::decode(reader)?,
            ty: Box::new(Ty::decode(reader)?),
        })
    }
}

impl Encode for SubTy {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), io::Error> {
        self.as_ty().encode(writer)
    }
}

impl Decode for SubTy {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        Ty::decode(reader).map(SubTy::from)
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
        if let Some(name) = &self.name {
            name.encode(writer)?;
        } else {
            0u8.encode(writer)?;
        }
        self.ord.encode(writer)
    }
}

impl Decode for Field {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let name = FieldName::decode(reader)?;
        let name = if name.is_empty() { None } else { Some(name) };
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

impl Encode for TyId {
    fn encode(&self, writer: &mut impl StenWrite) -> Result<(), Error> {
        writer.write_byte_array(*self.as_bytes())
    }
}

impl Decode for TyId {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(TyId::from_inner(blake3::Hash::from(buf)))
    }
}
