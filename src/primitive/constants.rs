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
