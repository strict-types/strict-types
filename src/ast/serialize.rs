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

use amplify::confinement::MediumVec;

use super::inner::TyInner;
use super::Ty;

pub enum DecodeError {}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Cls {
    Primitive = 0,
    Enum = 1,
    Union = 2,
    Struct = 3,
    Array = 4,
    Ascii = 5,
    Unicode = 6,
    List = 7,
    Set = 8,
    Map = 9,
}

impl Cls {
    pub const ALL: [Cls; 10] = [
        Cls::Primitive,
        Cls::Enum,
        Cls::Union,
        Cls::Struct,
        Cls::Array,
        Cls::Ascii,
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

impl Ty {
    pub fn deserialize(ast_data: MediumVec<u8>) -> Result<Self, DecodeError> { todo!() }

    pub fn serialize(&self) -> MediumVec<u8> { todo!() }
}

impl TyInner {
    pub const fn cls(&self) -> Cls {
        match self {
            TyInner::Primitive(_) => Cls::Primitive,
            TyInner::Enum(_) => Cls::Enum,
            TyInner::Union(_) => Cls::Union,
            TyInner::Struct(_) => Cls::Struct,
            TyInner::Array(_, _) => Cls::Array,
            TyInner::Ascii(_) => Cls::Ascii,
            TyInner::Unicode(_) => Cls::Unicode,
            TyInner::List(_, _) => Cls::List,
            TyInner::Set(_, _) => Cls::Set,
            TyInner::Map(_, _, _) => Cls::Map,
        }
    }
}
