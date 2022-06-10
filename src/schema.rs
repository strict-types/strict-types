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

use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use strict_encoding::{StrictDecode, StrictEncode};

use crate::{StrictSet, StrictVec};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictEncode, StrictDecode)]
#[display(Debug)]
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

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub struct StructField {
    pub optional: bool,
    pub value: DataType,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub struct StructType(StrictVec<StructField, 1>);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub enum KeyType {
    Primitive(PrimitiveType),
    Fixed(u16, PrimitiveType),
    Bytes,
    Unicode,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub enum DataType {
    Primitive(PrimitiveType),
    Union(StrictSet<PrimitiveType, 2>),
    Enum(StrictSet<PrimitiveType, 1>),
    Struct(StructTypeBox),
    Fixed(u16, DataTypeBox),
    List(DataTypeBox),
    Map(KeyType, DataTypeBox),
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct DataTypeBox(pub Box<DataType>);

impl Deref for DataTypeBox {
    type Target = DataType;

    fn deref(&self) -> &Self::Target { self.0.deref() }
}

impl DerefMut for DataTypeBox {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl AsRef<DataType> for DataTypeBox {
    fn as_ref(&self) -> &DataType { self.0.as_ref() }
}

impl AsMut<DataType> for DataTypeBox {
    fn as_mut(&mut self) -> &mut DataType { self.0.as_mut() }
}

impl StrictEncode for DataTypeBox {
    fn strict_encode<E: Write>(&self, e: E) -> Result<usize, strict_encoding::Error> {
        self.as_ref().strict_encode(e)
    }
}

impl StrictDecode for DataTypeBox {
    fn strict_decode<D: Read>(d: D) -> Result<Self, strict_encoding::Error> {
        Ok(Self(Box::new(DataType::strict_decode(d)?)))
    }
}

impl DataTypeBox {
    pub fn new(val: DataType) -> Self { Self(Box::new(val)) }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
pub struct StructTypeBox(pub Box<StructType>);

impl Deref for StructTypeBox {
    type Target = StructType;

    fn deref(&self) -> &Self::Target { self.0.deref() }
}

impl DerefMut for StructTypeBox {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl AsRef<StructType> for StructTypeBox {
    fn as_ref(&self) -> &StructType { self.0.as_ref() }
}

impl AsMut<StructType> for StructTypeBox {
    fn as_mut(&mut self) -> &mut StructType { self.0.as_mut() }
}

impl StructTypeBox {
    pub fn new(val: StructType) -> Self { Self(Box::new(val)) }
}

impl StrictEncode for StructTypeBox {
    fn strict_encode<E: Write>(&self, e: E) -> Result<usize, strict_encoding::Error> {
        self.as_ref().strict_encode(e)
    }
}

impl StrictDecode for StructTypeBox {
    fn strict_decode<D: Read>(d: D) -> Result<Self, strict_encoding::Error> {
        Ok(Self(Box::new(StructType::strict_decode(d)?)))
    }
}
