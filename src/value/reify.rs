// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 UBIDECO Institute
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

//! Reification module: reads & writes strict values from binary strict encodings.

use std::io;

use amplify::num::u24;
use encoding::constants::*;
use encoding::{DecodeError, StrictDecode, StrictReader};
use indexmap::IndexMap;

use crate::typify::{TypeSpec, TypedVal};
use crate::{SemId, StrictVal, Ty, TypeSystem};

impl TypeSystem {
    pub fn store(
        &self,
        spec: impl Into<TypeSpec>,
        obj: TypedVal,
        e: impl io::Write,
    ) -> Result<(), Error> {
        todo!()
    }

    pub fn load(&self, spec: impl Into<TypeSpec>, d: impl io::Read) -> Result<TypedVal, Error> {
        let spec = spec.into();
        let ty = &self.find(&spec).ok_or_else(|| Error::TypeAbsent(spec.clone()))?.ty;

        let read_list = |len: usize, ty: SemId, reader: StrictReader<_>| -> Result<_, Error> {
            let mut list = Vec::with_capacity(len);
            let mut d = reader.unbox();
            for _ in 0..len {
                let item = self.load(ty, &mut d)?;
                list.push(item.val);
            }
            Ok(list)
        };

        let read_map =
            |len: usize, key_ty: SemId, ty: SemId, reader: StrictReader<_>| -> Result<_, Error> {
                let mut list = Vec::with_capacity(len);
                let mut d = reader.unbox();
                for _ in 0..len {
                    let key = self.load(key_ty, &mut d)?;
                    let item = self.load(ty, &mut d)?;
                    list.push((key.val, item.val));
                }
                Ok(list)
            };

        let mut reader = StrictReader::with(usize::MAX, d);

        let val = match ty {
            Ty::Primitive(prim) => {
                match *prim {
                    U8 => StrictVal::num(u8::strict_decode(&mut reader)?),
                    U16 => StrictVal::num(u16::strict_decode(&mut reader)?),
                    U24 => StrictVal::num(u24::strict_decode(&mut reader)?.into_u32()),
                    U32 => StrictVal::num(u32::strict_decode(&mut reader)?),
                    U64 => StrictVal::num(u64::strict_decode(&mut reader)?),
                    U128 => StrictVal::num(u128::strict_decode(&mut reader)?),
                    I8 => StrictVal::num(i8::strict_decode(&mut reader)?),
                    I16 => StrictVal::num(i16::strict_decode(&mut reader)?),
                    // I24 => StrictVal::num(i24::strict_decode(&mut reader)?),
                    I32 => StrictVal::num(i32::strict_decode(&mut reader)?),
                    I64 => StrictVal::num(i64::strict_decode(&mut reader)?),
                    I128 => StrictVal::num(i128::strict_decode(&mut reader)?),
                    other => {
                        return Err(Error::NotImplemented(format!(
                            "loading {other} into a typed value is not yet implemented"
                        )))
                    }
                }
            }
            Ty::UnicodeChar => {
                todo!()
            }
            Ty::Enum(variants) => {
                let tag = u8::strict_decode(&mut reader)?;
                if !variants.has_tag(tag) {
                    return Err(DecodeError::EnumTagNotKnown(spec.to_string(), tag).into());
                }
                StrictVal::enumer(tag)
            }
            Ty::Union(variants) => {
                let tag = u8::strict_decode(&mut reader)?;
                let Some(ty) = variants.ty_by_ord(tag) else {
                    return Err(DecodeError::EnumTagNotKnown(spec.to_string(), tag).into());
                };
                let fields = self.load(*ty, reader.unbox())?;
                StrictVal::union(tag, fields.val)
            }
            Ty::Tuple(reqs) => {
                let mut fields = Vec::with_capacity(reqs.len());
                let mut d = reader.unbox();
                for ty in reqs {
                    let checked = self.load(*ty, &mut d)?;
                    fields.push(checked.val);
                }
                StrictVal::tuple(fields)
            }
            Ty::Struct(reqs) => {
                let mut fields = IndexMap::with_capacity(reqs.len());
                let mut d = reader.unbox();
                for field in reqs {
                    let checked = self.load(field.ty, &mut d)?;
                    fields.insert(field.name.clone(), checked.val);
                }
                StrictVal::Struct(fields)
            }
            Ty::Array(_ty, _len) => {
                todo!()
            }
            Ty::List(ty, sizing) if sizing.max <= u8::MAX as u64 => {
                let len = u8::strict_decode(&mut reader)?;
                let list = read_list(len as usize, *ty, reader)?;
                StrictVal::List(list)
            }
            Ty::List(ty, sizing) if sizing.max <= u16::MAX as u64 => {
                let len = u16::strict_decode(&mut reader)?;
                let list = read_list(len as usize, *ty, reader)?;
                StrictVal::List(list)
            }
            Ty::List(ty, sizing) if sizing.max <= u24::MAX.into_u64() => {
                let len = u24::strict_decode(&mut reader)?;
                let list = read_list(len.into_usize(), *ty, reader)?;
                StrictVal::List(list)
            }
            Ty::List(ty, sizing) if sizing.max <= u32::MAX as u64 => {
                let len = u32::strict_decode(&mut reader)?;
                let list = read_list(len as usize, *ty, reader)?;
                StrictVal::List(list)
            }
            Ty::List(ty, _) => {
                let len = u64::strict_decode(&mut reader)?;
                let list = read_list(len as usize, *ty, reader)?;
                StrictVal::List(list)
            }
            // TODO: Find a way to check for the uniqueness of the set values
            Ty::Set(ty, sizing) if sizing.max <= u8::MAX as u64 => {
                let len = u8::strict_decode(&mut reader)?;
                let list = read_list(len as usize, *ty, reader)?;
                StrictVal::Set(list)
            }
            Ty::Set(ty, sizing) if sizing.max <= u16::MAX as u64 => {
                let len = u16::strict_decode(&mut reader)?;
                let list = read_list(len as usize, *ty, reader)?;
                StrictVal::Set(list)
            }
            Ty::Set(ty, sizing) if sizing.max <= u24::MAX.into_u64() => {
                let len = u24::strict_decode(&mut reader)?;
                let list = read_list(len.into_usize(), *ty, reader)?;
                StrictVal::Set(list)
            }
            Ty::Set(ty, sizing) if sizing.max <= u32::MAX as u64 => {
                let len = u32::strict_decode(&mut reader)?;
                let list = read_list(len as usize, *ty, reader)?;
                StrictVal::Set(list)
            }
            Ty::Set(ty, _) => {
                let len = u64::strict_decode(&mut reader)?;
                let list = read_list(len as usize, *ty, reader)?;
                StrictVal::Set(list)
            }
            Ty::Map(key_ty, ty, sizing) if sizing.max <= u8::MAX as u64 => {
                let len = u8::strict_decode(&mut reader)?;
                let key_ty = key_ty.to_ty().id(None);
                let list = read_map(len as usize, key_ty, *ty, reader)?;
                StrictVal::Map(list)
            }
            Ty::Map(key_ty, ty, sizing) if sizing.max <= u16::MAX as u64 => {
                let len = u16::strict_decode(&mut reader)?;
                let key_ty = key_ty.to_ty().id(None);
                let list = read_map(len as usize, key_ty, *ty, reader)?;
                StrictVal::Map(list)
            }
            Ty::Map(key_ty, ty, sizing) if sizing.max <= u24::MAX.into_u64() => {
                let len = u24::strict_decode(&mut reader)?;
                let key_ty = key_ty.to_ty().id(None);
                let list = read_map(len.into_usize(), key_ty, *ty, reader)?;
                StrictVal::Map(list)
            }
            Ty::Map(key_ty, ty, sizing) if sizing.max <= u32::MAX as u64 => {
                let len = u32::strict_decode(&mut reader)?;
                let key_ty = key_ty.to_ty().id(None);
                let list = read_map(len as usize, key_ty, *ty, reader)?;
                StrictVal::Map(list)
            }
            Ty::Map(key_ty, ty, sizing) => {
                let len = u64::strict_decode(&mut reader)?;
                let key_ty = key_ty.to_ty().id(None);
                let list = read_map(len as usize, key_ty, *ty, reader)?;
                StrictVal::Map(list)
            }
        };

        Ok(TypedVal { val, spec })
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// unknown type `{0}`.
    TypeAbsent(TypeSpec),

    /// {0} is not yet implemented. Please update `strict_types` to the latest version.
    NotImplemented(String),

    #[display(inner)]
    #[from]
    Decode(DecodeError),
}
