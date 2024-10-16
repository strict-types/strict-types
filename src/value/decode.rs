// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2024 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2024 UBIDECO Institute
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

use amplify::ascii::AsciiString;
use amplify::confinement::{
    Confined, LargeAscii, LargeBlob, LargeString, MediumAscii, MediumBlob, MediumString,
    SmallAscii, SmallBlob, SmallString, TinyAscii, TinyBlob, TinyString, U16 as MAX16,
    U32 as MAX32,
};
use amplify::num::{u24, u40, u48, u56};
use encoding::{DecodeError, Primitive, ReadRaw, StreamReader, StrictDecode, StrictReader};
use indexmap::IndexMap;

use crate::typesys::{SymbolicSys, TypeSymbol, UnknownType};
use crate::typify::{TypeSpec, TypedVal};
use crate::value::Blob;
use crate::{SemId, StrictVal, Ty, TypeRef, TypeSystem};

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// unknown type `{0}`.
    TypeAbsent(TypeSpec),

    #[display(inner)]
    #[from]
    UnknownType(UnknownType),

    /// {0} is not yet implemented. Please update `strict_types` to the latest version.
    NotImplemented(String),

    #[display(inner)]
    #[from]
    Decode(DecodeError),

    /// data provided to reify operation are not entirely consumed during deserialization.
    NotEntirelyConsumed,
}

impl SymbolicSys {
    pub fn strict_deserialize_type(
        &self,
        spec: impl Into<TypeSpec>,
        data: &[u8],
    ) -> Result<TypedVal, Error> {
        let spec = spec.into();
        let sem_id = self.to_sem_id(spec.clone()).ok_or(Error::TypeAbsent(spec))?;
        self.as_types().strict_deserialize_type(sem_id, data)
    }

    pub fn strict_read_type(
        &self,
        spec: impl Into<TypeSpec>,
        d: &mut impl ReadRaw,
    ) -> Result<TypedVal, Error> {
        let spec = spec.into();
        let sem_id = self.to_sem_id(spec.clone()).ok_or(Error::TypeAbsent(spec))?;
        self.as_types().strict_read_type(sem_id, d)
    }
}

