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

use crate::encoding::{DefineTuple, DefineUnion, StrictEncode, TypedWrite, WriteTuple, WriteUnion};
use crate::typelib::{CompileRef, InlineRef, InlineRef1, InlineRef2};
use crate::{KeyTy, LibName, LibRef, SemId, Ty, TypeName, STEN_LIB};

impl StrictEncode for LibRef {
    fn strict_encode_dumb() -> Self { LibRef::Named(tn!("Some"), SemId::strict_encode_dumb()) }

    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        writer.write_union(libname!(STEN_LIB), tn!("LibRef"), |u| {
            let u = u
                .define_type::<Ty<InlineRef>>(fname!("inline"))
                .define_tuple(fname!("named"))
                .define_field::<TypeName>()
                .define_field::<SemId>()
                .complete()
                .define_tuple(fname!("extern"))
                .define_field::<TypeName>()
                .define_field::<LibName>()
                .define_field::<SemId>()
                .complete()
                .complete();

            Ok(match self {
                LibRef::Inline(ty) => u.write_type(fname!("inline"), ty)?,
                LibRef::Named(ty_name, id) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(id)?
                    .complete(),
                LibRef::Extern(ty_name, lib_name, id) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(lib_name)?
                    .write_field(id)?
                    .complete(),
            }
            .complete())
        })
    }
}

impl StrictEncode for CompileRef {
    fn strict_encode_dumb() -> Self { CompileRef::Named(tn!("Some")) }

    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        writer.write_union(libname!(STEN_LIB), tn!("CompileRef"), |u| {
            let u = u
                .define_type::<Ty<CompileRef>>(fname!("inline"))
                .define_tuple(fname!("named"))
                .define_field::<TypeName>()
                .complete()
                .define_tuple(fname!("extern"))
                .define_field::<TypeName>()
                .define_field::<LibName>()
                .complete()
                .complete();

            Ok(match self {
                CompileRef::Inline(ty) => u.write_type(fname!("inline"), ty.as_ref())?,
                CompileRef::Named(ty_name) => {
                    u.write_tuple(fname!("named"))?.write_field(ty_name)?.complete()
                }
                CompileRef::Extern(ty_name, lib_name) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(lib_name)?
                    .complete(),
            }
            .complete())
        })
    }
}

impl StrictEncode for InlineRef {
    fn strict_encode_dumb() -> Self { InlineRef::Named(tn!("Some"), SemId::strict_encode_dumb()) }

    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        writer.write_union(libname!(STEN_LIB), tn!("InlineRef"), |u| {
            let u = u
                .define_type::<Ty<InlineRef1>>(fname!("inline"))
                .define_tuple(fname!("named"))
                .define_field::<TypeName>()
                .define_field::<SemId>()
                .complete()
                .define_tuple(fname!("extern"))
                .define_field::<TypeName>()
                .define_field::<LibName>()
                .define_field::<SemId>()
                .complete()
                .complete();

            Ok(match self {
                InlineRef::Inline(ty) => u.write_type(fname!("inline"), ty)?,
                InlineRef::Named(ty_name, id) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(id)?
                    .complete(),
                InlineRef::Extern(ty_name, lib_name, id) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(lib_name)?
                    .write_field(id)?
                    .complete(),
            }
            .complete())
        })
    }
}

impl StrictEncode for InlineRef1 {
    fn strict_encode_dumb() -> Self { InlineRef1::Named(tn!("Some"), SemId::strict_encode_dumb()) }

    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        writer.write_union(libname!(STEN_LIB), tn!("InlineRef1"), |u| {
            let u = u
                .define_type::<Ty<InlineRef2>>(fname!("inline"))
                .define_tuple(fname!("named"))
                .define_field::<TypeName>()
                .define_field::<SemId>()
                .complete()
                .define_tuple(fname!("extern"))
                .define_field::<TypeName>()
                .define_field::<LibName>()
                .define_field::<SemId>()
                .complete()
                .complete();

            Ok(match self {
                InlineRef1::Inline(ty) => u.write_type(fname!("inline"), ty)?,
                InlineRef1::Named(ty_name, id) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(id)?
                    .complete(),
                InlineRef1::Extern(ty_name, lib_name, id) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(lib_name)?
                    .write_field(id)?
                    .complete(),
            }
            .complete())
        })
    }
}

impl StrictEncode for InlineRef2 {
    fn strict_encode_dumb() -> Self { InlineRef2::Named(tn!("Some"), SemId::strict_encode_dumb()) }

    unsafe fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        writer.write_union(libname!(STEN_LIB), tn!("InlineRef2"), |u| {
            let u = u
                .define_type::<Ty<KeyTy>>(fname!("inline"))
                .define_tuple(fname!("named"))
                .define_field::<TypeName>()
                .define_field::<SemId>()
                .complete()
                .define_tuple(fname!("extern"))
                .define_field::<TypeName>()
                .define_field::<LibName>()
                .define_field::<SemId>()
                .complete()
                .complete();

            Ok(match self {
                InlineRef2::Builtin(ty) => u.write_type(fname!("inline"), ty)?,
                InlineRef2::Named(ty_name, id) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(id)?
                    .complete(),
                InlineRef2::Extern(ty_name, lib_name, id) => u
                    .write_tuple(fname!("named"))?
                    .write_field(ty_name)?
                    .write_field(lib_name)?
                    .write_field(id)?
                    .complete(),
            }
            .complete())
        })
    }
}
