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
    DecodeError, FieldName, ReadStruct, ReadTuple, StrictDecode, StrictDumb, StrictEncode,
    StrictProduct, StrictStruct, StrictTuple, StrictType, TypedRead, TypedWrite, Variant,
    WriteStruct, STEN_LIB,
};

use crate::ast::ty::{UnionVariants, UnnamedFields};
use crate::ast::{EnumVariants, NamedFields};
use crate::{SemId, TypeRef};

impl StrictType for SemId {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictProduct for SemId {}
impl StrictTuple for SemId {
    const FIELD_COUNT: u8 = 1;
}
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

struct VariantInfo<Ref: TypeRef> {
    name: FieldName,
    ty: Ref,
}
impl<Ref: TypeRef> StrictDumb for VariantInfo<Ref> {
    fn strict_dumb() -> Self {
        Self {
            name: fname!("dumb"),
            ty: Ref::strict_dumb(),
        }
    }
}
impl<Ref: TypeRef> StrictType for VariantInfo<Ref> {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl<Ref: TypeRef> StrictProduct for VariantInfo<Ref> {}
impl<Ref: TypeRef> StrictStruct for VariantInfo<Ref> {
    const ALL_FIELDS: &'static [&'static str] = &["name", "ty"];
}
impl<Ref: TypeRef> StrictEncode for VariantInfo<Ref> {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_struct::<Self>(|sw| {
            Ok(sw
                .write_field(fname!("name"), &self.name)?
                .write_field(fname!("ty"), &self.ty)?
                .complete())
        })
    }
}
impl<Ref: TypeRef> StrictDecode for VariantInfo<Ref> {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_struct(|r| {
            let name = r.read_field(fname!("name"))?;
            let ty = r.read_field(fname!("ty"))?;
            Ok(Self { name, ty })
        })
    }
}

impl<Ref: TypeRef> StrictDumb for NamedFields<Ref> {
    fn strict_dumb() -> Self { fields!("dumb" => Ref::strict_dumb()) }
}
impl<Ref: TypeRef> StrictDumb for UnnamedFields<Ref> {
    fn strict_dumb() -> Self { fields!(Ref::strict_dumb()) }
}
impl<Ref: TypeRef> StrictDumb for UnionVariants<Ref> {
    fn strict_dumb() -> Self { variants!("dumb" => Ref::strict_dumb()) }
}
impl<Ref: TypeRef> StrictType for UnionVariants<Ref> {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl<Ref: TypeRef> StrictProduct for UnionVariants<Ref> {}
impl<Ref: TypeRef> StrictTuple for UnionVariants<Ref> {
    const FIELD_COUNT: u8 = 1;
}
impl<Ref: TypeRef> StrictEncode for UnionVariants<Ref> {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        let fields = TinyOrdMap::try_from_iter(self.iter().map(|(variant, ty)| {
            (variant.ord, VariantInfo {
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
        for (ord, info) in read {
            inner.insert(
                Variant {
                    name: info.name,
                    ord,
                },
                info.ty,
            );
        }
        UnionVariants::try_from(inner).map_err(DecodeError::from)
    }
}

impl StrictDumb for EnumVariants {
    fn strict_dumb() -> Self { variants!("dumb") }
}
