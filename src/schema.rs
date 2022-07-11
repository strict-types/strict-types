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

use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::io::Read;

use strict_encoding::{StrictDecode, StrictEncode};

use crate::{AsciiString, OversizeError, StrictMap, StrictSet, StrictVec};

pub type TypeName = AsciiString<1, 32>;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
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

    AsciiChar = 0xFE,
    UnicodeChar = 0xFF,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[derive(StrictEncode, StrictDecode)]
pub struct StructField {
    pub ty: TypeRef,
    pub optional: bool,
}

impl Display for StructField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.ty, f)?;
        if self.optional {
            f.write_str("?")?;
        }
        Ok(())
    }
}

impl StructField {
    pub fn with(type_name: &'static str) -> Self {
        StructField {
            ty: type_name.try_into().expect("invalid struct type name"),
            optional: false,
        }
    }

    pub fn primitive(prim: PrimitiveType) -> Self {
        StructField {
            ty: TypeRef::InPlace(TypeConstr::Plain(prim)),
            optional: false,
        }
    }

    pub fn bytes() -> Self {
        StructField {
            ty: TypeRef::bytes(),
            optional: false,
        }
    }

    pub fn ascii_string() -> Self {
        StructField {
            ty: TypeRef::ascii_string(),
            optional: false,
        }
    }

    pub fn unicode_string() -> Self {
        StructField {
            ty: TypeRef::unicode_string(),
            optional: false,
        }
    }

    pub fn array(prim: PrimitiveType, size: u16) -> Self {
        StructField {
            ty: TypeRef::InPlace(TypeConstr::Array(size, prim)),
            optional: false,
        }
    }

    pub fn list(prim: PrimitiveType) -> Self {
        StructField {
            ty: TypeRef::InPlace(TypeConstr::List(prim)),
            optional: false,
        }
    }

    pub fn map(key: impl Into<KeyType>, prim: PrimitiveType) -> Self {
        StructField {
            ty: TypeRef::InPlace(TypeConstr::Map(key.into(), prim)),
            optional: false,
        }
    }

    pub fn typed_list(ty: &'static str) -> Self {
        StructField {
            ty: TypeRef::NameRef(TypeConstr::List(ty.try_into().expect("bad name"))),
            optional: false,
        }
    }

    pub fn typed_map(key: impl Into<KeyType>, ty: &'static str) -> Self {
        StructField {
            ty: TypeRef::NameRef(TypeConstr::Map(key.into(), ty.try_into().expect("bad name"))),
            optional: false,
        }
    }

    pub fn optional(ty: TypeRef) -> Self { StructField { ty, optional: true } }
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
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

impl StructType {
    #[doc(hidden)]
    pub unsafe fn from_unchecked(data: StrictVec<StructField, 1>) -> StructType { Self(data) }
}

impl<'me> IntoIterator for &'me StructType {
    type Item = &'me StructField;
    type IntoIter = std::slice::Iter<'me, StructField>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[derive(StrictEncode, StrictDecode)]
#[strict_encoding(by_value, repr = u8)]
pub enum KeyType {
    #[display(inner)]
    #[strict_encoding(value = 0x00)]
    #[from]
    Primitive(PrimitiveType),

    #[display("{1}[{0}]")]
    #[strict_encoding(value = 0x10)]
    Array(u16, PrimitiveType),

    #[display("{0}[]")]
    #[strict_encoding(value = 0x20)]
    List(PrimitiveType),
}

impl KeyType {
    pub fn bytes() -> Self { KeyType::List(PrimitiveType::U8) }

    pub fn ascii_string() -> Self { KeyType::List(PrimitiveType::AsciiChar) }

