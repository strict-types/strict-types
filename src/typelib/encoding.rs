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
use crate::typelib::{InlineRef, InlineRef1, InlineRef2};
use crate::{KeyTy, LibName, LibRef, SemId, Ty, TypeName};

impl StrictEncode for LibRef {
    fn strict_encode_dumb() -> Self { LibRef::Named(tn!("Some"), SemId::strict_encode_dumb()) }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        let u = writer
            .define_union(Some("LibRef"))
            .define_type::<Ty<InlineRef>>("inline")
            .define_tuple("named")
            .define_field::<TypeName>()
            .define_field::<SemId>()
            .complete()
            .define_tuple("extern")
            .define_field::<TypeName>()
            .define_field::<LibName>()
            .define_field::<SemId>()
            .complete()
            .complete();

        Ok(match self {
            LibRef::Inline(ty) => u.write_type("inline", ty)?,
            LibRef::Named(ty_name, id) => {
                u.write_tuple("named")?.write_field(ty_name)?.write_field(id)?.complete()
            }
            LibRef::Extern(ty_name, lib_name, id) => u
                .write_tuple("named")?
                .write_field(ty_name)?
                .write_field(lib_name)?
                .write_field(id)?
                .complete(),
        }
        .complete())
    }
}

impl StrictEncode for InlineRef {
    fn strict_encode_dumb() -> Self { InlineRef::Named(tn!("Some"), SemId::strict_encode_dumb()) }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        let u = writer
            .define_union(Some("InlineRef"))
            .define_type::<Ty<InlineRef1>>("inline")
            .define_tuple("named")
            .define_field::<TypeName>()
            .define_field::<SemId>()
            .complete()
            .define_tuple("extern")
            .define_field::<TypeName>()
            .define_field::<LibName>()
            .define_field::<SemId>()
            .complete()
            .complete();

        Ok(match self {
            InlineRef::Builtin(ty) => u.write_type("inline", ty)?,
            InlineRef::Named(ty_name, id) => {
                u.write_tuple("named")?.write_field(ty_name)?.write_field(id)?.complete()
            }
            InlineRef::Extern(ty_name, lib_name, id) => u
                .write_tuple("named")?
                .write_field(ty_name)?
                .write_field(lib_name)?
                .write_field(id)?
                .complete(),
        }
        .complete())
    }
}

impl StrictEncode for InlineRef1 {
    fn strict_encode_dumb() -> Self { InlineRef1::Named(tn!("Some"), SemId::strict_encode_dumb()) }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        let u = writer
            .define_union(Some("InlineRef1"))
            .define_type::<Ty<InlineRef2>>("inline")
            .define_tuple("named")
            .define_field::<TypeName>()
            .define_field::<SemId>()
            .complete()
            .define_tuple("extern")
            .define_field::<TypeName>()
            .define_field::<LibName>()
            .define_field::<SemId>()
            .complete()
            .complete();

        Ok(match self {
            InlineRef1::Builtin(ty) => u.write_type("inline", ty)?,
            InlineRef1::Named(ty_name, id) => {
                u.write_tuple("named")?.write_field(ty_name)?.write_field(id)?.complete()
            }
            InlineRef1::Extern(ty_name, lib_name, id) => u
                .write_tuple("named")?
                .write_field(ty_name)?
                .write_field(lib_name)?
                .write_field(id)?
                .complete(),
        }
        .complete())
    }
}

impl StrictEncode for InlineRef2 {
    fn strict_encode_dumb() -> Self { InlineRef2::Named(tn!("Some"), SemId::strict_encode_dumb()) }

    fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        let u = writer
            .define_union(Some("InlineRef2"))
            .define_type::<Ty<KeyTy>>("inline")
            .define_tuple("named")
            .define_field::<TypeName>()
            .define_field::<SemId>()
            .complete()
            .define_tuple("extern")
            .define_field::<TypeName>()
            .define_field::<LibName>()
            .define_field::<SemId>()
            .complete()
            .complete();

        Ok(match self {
            InlineRef2::Builtin(ty) => u.write_type("inline", ty)?,
            InlineRef2::Named(ty_name, id) => {
                u.write_tuple("named")?.write_field(ty_name)?.write_field(id)?.complete()
            }
            InlineRef2::Extern(ty_name, lib_name, id) => u
                .write_tuple("named")?
                .write_field(ty_name)?
                .write_field(lib_name)?
                .write_field(id)?
                .complete(),
        }
        .complete())
    }
}