impl TypeSystem {
    fn strict_read_list(
        &self,
        len: usize,
        ty: SemId,
        d: &mut impl ReadRaw,
    ) -> Result<Vec<StrictVal>, Error> {
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            let item = self.strict_read_type(ty, d)?;
            list.push(item.val);
        }
        Ok(list)
    }

    fn strict_read_map(
        &self,
        len: usize,
        key_ty: SemId,
        ty: SemId,
        d: &mut impl ReadRaw,
    ) -> Result<Vec<(StrictVal, StrictVal)>, Error> {
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            let key = self.strict_read_type(key_ty, d)?;
            let item = self.strict_read_type(ty, d)?;
            list.push((key.val, item.val));
        }
        Ok(list)
    }

    pub fn strict_deserialize_type(&self, sem_id: SemId, data: &[u8]) -> Result<TypedVal, Error> {
        let mut cursor = StreamReader::cursor::<MAX32>(data);
        let ty = self.strict_read_type(sem_id, &mut cursor)?;
        if cursor.unconfine().position() as usize != data.len() {
            return Err(Error::NotEntirelyConsumed);
        }
        Ok(ty)
    }

    pub fn strict_read_type(
        &self,
        sem_id: SemId,
        mut d: &mut impl ReadRaw,
    ) -> Result<TypedVal, Error> {
        let spec = TypeSpec::from(sem_id);
        let ty = self.find(sem_id).ok_or_else(|| Error::TypeAbsent(spec.clone()))?;

        let mut reader = StrictReader::with(d);

        let val = match ty {
            Ty::Primitive(prim) => {
                match *prim {
                    Primitive::UNIT => StrictVal::Unit,
                    Primitive::BYTE => StrictVal::num(u8::strict_decode(&mut reader)?),
                    Primitive::U8 => StrictVal::num(u8::strict_decode(&mut reader)?),
                    Primitive::U16 => StrictVal::num(u16::strict_decode(&mut reader)?),
                    Primitive::U24 => StrictVal::num(u24::strict_decode(&mut reader)?.into_u32()),
                    Primitive::U32 => StrictVal::num(u32::strict_decode(&mut reader)?),
                    Primitive::U40 => StrictVal::num(u40::strict_decode(&mut reader)?),
                    Primitive::U48 => StrictVal::num(u48::strict_decode(&mut reader)?),
                    Primitive::U56 => StrictVal::num(u56::strict_decode(&mut reader)?),
                    Primitive::U64 => StrictVal::num(u64::strict_decode(&mut reader)?),
                    // Primitive::U128 => StrictVal::num(u128::strict_decode(&mut reader)?),
                    Primitive::I8 => StrictVal::num(i8::strict_decode(&mut reader)?),
                    Primitive::I16 => StrictVal::num(i16::strict_decode(&mut reader)?),
                    // I24 => StrictVal::num(i24::strict_decode(&mut reader)?),
                    Primitive::I32 => StrictVal::num(i32::strict_decode(&mut reader)?),
                    Primitive::I64 => StrictVal::num(i64::strict_decode(&mut reader)?),
                    // Primitive::I128 => StrictVal::num(i128::strict_decode(&mut reader)?),
                    other => {
                        return Err(Error::NotImplemented(format!(
                            "loading {other} into a typed value is not yet implemented"
                        )));
                    }
                }
            }
            Ty::UnicodeChar => {
                todo!()
            }

            // ASCII strings:
            Ty::List(sem_id, sizing)
                if self
                    .find(*sem_id)
                    .ok_or_else(|| Error::TypeAbsent(spec.clone()))?
                    .is_char_enum() =>
            {
                if sizing.max <= u8::MAX as u64 {
                    StrictVal::String(TinyAscii::strict_decode(&mut reader)?.to_string())
                } else if sizing.max <= u16::MAX as u64 {
                    StrictVal::String(SmallAscii::strict_decode(&mut reader)?.to_string())
                } else if sizing.max <= u24::MAX.into_u64() {
                    StrictVal::String(MediumAscii::strict_decode(&mut reader)?.to_string())
                } else if sizing.max <= u32::MAX as u64 {
                    StrictVal::String(LargeAscii::strict_decode(&mut reader)?.to_string())
                } else {
                    StrictVal::String(
                        Confined::<AsciiString, 0, { u64::MAX as usize }>::strict_decode(
                            &mut reader,
                        )?
                        .to_string(),
                    )
                }
            }
            // Restricted strings:
            Ty::Tuple(fields) if self.is_rstring(fields)? => {
                let (_, sizing) = self.rstring_sizing(fields)?.expect("checked in match");
                if sizing.max <= u8::MAX as u64 {
                    StrictVal::String(TinyAscii::strict_decode(&mut reader)?.to_string())
                } else if sizing.max <= u16::MAX as u64 {
                    StrictVal::String(SmallAscii::strict_decode(&mut reader)?.to_string())
                } else if sizing.max <= u24::MAX.into_u64() {
                    StrictVal::String(MediumAscii::strict_decode(&mut reader)?.to_string())
                } else if sizing.max <= u32::MAX as u64 {
                    StrictVal::String(LargeAscii::strict_decode(&mut reader)?.to_string())
                } else {
                    StrictVal::String(
                        Confined::<AsciiString, 0, { u64::MAX as usize }>::strict_decode(
                            &mut reader,
                        )?
                        .to_string(),
                    )
                }
            }

            Ty::Enum(variants) => {
                let tag = u8::strict_decode(&mut reader)?;
                let Some(name) = variants.name_by_tag(tag) else {
                    return Err(DecodeError::EnumTagNotKnown(spec.to_string(), tag).into());
                };
                StrictVal::enumer(name.clone())
            }
            Ty::Union(variants) => {
                let tag = u8::strict_decode(&mut reader)?;
                let Some((variant, ty)) = variants.by_tag(tag) else {
                    return Err(DecodeError::EnumTagNotKnown(spec.to_string(), tag).into());
                };
                let fields = self.strict_read_type(*ty, reader.unbox())?;
                StrictVal::union(variant.name.clone(), fields.val)
            }
            Ty::Tuple(reqs) => {
                let mut fields = Vec::with_capacity(reqs.len());
                let d = reader.unbox();
                for ty in reqs {
                    let checked = self.strict_read_type(*ty, d)?;
                    fields.push(checked.val);
                }
                StrictVal::tuple(fields)
            }
            Ty::Struct(reqs) => {
                let mut fields = IndexMap::with_capacity(reqs.len());
                let d = reader.unbox();
                for field in reqs {
                    let checked = self.strict_read_type(field.ty, d)?;
                    fields.insert(field.name.clone(), checked.val);
                }
                StrictVal::Struct(fields)
            }

            // Fixed-size arrays:
            Ty::Array(ty, len) if ty.is_byte() => {
                let d = reader.unbox();
                let buf = d.read_raw::<MAX16>(*len as usize).map_err(DecodeError::from)?;
                StrictVal::Bytes(Blob(buf))
            }
            Ty::Array(ty, len) => {
                let mut list = Vec::<StrictVal>::with_capacity(*len as usize);
                let d = reader.unbox();
                for _ in 0..*len {
                    let checked = self.strict_read_type(*ty, d)?;
                    list.push(checked.val);
                }
                StrictVal::List(list)
            }

            // Byte strings:
            Ty::List(ty, sizing) if ty.is_byte() && sizing.max <= u8::MAX as u64 => {
                let string = TinyBlob::strict_decode(&mut reader)?;
                StrictVal::Bytes(Blob(string.release()))
            }
            Ty::List(ty, sizing) if ty.is_byte() && sizing.max <= u16::MAX as u64 => {
                let string = SmallBlob::strict_decode(&mut reader)?;
                StrictVal::Bytes(Blob(string.release()))
            }
            Ty::List(ty, sizing) if ty.is_byte() && sizing.max <= u24::MAX.into_u64() => {
                let string = MediumBlob::strict_decode(&mut reader)?;
                StrictVal::Bytes(Blob(string.release()))
            }
            Ty::List(ty, sizing) if ty.is_byte() && sizing.max <= u32::MAX as u64 => {
                let string = LargeBlob::strict_decode(&mut reader)?;
                StrictVal::Bytes(Blob(string.release()))
            }

            // Unicode strings:
            Ty::List(ty, sizing) if ty.is_unicode_char() && sizing.max <= u8::MAX as u64 => {
                let string = TinyString::strict_decode(&mut reader)?;
                StrictVal::String(string.release())
            }
            Ty::List(ty, sizing) if ty.is_unicode_char() && sizing.max <= u16::MAX as u64 => {
                let string = SmallString::strict_decode(&mut reader)?;
                StrictVal::String(string.release())
            }
            Ty::List(ty, sizing) if ty.is_unicode_char() && sizing.max <= u24::MAX.into_u64() => {
                let string = MediumString::strict_decode(&mut reader)?;
                StrictVal::String(string.release())
            }
            Ty::List(ty, sizing) if ty.is_unicode_char() && sizing.max <= u32::MAX as u64 => {
                let string = LargeString::strict_decode(&mut reader)?;
                StrictVal::String(string.release())
            }

            // Other lists:
            Ty::List(ty, sizing) if sizing.max <= u8::MAX as u64 => {
                let len = u8::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len as usize, *ty, d)?;
                StrictVal::List(list)
            }
            Ty::List(ty, sizing) if sizing.max <= u16::MAX as u64 => {
                let len = u16::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len as usize, *ty, d)?;
                StrictVal::List(list)
            }
            Ty::List(ty, sizing) if sizing.max <= u24::MAX.into_u64() => {
                let len = u24::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len.into_usize(), *ty, d)?;
                StrictVal::List(list)
            }
            Ty::List(ty, sizing) if sizing.max <= u32::MAX as u64 => {
                let len = u32::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len as usize, *ty, d)?;
                StrictVal::List(list)
            }
            Ty::List(ty, _) => {
                let len = u64::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len as usize, *ty, d)?;
                StrictVal::List(list)
            }
            // TODO: Find a way to check for the uniqueness of the set values
            Ty::Set(ty, sizing) if sizing.max <= u8::MAX as u64 => {
                let len = u8::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len as usize, *ty, d)?;
                StrictVal::Set(list)
            }
            Ty::Set(ty, sizing) if sizing.max <= u16::MAX as u64 => {
                let len = u16::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len as usize, *ty, d)?;
                StrictVal::Set(list)
            }
            Ty::Set(ty, sizing) if sizing.max <= u24::MAX.into_u64() => {
                let len = u24::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len.into_usize(), *ty, d)?;
                StrictVal::Set(list)
            }
            Ty::Set(ty, sizing) if sizing.max <= u32::MAX as u64 => {
                let len = u32::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len as usize, *ty, d)?;
                StrictVal::Set(list)
            }
            Ty::Set(ty, _) => {
                let len = u64::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_list(len as usize, *ty, d)?;
                StrictVal::Set(list)
            }
            Ty::Map(key_id, id, sizing) if sizing.max <= u8::MAX as u64 => {
                let len = u8::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_map(len as usize, *key_id, *id, d)?;
                StrictVal::Map(list)
            }
            Ty::Map(key_id, id, sizing) if sizing.max <= u16::MAX as u64 => {
                let len = u16::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_map(len as usize, *key_id, *id, d)?;
                StrictVal::Map(list)
            }
            Ty::Map(key_id, id, sizing) if sizing.max <= u24::MAX.into_u64() => {
                let len = u24::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_map(len.into_usize(), *key_id, *id, d)?;
                StrictVal::Map(list)
            }
            Ty::Map(key_id, id, sizing) if sizing.max <= u32::MAX as u64 => {
                let len = u32::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_map(len as usize, *key_id, *id, d)?;
                StrictVal::Map(list)
            }
            Ty::Map(key_id, id, _sizing) => {
                let len = u64::strict_decode(&mut reader)?;
                d = reader.unbox();
                let list = self.strict_read_map(len as usize, *key_id, *id, d)?;
                StrictVal::Map(list)
            }
        };

        Ok(TypedVal {
            val,
            orig: TypeSymbol::unnamed(sem_id),
        })
    }
}

#[cfg(test)]
mod test {
    use super::super::test_helpers::*;
    // use super::*;

    #[test]
    fn typify() {
        let sys = test_system();
        //let nominal = Nominal::with("TICK", "Some name", 2);
        let value = ston!(name "Some name", ticker "TICK", precision svenum!(2));
        let checked = sys.typify(value, "TestLib.Nominal").unwrap();
        assert_eq!(
            format!("{}", checked.val),
            r#"name "Some name", ticker "TICK", precision twoDecimals"#
        );
    }
}
