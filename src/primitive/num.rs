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

/// Information about numeric type
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NumInfo {
    /// Class of the number
    pub ty: NumTy,
    /// Size of the number, in bytes
    pub size: NumSize,
}

impl NumInfo {
    pub const fn from_code(id: u8) -> Self {
        NumInfo {
            ty: NumTy::from_code(id),
            size: NumSize::from_code(id),
        }
    }

    pub const fn into_code(self) -> u8 { self.ty.into_code() | self.size.into_code() }

    pub const fn byte_size(self) -> u16 { self.size.byte_size() }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NumSize(NumSizeInner);

impl NumSize {
    pub(super) const fn from_bytes(bytes: u16) -> Self {
        NumSize(if bytes < 0x20 {
            NumSizeInner::Bytes(bytes as u8)
        } else if bytes % 16 != 0 {
            unreachable!()
        } else {
            NumSizeInner::Factored((bytes / 16 - 2) as u8)
        })
    }

    pub(super) const fn from_code(id: u8) -> Self {
        let code = id & 0x1F;
        NumSize(match id & 0x20 / 0x20 {
            0 => NumSizeInner::Bytes(code),
            1 => NumSizeInner::Factored(code),
            _ => unreachable!(),
        })
    }

    pub(super) const fn into_code(self) -> u8 {
        match self.0 {
            NumSizeInner::Bytes(bytes) => bytes,
            NumSizeInner::Factored(factor) => factor | 0x20,
        }
    }

    pub const fn byte_size(self) -> u16 {
        match self.0 {
            NumSizeInner::Bytes(bytes) => bytes as u16,
            NumSizeInner::Factored(factor) => 2 * (factor as u16 + 1),
        }
    }
}

/// The way how the size is computed and encoded in the type id
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum NumSizeInner {
    /// Lowest 5 bits contain type size in bytes
    Bytes(u8),
    /// Lowest 5 bits contain a factor defining the size according to the
    /// equation `16 * (2 + factor)`
    Factored(u8),
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
    pub const fn from_code(id: u8) -> Self {
        match id & 0xC0 {
            x if x == NumTy::Unsigned as u8 => NumTy::Unsigned,
            x if x == NumTy::Signed as u8 => NumTy::Signed,
            x if x == NumTy::NonZero as u8 => NumTy::NonZero,
            x if x == NumTy::Float as u8 => NumTy::Float,
            _ => unreachable!(),
        }
    }

    pub const fn into_code(self) -> u8 { self as u8 }
}
