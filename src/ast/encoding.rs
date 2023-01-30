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

use std::collections::BTreeMap;
use std::io;

use amplify::confinement::TinyOrdMap;
use strict_encoding::{
    DecodeError, FieldName, ReadTuple, StrictDecode, StrictDumb, StrictEncode, StrictType,
    TypedRead, TypedWrite, Variant, STEN_LIB,
};

use crate::ast::ty::UnionVariants;
use crate::{SemId, TypeRef};

impl StrictEncode for SemId {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_newtype::<Self>(self.as_bytes())
    }
}
impl StrictDecode for SemId {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_tuple(|r| r.read_field::<[u8; 32]>().map(SemId::from))
    }
}

#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STEN_LIB)]
struct VariantInfo<Ref: TypeRef> {
    name: FieldName,
    ty: Ref,
}

impl<Ref: TypeRef> StrictEncode for UnionVariants<Ref> {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        let fields = TinyOrdMap::try_from_iter(self.iter().map(|(variant, ty)| {
            (variant.tag, VariantInfo {
                name: variant.name.clone(),
                ty: ty.clone(),
            })
        }))
        .expect("guaranteed by Variant type");
        writer.write_newtype::<Self>(&fields)
    }
}
impl<Ref: TypeRef> StrictDecode for UnionVariants<Ref> {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        let read = TinyOrdMap::<u8, VariantInfo<Ref>>::strict_decode(reader)?;
        let mut inner = BTreeMap::new();
        for (tag, info) in read {
            inner.insert(
                Variant {
                    name: info.name,
                    tag,
                },
                info.ty,
            );
        }
        UnionVariants::try_from(inner).map_err(DecodeError::from)
    }
}
