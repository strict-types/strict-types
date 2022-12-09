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
use std::fmt::{self, Display, Formatter};

use amplify::ascii::AsciiString;
use amplify::confinement::{Confined, SmallOrdMap};

use crate::ast::{FieldName, Variants};
use crate::primitive::Primitive;
use crate::util::Sizing;

pub type TypeName = Confined<AsciiString, 1, 32>;
pub type TypeRef = TypeName;

pub struct TypeLib {
    pub types: SmallOrdMap<TypeName, TypeDef>,
}

pub type EnumDef = Variants;

#[derive(Wrapper, WrapperMut, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct FieldDef(Confined<BTreeMap<FieldName, TypeRef>, 1, { u8::MAX as usize }>);

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(inner)]
pub enum TypeDef {
    Primitive(Primitive),
    Enum(EnumDef),
    Union(FieldDef),
    Struct(FieldDef),
    Array(TypeRef, u16),
    Ascii(Sizing),
    Unicode(Sizing),
    List(TypeRef, Sizing),
    Set(TypeRef, Sizing),
    Map(KeyDef, TypeRef, Sizing),
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(inner)]
pub enum KeyDef {
    Primitive(Primitive),
    Enum(TypeRef),
    /// Fixed-size byte array
    Array(u16),
    Ascii(Sizing),
    Unicode(Sizing),
    Bytes(Sizing),
}

impl Display for FieldDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter();
        let last = iter.next_back();
        for (name, ty_ref) in iter {
            write!(f, "{} {}, ", name, ty_ref)?;
        }
        if let Some((name, ty_ref)) = last {
            write!(f, "{} {}", name, ty_ref)?;
        }
        Ok(())
    }
}
