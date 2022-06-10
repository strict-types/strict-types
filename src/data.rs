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

use crate::{StrictSet, StrictVec};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum PrimitiveType {
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    U512,
    U1024,

    I8,
    I16,
    I32,
    I64,
    I128,
    I256,
    I512,
    I1024,

    F16b,
    F16,
    F32,
    F64,
    F80,
    F128,
    F256,
    F512,
}

pub struct StructField {
    pub optional: bool,
    pub value: DataType,
}

pub struct StructType(StrictVec<StructField, 1>);

pub enum KeyType {
    Primitive(PrimitiveType),
    Fixed(u16, PrimitiveType),
    Bytes,
    Unicode,
}

pub enum DataType {
    Primitive(PrimitiveType),
    Union(StrictSet<PrimitiveType, 2>),
    Enum(StrictSet<PrimitiveType, 1>),
    Struct(Box<StructType>),
    Fixed(u16, Box<DataType>),
    List(Box<DataType>),
    Map(KeyType, Box<DataType>),
}
