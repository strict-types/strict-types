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

use std::fmt::{self, Display, Formatter, Write};

use crate::primitive::{NumInfo, NumSize, NumTy};
use crate::{StenSchema, StenType, Ty};

pub const U8: Primitive = Primitive::unsigned(1);
pub const U16: Primitive = Primitive::unsigned(2);
pub const U24: Primitive = Primitive::unsigned(3);
pub const U32: Primitive = Primitive::unsigned(4);
pub const U48: Primitive = Primitive::unsigned(6);
pub const U64: Primitive = Primitive::unsigned(8);
pub const U128: Primitive = Primitive::unsigned(16);
pub const U160: Primitive = Primitive::unsigned(20);
pub const U256: Primitive = Primitive::unsigned(32);
pub const U512: Primitive = Primitive::unsigned(64);
pub const U1024: Primitive = Primitive::unsigned(128);

pub const I8: Primitive = Primitive::signed(1);
pub const I16: Primitive = Primitive::signed(2);
pub const I24: Primitive = Primitive::signed(3);
pub const I32: Primitive = Primitive::signed(4);
pub const I48: Primitive = Primitive::signed(6);
pub const I64: Primitive = Primitive::signed(8);
pub const I128: Primitive = Primitive::signed(16);
pub const I256: Primitive = Primitive::signed(32);
pub const I512: Primitive = Primitive::signed(64);
pub const I1024: Primitive = Primitive::signed(128);

pub const N8: Primitive = Primitive::non_zero(1);
pub const N16: Primitive = Primitive::non_zero(2);
pub const N24: Primitive = Primitive::non_zero(3);
pub const N32: Primitive = Primitive::non_zero(4);
pub const N48: Primitive = Primitive::non_zero(6);
pub const N64: Primitive = Primitive::non_zero(8);
pub const N128: Primitive = Primitive::non_zero(16);

pub const F16: Primitive = Primitive::float(2);
pub const F32: Primitive = Primitive::float(4);
pub const F64: Primitive = Primitive::float(8);
pub const F80: Primitive = Primitive::float(10);
pub const F128: Primitive = Primitive::float(16);
pub const F256: Primitive = Primitive::float(32);

pub const UNIT: Primitive = Primitive(0x00);
pub const BYTE: Primitive = Primitive(0x40);
pub const RESERVED: Primitive = Primitive(0x80);
pub const F16B: Primitive = Primitive(0xC0);

pub const FLOAT_RESERVED_1: Primitive = Primitive(0xC1);
pub const FLOAT_RESERVED_2: Primitive = Primitive(0xC3);
pub const FLOAT_RESERVED_3: Primitive = Primitive(0xC5);
pub const FLOAT_RESERVED_4: Primitive = Primitive(0xC6);
pub const FLOAT_RESERVED_5: Primitive = Primitive(0xC7);
pub const FLOAT_RESERVED_6: Primitive = Primitive(0xC9);
pub const FLOAT_RESERVED_7: Primitive = Primitive(0xCB);
pub const FLOAT_RESERVED_8: Primitive = Primitive(0xCC);
pub const FLOAT_RESERVED_9: Primitive = Primitive(0xCD);
pub const FLOAT_RESERVED_10: Primitive = Primitive(0xCE);
pub const FLOAT_RESERVED_11: Primitive = Primitive(0xCF);
pub const FLOAT_RESERVED_12: Primitive = Primitive(0xD1);
pub const FLOAT_RESERVED_13: Primitive = Primitive(0xD2);
pub const FLOAT_RESERVED_14: Primitive = Primitive(0xD3);
pub const FLOAT_RESERVED_15: Primitive = Primitive(0xD4);
pub const FLOAT_RESERVED_16: Primitive = Primitive(0xD5);
pub const FLOAT_RESERVED_17: Primitive = Primitive(0xD6);
pub const FLOAT_RESERVED_18: Primitive = Primitive(0xD7);
pub const FLOAT_RESERVED_19: Primitive = Primitive(0xD8);
pub const FLOAT_RESERVED_20: Primitive = Primitive(0xD9);
pub const FLOAT_RESERVED_21: Primitive = Primitive(0xDA);
pub const FLOAT_RESERVED_22: Primitive = Primitive(0xDB);
pub const FLOAT_RESERVED_23: Primitive = Primitive(0xDC);
pub const FLOAT_RESERVED_24: Primitive = Primitive(0xDE);
pub const FLOAT_RESERVED_25: Primitive = Primitive(0xDF);