    pub fn unicode_string() -> Self { KeyType::List(PrimitiveType::UnicodeChar) }
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[derive(StrictEncode, StrictDecode)]
pub struct UnionType(StrictSet<PrimitiveType, 2>);

impl Display for UnionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.len() as usize;
        for (pos, field) in self.0.iter().enumerate() {
            Display::fmt(field, f)?;
            if pos < len - 1 {
                f.write_str(" | ")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
#[derive(StrictEncode, StrictDecode)]
#[strict_encoding(by_order, repr = u8)]
#[display(inner)]
pub enum TypeRef {
    #[from]
    #[from(PrimitiveType)]
    InPlace(TypeConstr<PrimitiveType>),

    #[from]
    #[from(TypeName)]
    NameRef(TypeConstr<TypeName>),
}

impl From<&'static str> for TypeRef {
    fn from(value: &'static str) -> Self {
        TypeRef::NameRef(AsciiString::try_from(value).expect("incorrect typ name").into())
    }
}

impl TypeRef {
    pub fn new(name: &'static str) -> TypeRef {
        TypeRef::NameRef(TypeConstr::Plain(name.try_into().expect("invalid type name")))
    }

    pub fn bytes() -> TypeRef { TypeRef::InPlace(TypeConstr::List(PrimitiveType::U8)) }

    pub fn ascii_string() -> TypeRef {
        TypeRef::InPlace(TypeConstr::List(PrimitiveType::AsciiChar))
    }

    pub fn unicode_string() -> TypeRef {
        TypeRef::InPlace(TypeConstr::List(PrimitiveType::UnicodeChar))
    }

    pub fn u8() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::U8)) }
    pub fn u16() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::U16)) }
    pub fn u32() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::U32)) }
    pub fn u64() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::U64)) }
    pub fn u128() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::U128)) }

    pub fn i8() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::I8)) }
    pub fn i16() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::I16)) }
    pub fn i32() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::I32)) }
    pub fn i64() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::I64)) }
    pub fn i128() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::I128)) }

    pub fn f32() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::F32)) }
    pub fn f64() -> TypeRef { TypeRef::InPlace(TypeConstr::Plain(PrimitiveType::F64)) }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub enum TypeConstr<T>
where T: Clone + Ord + Eq + Hash + Debug
{
    #[from]
    Plain(T),
    Array(u16, T),
    List(T),
    Set(T),
    Map(KeyType, T),
}

impl TypeConstr<TypeName> {
    pub fn type_name(&self) -> &TypeName {
        match self {
            TypeConstr::Plain(name) => name,
            TypeConstr::Array(_, name) => name,
            TypeConstr::List(name) => name,
            TypeConstr::Set(name) => name,
            TypeConstr::Map(_, name) => name,
        }
    }
}

impl<T> Display for TypeConstr<T>
where T: Clone + Ord + Eq + Hash + Debug + Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypeConstr::Plain(ty) => Display::fmt(ty, f),
            TypeConstr::Array(size, ty) => {
                Display::fmt(ty, f)?;
                write!(f, "*{}", size)
            }
            TypeConstr::List(ty) => {
                Display::fmt(ty, f)?;
                f.write_str("*")
            }
            TypeConstr::Set(ty) => {
                f.write_str("{")?;
                Display::fmt(ty, f)?;
                f.write_str("}")
            }
            TypeConstr::Map(key, ty) => {
                f.write_str("{")?;
                Display::fmt(key, f)?;
                f.write_str("} -> ")?;
                Display::fmt(ty, f)
            }
        }
    }
}

impl<T> StrictEncode for TypeConstr<T>
where T: Clone + Ord + Eq + Hash + Debug + StrictEncode
{
    fn strict_encode<E: std::io::Write>(&self, mut e: E) -> Result<usize, strict_encoding::Error> {
        Ok(match self {
            TypeConstr::Plain(ty) => {
                strict_encode_list!(e; 0x00u8, ty)
            }
            TypeConstr::Array(size, ty) => {
                strict_encode_list!(e; 0x10u8, size, ty)
            }
            TypeConstr::List(ty) => {
                strict_encode_list!(e; 0x11u8, ty)
            }
            TypeConstr::Set(ty) => {
                strict_encode_list!(e; 0x12u8, ty)
            }
            TypeConstr::Map(key, ty) => {
                strict_encode_list!(e; 0x13u8, key, ty)
            }
        })
    }
}

