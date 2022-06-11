// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::fmt::{self, Display, Formatter};

use strict_encoding::{StrictDecode, StrictEncode};

use crate::{AsciiString, StrictSet, StrictVec};

pub type DataTypeName = AsciiString<1, 32>;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictEncode, StrictDecode)]
#[strict_encoding(by_value, repr = u8)]
#[display(Debug)]
pub enum PrimitiveType {
    U8 = 0x00,
    U16 = 0x01,
    U32 = 0x02,
    U64 = 0x03,
    U128 = 0x04,
    U256 = 0x05,
    U512 = 0x06,
    U1024 = 0x07,

    I8 = 0x10,
    I16 = 0x11,
    I32 = 0x12,
    I64 = 0x13,
    I128 = 0x14,
    I256 = 0x15,
    I512 = 0x16,
    I1024 = 0x17,

    F16b = 0x30,
    F16 = 0x31,
    F32 = 0x32,
    F64 = 0x33,
    F80 = 0x34,
    F128 = 0x35,
    F256 = 0x36,
    F512 = 0x37,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub struct StructField {
    pub optional: bool,
    pub type_name: DataTypeName,
}

impl Display for StructField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.type_name)?;
        if self.optional {
            f.write_str("?")?;
        }
        Ok(())
    }
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[derive(StrictEncode, StrictDecode)]
pub struct StructType(StrictVec<StructField, 1>);

impl Display for StructType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.len() as usize;
        for (pos, field) in self.0.iter().enumerate() {
            Display::fmt(field, f)?;
            if pos < len - 1 {
                f.write_str(", ")?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictEncode, StrictDecode)]
pub enum KeyType {
    #[display(inner)]
    Primitive(PrimitiveType),

    #[display("{1}[{0}]")]
    Fixed(u16, PrimitiveType),

    #[display("Bytes")]
    Bytes,

    #[display("String")]
    Unicode,
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[derive(StrictEncode, StrictDecode)]
pub struct EnumType(StrictSet<PrimitiveType, 1>);

impl Display for EnumType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.len() as usize;
        for (pos, field) in self.0.iter().enumerate() {
            Display::fmt(field, f)?;
            if pos < len - 1 {
                f.write_str(", ")?;
            }
        }
        Ok(())
    }
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[derive(StrictEncode, StrictDecode)]
pub struct UnionType(StrictSet<PrimitiveType, 2>);

impl Display for UnionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.len() as usize;
        for (pos, field) in self.0.iter().enumerate() {
            Display::fmt(field, f)?;
            if pos < len - 1 {
                f.write_str(", ")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub enum DataType {
    Primitive(PrimitiveType),
    Union(DataTypeName),
    Enum(DataTypeName),
    Struct(DataTypeName),
    Fixed(u16, DataTypeName),
    List(DataTypeName),
    Map(KeyType, DataTypeName),
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Primitive(ty) => Display::fmt(ty, f),
            DataType::Union(ty) => Display::fmt(ty, f),
            DataType::Enum(ty) => Display::fmt(ty, f),
            DataType::Struct(ty) => Display::fmt(ty, f),
            DataType::Fixed(size, ty) => {
                Display::fmt(ty, f)?;
                write!(f, "[{}]", size)
            }
            DataType::List(ty) => {
                Display::fmt(ty, f)?;
                f.write_str("*")
            }
            DataType::Map(key, ty) => {
                f.write_str("{")?;
                Display::fmt(key, f)?;
                f.write_str("} -> ")?;
                Display::fmt(ty, f)?;
                f.write_str("")
            }
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictEncode, StrictDecode)]
#[display("{name} :: {ty}")]
pub struct TypeDecl {
    pub name: DataTypeName,
    pub ty: DataType,
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[derive(StrictEncode, StrictDecode)]
pub struct TypeSystem(StrictVec<TypeDecl>);
