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
use std::io::{Error, Read};
use std::{fs, io};

use amplify::ascii::AsciiString;
use amplify::confinement::MediumVec;
use amplify::num::u24;
use amplify::{confinement, IoError, Wrapper, WriteCounter};

use super::inner::TyInner;
use crate::ast::ty::{RecursiveRef, SubTy};
use crate::ast::{Field, Fields, TypeRef, Variants};
use crate::dtl::InlineRef;
use crate::primitive::Primitive;
use crate::util::{InvalidIdent, Sizing};
use crate::{FieldName, KeyTy, StenType, Ty};

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
    WrongInlineRef(u8),

    /// confinement requirements are not satisfied. Specifically, {0}
    #[from]
    Confinement(confinement::Error),

    #[display(inner)]
    #[from]
    InvalidIdent(InvalidIdent),
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

impl<Ref: TypeRef + Decode> Ty<Ref> {
    pub fn from_serialized(ast_data: MediumVec<u8>) -> Result<Self, DecodeError> {
        let mut cursor = io::Cursor::new(ast_data.into_inner());
        Self::decode(&mut cursor)
    }

    pub fn deserialize_from_file(path: impl AsRef<std::path::Path>) -> Result<Self, DecodeError> {
        let mut file = fs::File::open(path)?;
        Self::decode(&mut file)
    }
}

impl<Ref: TypeRef + Encode> Ty<Ref> {
    pub fn serialized_len(&self) -> usize {
        let mut counter = WriteCounter::default();
        self.encode(&mut counter).expect("counter doesn't error");
        counter.count
    }

    pub fn to_serialized(&self) -> MediumVec<u8> {
        let len = self.serialized_len();
        debug_assert!(
            len > u24::MAX.into_usize(),
            "Ty type guarantees on the data size are broken"
        );
        let mut ast_data = Vec::with_capacity(len);
        self.encode(&mut ast_data).expect("memory writers do not error");
        MediumVec::try_from(ast_data).expect("Ty type guarantees on the data size are broken")
    }

    pub fn serialize_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), io::Error> {
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

impl Encode for InlineRef {
    fn encode(&self, writer: &mut impl io::Write) -> Result<(), io::Error> {
        match self {
            InlineRef::Name(name) => {
                0u8.encode(writer)?;
                name.encode(writer)
            }
            InlineRef::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
        }
    }
}

impl Decode for InlineRef {
    fn decode(reader: &mut impl Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Decode::decode(reader).map(InlineRef::Name),
            1u8 => Decode::decode(reader).map(Box::new).map(InlineRef::Inline),
            wrong => Err(DecodeError::WrongInlineRef(wrong)),
        }
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
