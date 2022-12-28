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
use std::ops::Deref;

use amplify::confinement::TinyOrdMap;

use crate::ast::{Field, Fields, Step, Variants};
use crate::encoding::{
    DefineTuple, DefineUnion, StrictEncode, TypedWrite, WriteStruct, WriteTuple, WriteUnion,
};
use crate::util::Sizing;
use crate::{FieldName, KeyTy, SemId, Ty, TypeRef, STEN_LIB};

impl StrictEncode for SemId {
    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_type(libname!(STEN_LIB), tn!("SemId"), self.as_bytes())
    }
}

impl StrictEncode for Step {
    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_union(libname!(STEN_LIB), tn!("Step"), |udef| {
            let uwriter = udef
                .define_type::<FieldName>(fname!("namedField"))
                .define_type::<u8>(fname!("unnamedField"))
                .define_unit(fname!("index"))
                .define_unit(fname!("list"))
                .define_unit(fname!("set"))
                .define_unit(fname!("map"))
                .complete();

            Ok(match self {
                Step::NamedField(name) => uwriter.write_type(fname!("namedField"), name),
                Step::UnnamedField(ord) => uwriter.write_type(fname!("unnamedField"), ord),
                Step::Index => uwriter.write_unit(fname!("index")),
                Step::List => uwriter.write_unit(fname!("list")),
                Step::Set => uwriter.write_unit(fname!("set")),
                Step::Map => uwriter.write_unit(fname!("map")),
            }?
            .complete())
        })
    }
}

impl StrictEncode for Sizing {
    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_struct(libname!(STEN_LIB), tn!("Sizing"), |sw| {
            Ok(sw
                .write_field(fname!("min"), &self.min)?
                .write_field(fname!("max"), &self.max)?
                .complete())
        })
    }
}

impl StrictEncode for Field {
    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_struct(libname!(STEN_LIB), tn!("Field"), |sw| {
            Ok(sw
                .write_field(fname!("name"), &self.name)?
                .write_field(fname!("ord"), &self.ord)?
                .complete())
        })
    }
}

impl<Ref: TypeRef, const OP: bool> StrictEncode for Fields<Ref, OP> {
    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        struct FieldInfo<R: TypeRef> {
            name: Option<FieldName>,
            ty: R,
        }
        impl<R: TypeRef> StrictEncode for FieldInfo<R> {
            unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
                writer.write_struct(libname!(STEN_LIB), tn!("Field{}", R::TYPE_NAME), |sw| {
                    Ok(sw
                        .write_field(fname!("name"), &self.name)?
                        .write_field(fname!("ty"), &self.ty)?
                        .complete())
                })
            }
        }

        let fields = TinyOrdMap::try_from_iter(self.iter().map(|(field, ty)| {
            (field.ord, FieldInfo {
                name: field.name.clone(),
                ty: ty.clone(),
            })
        }))
        .expect("guaranteed by Fields type");
        writer.write_type(libname!(STEN_LIB), tn!("FieldList{}", Ref::TYPE_NAME), &fields)
    }
}

impl StrictEncode for Variants {
    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_type(libname!(STEN_LIB), tn!("Variants"), self.deref())
    }
}

impl<Ref: TypeRef> StrictEncode for Ty<Ref> {
    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_union(libname!(STEN_LIB), tn!("Ty{}", Ref::TYPE_NAME), |u| {
            let u = u
                .define_type::<u8>(fname!("primitive"))
                .define_unit(fname!("unicode"))
                .define_type::<Variants>(fname!("enum"))
                .define_type::<Fields<Ref, false>>(fname!("union"))
                .define_type::<Fields<Ref, true>>(fname!("struct"));
            let u = u
                .define_tuple(fname!("array"))
                .define_field::<Ref>()
                .define_field::<u16>()
                .complete();
            let u = u
                .define_tuple(fname!("list"))
                .define_field::<Ref>()
                .define_field::<Sizing>()
                .complete();
            let u = u
                .define_tuple(fname!("set"))
                .define_field::<Ref>()
                .define_field::<Sizing>()
                .complete();
            let u = u
                .define_tuple(fname!("map"))
                .define_field::<KeyTy>()
                .define_field::<Ref>()
                .define_field::<Sizing>()
                .complete();

            let u = u.complete();

            let u = match self {
                Ty::Primitive(prim) => u.write_type(fname!("primitive"), &prim.into_code())?,
                Ty::UnicodeChar => u.write_unit(fname!("unicode"))?,
                Ty::Enum(vars) => u.write_type(fname!("enum"), vars)?,
                Ty::Union(fields) => u.write_type(fname!("union"), fields)?,
                Ty::Struct(fields) => u.write_type(fname!("struct"), fields)?,
                Ty::Array(ty, len) => {
                    u.write_tuple(fname!("array"))?.write_field(ty)?.write_field(len)?.complete()
                }
                Ty::List(ty, sizing) => {
                    u.write_tuple(fname!("list"))?.write_field(ty)?.write_field(sizing)?.complete()
                }
                Ty::Set(ty, sizing) => {
                    u.write_tuple(fname!("set"))?.write_field(ty)?.write_field(sizing)?.complete()
                }
                Ty::Map(key, ty, sizing) => u
                    .write_tuple(fname!("map"))?
                    .write_field(key)?
                    .write_field(ty)?
                    .write_field(sizing)?
                    .complete(),
            };

            Ok(u.complete())
        })
    }
}

impl StrictEncode for KeyTy {
    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_union(libname!(STEN_LIB), tn!("KeyTy"), |u| {
            let u = u
                .define_type::<u8>(fname!("primitive"))
                .define_type::<Variants>(fname!("enum"))
                .define_type::<u16>(fname!("array"))
                .define_type::<Sizing>(fname!("unicode"))
                .define_type::<Sizing>(fname!("ascii"))
                .define_type::<Sizing>(fname!("bytes"))
                .complete();

            let u = match self {
                KeyTy::Primitive(prim) => u.write_type(fname!("primitive"), &prim.into_code())?,
                KeyTy::Enum(vars) => u.write_type(fname!("enum"), vars)?,
                KeyTy::Array(len) => u.write_type(fname!("array"), len)?,
                KeyTy::UnicodeStr(sizing) => u.write_type(fname!("unicode"), sizing)?,
                KeyTy::AsciiStr(sizing) => u.write_type(fname!("ascii"), sizing)?,
                KeyTy::Bytes(sizing) => u.write_type(fname!("bytes"), sizing)?,
            };

            Ok(u.complete())
        })
    }
}
