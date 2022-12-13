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

use amplify::Wrapper;

use crate::ast::ty::{NestedRef, SubTy};
use crate::ast::{Field, Fields, TyInner, TypeRef, Variants};
use crate::primitive::Primitive;
use crate::{
    Cls, Decode, DecodeError, Deserialize, Encode, FieldName, KeyTy, Serialize, StenType, Ty, TyId,
};

impl<Ref: TypeRef> TyInner<Ref> {
    pub const fn cls(&self) -> Cls {
        match self {
            TyInner::Primitive(_) => Cls::Primitive,
            TyInner::Enum(_) => Cls::Enum,
            TyInner::Union(_) => Cls::Union,
            TyInner::Struct(_) => Cls::Struct,
            TyInner::Array(_, _) => Cls::Array,
            TyInner::Unicode(_) => Cls::Unicode,
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
            KeyTy::Unicode(_) => Cls::Unicode,
            KeyTy::Bytes(_) => Cls::List,
        }
    }
}

impl<Ref: TypeRef + Decode> Deserialize for Ty<Ref> {}

impl<Ref: TypeRef + Encode> Serialize for Ty<Ref> {}

impl<Ref: TypeRef + Encode> Encode for Ty<Ref> {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.cls().encode(writer)?;
        match self.as_inner() {
            TyInner::Primitive(prim) => prim.encode(writer),
            TyInner::Enum(vars) => vars.encode(writer),
            TyInner::Union(fields) => fields.encode(writer),
            TyInner::Struct(fields) => fields.encode(writer),
            TyInner::Array(ty, len) => {
                writer.write_all(&len.to_le_bytes())?;
                ty.encode(writer)
            }
            TyInner::Unicode(sizing) => sizing.encode(writer),
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
            Cls::Unicode => TyInner::Unicode(Decode::decode(reader)?),
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
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.into_code().encode(writer)
    }
}

impl Decode for Primitive {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        u8::decode(reader).map(Primitive::from_code)
    }
}

impl Encode for StenType {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.as_ty().encode(writer)
    }
}

impl Encode for SubTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.as_ty().encode(writer)
    }
}

impl Decode for SubTy {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        Ty::decode(reader).map(SubTy::from)
    }
}

impl Encode for KeyTy {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.cls().encode(writer)?;
        match self {
            KeyTy::Primitive(prim) => prim.encode(writer),
            KeyTy::Enum(vars) => vars.encode(writer),
            KeyTy::Array(len) => writer.write_all(&len.to_le_bytes()),
            KeyTy::Unicode(sizing) => sizing.encode(writer),
            KeyTy::Bytes(sizing) => sizing.encode(writer),
        }
    }
}

impl Decode for KeyTy {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        Ok(match Cls::decode(reader)? {
            Cls::Primitive => KeyTy::Primitive(Decode::decode(reader)?),
            Cls::Enum => KeyTy::Enum(Decode::decode(reader)?),
            Cls::Array => KeyTy::Array(Decode::decode(reader)?),
            Cls::Unicode => KeyTy::Unicode(Decode::decode(reader)?),
            Cls::List => KeyTy::Bytes(Decode::decode(reader)?),
            wrong => return Err(DecodeError::InvalidTyCls(wrong)),
        })
    }
}

impl Encode for Variants {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        self.len_u8().encode(writer)?;
        for (field, val) in self.as_inner() {
            field.encode(writer)?;
            val.encode(writer)?;
        }
        Ok(())
    }
}

impl Decode for Variants {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let len = u8::decode(reader)?;
        let mut map = BTreeMap::new();
        for _ in 0..len {
            map.insert(Decode::decode(reader)?, Decode::decode(reader)?);
        }
        Variants::try_from(map).map_err(DecodeError::from)
    }
}

impl Encode for Field {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
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
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        writer.write(&[self.len_u8()])?;
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
        for _ in 0..len {
            map.insert(Decode::decode(reader)?, Decode::decode(reader)?);
        }
        Fields::try_from(map).map_err(DecodeError::from)
    }
}

impl Encode for TyId {
    fn encode(&self, writer: &mut impl Write) -> Result<(), Error> {
        writer.write_all(self.as_inner().as_bytes())
    }
}

impl Decode for TyId {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(TyId::from_inner(blake3::Hash::from(buf)))
    }
}