pub const FLOAT_RESERVED_26: Primitive = Primitive(0xE1);
pub const FLOAT_RESERVED_27: Primitive = Primitive(0xE2);
pub const FLOAT_RESERVED_28: Primitive = Primitive(0xE3);
pub const FLOAT_RESERVED_29: Primitive = Primitive(0xE4);
pub const FLOAT_RESERVED_30: Primitive = Primitive(0xE5);
pub const FLOAT_RESERVED_31: Primitive = Primitive(0xE6);
pub const FLOAT_RESERVED_32: Primitive = Primitive(0xE7);
pub const FLOAT_RESERVED_33: Primitive = Primitive(0xE8);
pub const FLOAT_RESERVED_34: Primitive = Primitive(0xE9);
pub const FLOAT_RESERVED_35: Primitive = Primitive(0xEA);
pub const FLOAT_RESERVED_36: Primitive = Primitive(0xEB);
pub const FLOAT_RESERVED_37: Primitive = Primitive(0xEC);
pub const FLOAT_RESERVED_38: Primitive = Primitive(0xEE);
pub const FLOAT_RESERVED_39: Primitive = Primitive(0xEF);

pub const FLOAT_RESERVED_40: Primitive = Primitive(0xF0);
pub const FLOAT_RESERVED_41: Primitive = Primitive(0xF1);
pub const FLOAT_RESERVED_42: Primitive = Primitive(0xF2);
pub const FLOAT_RESERVED_43: Primitive = Primitive(0xF3);
pub const FLOAT_RESERVED_44: Primitive = Primitive(0xF4);
pub const FLOAT_RESERVED_45: Primitive = Primitive(0xF5);
pub const FLOAT_RESERVED_46: Primitive = Primitive(0xF6);
pub const FLOAT_RESERVED_47: Primitive = Primitive(0xF7);
pub const FLOAT_RESERVED_48: Primitive = Primitive(0xF8);
pub const FLOAT_RESERVED_49: Primitive = Primitive(0xF9);
pub const FLOAT_RESERVED_50: Primitive = Primitive(0xFA);
pub const FLOAT_RESERVED_51: Primitive = Primitive(0xFB);
pub const FLOAT_RESERVED_52: Primitive = Primitive(0xFC);
pub const FLOAT_RESERVED_53: Primitive = Primitive(0xFE);
pub const FLOAT_RESERVED_54: Primitive = Primitive(0xFF);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Primitive(u8);

impl StenSchema for Primitive {
    const STEN_TYPE_NAME: &'static str = "Primitive";

    fn sten_ty() -> Ty<StenType> { u8::sten_ty() }
}

impl Primitive {
    pub const fn unsigned(bytes: u16) -> Self {
        Primitive(
            NumInfo {
                ty: NumTy::Unsigned,
                size: NumSize::from_bytes(bytes),
            }
            .into_code(),
        )
    }

    pub const fn signed(bytes: u16) -> Self {
        Primitive(
            NumInfo {
                ty: NumTy::Signed,
                size: NumSize::from_bytes(bytes),
            }
            .into_code(),
        )
    }

    pub const fn non_zero(bytes: u16) -> Self {
        Primitive(
            NumInfo {
                ty: NumTy::NonZero,
                size: NumSize::from_bytes(bytes),
            }
            .into_code(),
        )
    }

    pub const fn float(bytes: u16) -> Self {
        Primitive(
            NumInfo {
                ty: NumTy::Float,
                size: NumSize::from_bytes(bytes),
            }
            .into_code(),
        )
    }

    pub fn from_code(code: u8) -> Self { Primitive(code) }
    pub fn into_code(self) -> u8 { self.0 }

    pub fn info(self) -> NumInfo { NumInfo::from_code(self.0) }

    pub fn byte_size(self) -> u16 { self.info().byte_size() }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            UNIT => return f.write_str("()"),
            BYTE => return f.write_str("Byte"),
            F16B => return f.write_str("F16b"),
            _ => {}
        }

        let info = self.info();
        f.write_char(match info.ty {
            NumTy::Unsigned => 'U',
            NumTy::Signed => 'I',
            NumTy::NonZero => 'N',
            NumTy::Float => 'F',
        })?;

        write!(f, "{}", info.byte_size() * 8)
    }
}
