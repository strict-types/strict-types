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

use std::io;

use amplify::confinement::Confined;
use amplify::num::u24;
use encoding::constants::UNIT;
use encoding::{
    SerializeError, Sizing, StrictEncode, StrictSerialize, StrictType, TypeName, TypedWrite,
};

use crate::typify::TypedVal;
use crate::value::{EnumTag, StrictNum};
use crate::{SemId, StrictVal, Ty, TypeRef, TypeSystem};

#[derive(Clone, Debug)]
pub struct SerializedType<const MAX_LEN: usize>(Confined<Vec<u8>, 0, MAX_LEN>);

#[doc(hidden)]
impl<const MAX_LEN: usize> StrictType for SerializedType<MAX_LEN> {
    const STRICT_LIB_NAME: &'static str = "";
    fn strict_name() -> Option<TypeName> { None }
}
#[doc(hidden)]
impl<const MAX_LEN: usize> StrictEncode for SerializedType<MAX_LEN> {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        self.0.strict_encode(writer)
    }
}
impl<const MAX_LEN: usize> StrictSerialize for SerializedType<MAX_LEN> {}

impl TypeSystem {
    pub fn strict_serialize_type<const MAX_LEN: usize>(
        &self,
        typed: &TypedVal,
    ) -> Result<SerializedType<MAX_LEN>, SerializeError> {
        let mut buf = Vec::new();
        self.strict_write_type(typed, &mut buf)?;
        Confined::try_from(buf).map(SerializedType).map_err(SerializeError::from)
    }

    pub fn strict_write_type(
        &self,
        typed: &TypedVal,
        writer: &mut impl io::Write,
    ) -> Result<(), io::Error> {
        let sem_id = self.to_sem_id(typed.as_spec()).expect("typified with some other TypeSystem");
        self.strict_write_value(&typed.val, sem_id, writer)
    }

    fn strict_write_value(
        &self,
        val: &StrictVal,
        sem_id: SemId,
        writer: &mut impl io::Write,
    ) -> Result<(), io::Error> {
        let info = self.find(&sem_id.into()).expect("typified with some other TypeSystem");
        match (&val, &info.ty) {
            (StrictVal::Unit, Ty::Primitive(prim)) => {
                debug_assert_eq!(*prim, UNIT);
                // Do nothing
            }
            (StrictVal::Number(StrictNum::Uint(num)), Ty::Primitive(prim)) => {
                let bytes_count = prim.byte_size() as usize;
                let le_bytes = &num.to_le_bytes()[0..bytes_count];
                writer.write_all(le_bytes)?;
            }
            (StrictVal::Number(StrictNum::BigUint(num)), Ty::Primitive(prim)) => {
                let bytes_count = prim.byte_size() as usize;
                let le_bytes = &num.to_le_bytes()[0..bytes_count];
                writer.write_all(le_bytes)?;
            }
            (StrictVal::Number(StrictNum::Int(num)), Ty::Primitive(prim)) => {
                let bytes_count = prim.byte_size() as usize;
                let le_bytes = &num.to_le_bytes()[0..bytes_count];
                writer.write_all(le_bytes)?;
            }
            (StrictVal::Number(StrictNum::BigInt(num)), Ty::Primitive(prim)) => {
                let bytes_count = prim.byte_size() as usize;
                let le_bytes = &num.to_le_bytes()[0..bytes_count];
                writer.write_all(le_bytes)?;
            }

            (StrictVal::String(s), Ty::UnicodeChar) => {
                debug_assert_eq!(s.chars().count(), 1);
                writer.write_all(s.as_bytes())?;
            }
            (StrictVal::Bytes(vec), Ty::Array(_, len)) => {
                debug_assert_eq!(vec.len(), *len as usize);
                writer.write_all(&vec)?;
            }
            (StrictVal::String(s), Ty::Array(_, len)) => {
                debug_assert_eq!(s.len(), *len as usize);
                writer.write_all(s.as_bytes())?;
            }

            (StrictVal::Tuple(vals), Ty::Tuple(fields)) => {
                debug_assert_eq!(vals.len(), fields.len());
                for (val, sem_id) in vals.iter().zip(fields) {
                    self.strict_write_value(val, *sem_id, writer)?;
                }
            }
            (StrictVal::Struct(vals), Ty::Struct(fields)) => {
                debug_assert_eq!(vals.len(), fields.len());
                for (val, field) in vals.values().zip(fields) {
                    self.strict_write_value(val, field.ty, writer)?;
                }
            }
            (StrictVal::Enum(EnumTag::Ord(tag)), Ty::Enum(variants)) => {
                debug_assert!(variants.has_tag(*tag));
                writer.write_all(&[*tag])?;
            }
            (StrictVal::Enum(EnumTag::Name(tag)), Ty::Enum(variants)) => {
                let tag = variants.tag_by_name(tag).expect("Type::System::typify guarantees");
                writer.write_all(&[tag])?;
            }
            (StrictVal::Union(EnumTag::Ord(tag), val), Ty::Union(variants)) => {
                let sem_id = variants.ty_by_tag(*tag).expect("Type::System::typify guarantees");
                writer.write_all(&[*tag])?;
                self.strict_write_value(val, *sem_id, writer)?;
            }
            (StrictVal::Union(EnumTag::Name(tag), val), Ty::Union(variants)) => {
                let (variant, sem_id) =
                    variants.by_name(tag).expect("Type::System::typify guarantees");
                writer.write_all(&[variant.tag])?;
                self.strict_write_value(val, *sem_id, writer)?;
            }

            (StrictVal::String(s), Ty::List(_, sizing)) => {
                let bytes_count = sizing.byte_size() as usize;
                let le_bytes = &s.len().to_le_bytes()[0..bytes_count];
                writer.write_all(le_bytes)?;
                writer.write_all(s.as_bytes())?;
            }
            (StrictVal::Bytes(s), Ty::List(_, sizing)) => {
                let bytes_count = sizing.byte_size() as usize;
                let le_bytes = &s.len().to_le_bytes()[0..bytes_count];
                writer.write_all(le_bytes)?;
                writer.write_all(s)?;
            }
            (StrictVal::List(list), Ty::List(sem_id, sizing))
            | (StrictVal::Set(list), Ty::Set(sem_id, sizing)) => {
                let bytes_count = sizing.byte_size() as usize;
                let le_bytes = &list.len().to_le_bytes()[0..bytes_count];
                writer.write_all(le_bytes)?;
                for val in list {
                    self.strict_write_value(val, *sem_id, writer)?;
                }
            }
            (StrictVal::Map(list), Ty::Map(key_id, sem_id, sizing)) => {
                let bytes_count = sizing.byte_size() as usize;
                let le_bytes = &list.len().to_le_bytes()[0..bytes_count];
                writer.write_all(le_bytes)?;
                for (key, val) in list {
                    self.strict_write_value(key, key_id.id(), writer)?;
                    self.strict_write_value(val, *sem_id, writer)?;
                }
            }

            (a, b) => panic!("bug in business logic of type system. Details:\n{a:#?}\n{b:#?}"),
        }

        Ok(())
    }
}

trait SizingExt {
    fn byte_size(&self) -> usize;
}

impl SizingExt for Sizing {
    fn byte_size(&self) -> usize {
        match self.max {
            one if one <= u8::MAX as u64 => 1,
            two if two <= u16::MAX as u64 => 2,
            three if three <= u24::MAX.into_u64() => 3,
            four if four <= u32::MAX as u64 => 4,
            eight if eight <= u64::MAX as u64 => 8,
            _ => unreachable!(),
        }
    }
}
