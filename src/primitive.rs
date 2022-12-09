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

use amplify::num::u5;

pub mod constants {
    pub const U8: u8 = 0x01;
    pub const U16: u8 = 0x02;
    pub const U24: u8 = 0x03;
    pub const U32: u8 = 0x04;
    pub const U48: u8 = 0x06;
    pub const U64: u8 = 0x08;
    pub const U128: u8 = 0x10;
    pub const U160: u8 = 0x14;
    pub const U256: u8 = 0x20;
    pub const U512: u8 = 0x22;
    pub const U1024: u8 = 0x36;
    pub const I8: u8 = 0x41;
    pub const I16: u8 = 0x42;
    pub const I24: u8 = 0x43;
    pub const I32: u8 = 0x44;
    pub const I48: u8 = 0x46;
    pub const I64: u8 = 0x48;
    pub const I128: u8 = 0x50;
    pub const I256: u8 = 0x60;
    pub const I512: u8 = 0x62;
    pub const I1024: u8 = 0x76;
    pub const N8: u8 = 0x81;
    pub const N16: u8 = 0x82;
    pub const N24: u8 = 0x83;
    pub const N32: u8 = 0x84;
    pub const N48: u8 = 0x86;
    pub const N64: u8 = 0x88;
    pub const N128: u8 = 0x91;
    pub const F16: u8 = 0xC1;
    pub const F32: u8 = 0xC4;
    pub const F64: u8 = 0xC8;
    pub const F80: u8 = 0xCA;
    pub const F128: u8 = 0xD0;
    pub const F256: u8 = 0xE0;

    pub const UNIT: u8 = 0x00;
    pub const BYTE: u8 = 0x40;
    pub const CHAR: u8 = 0x80;
    pub const F16B: u8 = 0xC0;
}
pub use constants::*;

/// Information about numeric type
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NumInfo {
    /// Class of the number
    pub ty: NumTy,
    /// Size of the number, in bytes
    pub size: NumSize,
}

impl NumInfo {
    pub fn from_code(id: u8) -> Self {
        NumInfo {
            ty: NumTy::from_code(id),
            size: NumSize::from_code(id),
        }
    }

    pub fn into_code(self) -> u8 { self.ty.into_code() | self.size.into_code() }

    pub fn size(self) -> u16 { self.size.size() }
}

/// The way how the size is computed and encoded in the type id
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NumSize {
    /// Lowest 5 bits contain type size in bytes
    Bytes(u5),
    /// Lowest 5 bits contain a factor defining the size according to the
    /// equation `16 * (2 + factor)`
    Factored(u5),
}

impl NumSize {
    pub fn from_code(id: u8) -> Self {
        let code = id & 0x1F;
        match id & 0x20 / 0x20 {
            0 => NumSize::Bytes(code.try_into().expect("bit masked")),
            1 => NumSize::Factored(code.try_into().expect("bit masked")),
            _ => unreachable!(),
        }
    }

    pub fn into_code(self) -> u8 {
        match self {
            NumSize::Bytes(bytes) => bytes.as_u8(),
            NumSize::Factored(factor) => factor.as_u8() | 0x20,
        }
    }

    pub fn size(self) -> u16 {
        match self {
            NumSize::Bytes(bytes) => bytes.as_u8() as u16,
            NumSize::Factored(factor) => 2 * (factor.as_u8() as u16 + 1),
        }
    }
}

/// Class of the number type
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum NumTy {
    Unsigned = 0x00,
    Signed = 0x40,
    NonZero = 0x80,
    Float = 0xC0,
}

impl NumTy {
    pub fn from_code(id: u8) -> Self {
        match id & 0xC0 {
            x if x == NumTy::Unsigned as u8 => NumTy::Unsigned,
            x if x == NumTy::Signed as u8 => NumTy::Signed,
            x if x == NumTy::NonZero as u8 => NumTy::NonZero,
            x if x == NumTy::Float as u8 => NumTy::Float,
            _ => unreachable!(),
        }
    }

    pub fn into_code(self) -> u8 { self as u8 }
}