impl<T> StrictDecode for TypeConstr<T>
where T: Clone + Ord + Eq + Hash + Debug + StrictDecode
{
    fn strict_decode<D: Read>(mut d: D) -> Result<Self, strict_encoding::Error> {
        let ty = u8::strict_decode(&mut d)?;
        Ok(match ty {
            0x00 => Self::Plain(StrictDecode::strict_decode(&mut d)?),
            0x10 => Self::Array(
                StrictDecode::strict_decode(&mut d)?,
                StrictDecode::strict_decode(&mut d)?,
            ),
            0x11 => Self::List(StrictDecode::strict_decode(&mut d)?),
            0x12 => Self::Set(StrictDecode::strict_decode(&mut d)?),
            0x13 => Self::Map(
                StrictDecode::strict_decode(&mut d)?,
                StrictDecode::strict_decode(&mut d)?,
            ),
            other => {
                return Err(strict_encoding::Error::EnumValueNotKnown("TypeConstr", other as usize))
            }
        })
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct TypeSystem(StrictMap<TypeName, StructType>);

impl Display for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (name, ty) in &self.0 {
            Display::fmt(name, f)?;
            f.write_str(" :: ")?;
            Display::fmt(ty, f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl TypeSystem {
    pub fn new() -> Self { default!() }

    pub fn push(&mut self, name: TypeName, ty: StructType) -> Result<(), Error> {
        if self.0.contains_key(&name) {
            return Err(Error::DuplicatedType(name));
        }
        self.0.insert(name, ty)?;
        Ok(())
    }

    #[inline]
    pub fn get(&self, name: &TypeName) -> Option<&StructType> { self.0.get(name) }

    pub fn verify(&self) -> Result<(), Vec<TypeInconsistency>> {
        let mut issues = vec![];
        for (name, ty) in &*self.0 {
            for (no, field) in ty.into_iter().enumerate() {
                if let TypeRef::NameRef(r) = &field.ty {
                    if self.get(r.type_name()).is_none() {
                        issues.push(TypeInconsistency {
                            container: name.clone(),
                            field_no: no,
                            absent_type: r.type_name().clone(),
                        });
                    }
                }
            }
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display("type '{container}' references unknown type '{absent_type}' in its field #{field_no}")]
pub struct TypeInconsistency {
    pub container: TypeName,
    pub field_no: usize,
    pub absent_type: TypeName,
}

#[macro_export]
macro_rules! type_system {
    ($($name:literal :: { $($field:expr),+ $(,)? }),+ $(,)?) => { {
        use std::convert::TryInto;
        let mut ts = $crate::TypeSystem::new();
        $(
        let name = $name.try_into().expect("inline strict_vec literal contains invalid number of items");
        let ty = unsafe { $crate::StructType::from_unchecked(strict_vec![$($field),+]) };
        ts.push(name, ty).expect("invalid type declaration");
        )+
        ts
    } }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// type `{0}` is already defined
    DuplicatedType(TypeName),

    #[from]
    #[display(inner)]
    Oversize(OversizeError),
}

#[cfg(test)]
mod test {
    use amplify::hex::ToHex;

    use super::*;
    use crate::strict_vec;

    fn type_system() -> TypeSystem {
        type_system![
           "Transaction" :: {
                StructField::primitive(PrimitiveType::U32),
                StructField::typed_list("Input"),
                StructField::typed_list("Output"),
                StructField::primitive(PrimitiveType::U32),
            },
            "Input" :: {
                StructField::with("OutPoint"),
                StructField::with("Bytes"),
                StructField::with("Witness"),
            },
            "Output" :: {
                StructField::primitive(PrimitiveType::U64),
                StructField::with("Bytes"),
            },
            "OutPoint" :: {
                StructField::with("Txid"),
                StructField::primitive(PrimitiveType::U16),
            },
            "Txid" :: { StructField::array(PrimitiveType::U8, 32) },
            "Witness" :: { StructField::typed_list("Bytes") },
            "Meta" :: {
                StructField::ascii_string(), // Name
                StructField::typed_map(KeyType::unicode_string(), "UnicodeString"), // Arbitrary map
            }
        ]
    }

    #[test]
    fn display() {
        println!("{}", type_system());
    }

    #[test]
    fn test_encode() {
        let ts = type_system();
        let data = ts.strict_serialize().unwrap();
        println!("{}", data.to_hex());
        let ts2 = TypeSystem::strict_deserialize(&data).unwrap();
        println!("{}", ts2);
        assert_eq!(ts, ts2);
    }
}
