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

use strict_encoding::{
    DecodeError, DefineTuple, DefineUnion, LibName, ReadTuple, ReadUnion, StrictDecode, StrictDumb,
    StrictEncode, StrictSum, StrictType, StrictUnion, TypeName, TypedRead, TypedWrite, WriteTuple,
    WriteUnion, STEN_LIB,
};

use crate::typelib::{CompileRef, InlineRef, InlineRef1, InlineRef2};
use crate::{LibRef, SemId, Ty};

macro_rules! impl_strict {
    ($ty:ty) => {
        impl StrictDumb for $ty {
            fn strict_dumb() -> Self { Self::Inline(Ty::UnicodeChar.into()) }
        }
        impl StrictType for $ty {
            const STRICT_LIB_NAME: &'static str = STEN_LIB;
        }
        impl StrictSum for $ty {
            const ALL_VARIANTS: &'static [(u8, &'static str)] =
                &[(0, "inline"), (1, "named"), (2, "extern")];
            fn variant_name(&self) -> &'static str {
                match self {
                    Self::Inline(_) => Self::ALL_VARIANTS[0].1,
                    Self::Named(_, _) => Self::ALL_VARIANTS[1].1,
                    Self::Extern(_, _, _) => Self::ALL_VARIANTS[2].1,
                }
            }
        }
        impl StrictUnion for $ty {}
        impl StrictEncode for $ty {
            fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
                writer.write_union::<Self>(|d| {
                    let w = d
                        .define_newtype::<Ty<InlineRef>>(fname!("inline"))
                        .define_tuple(fname!("named"), |w| {
                            w.define_field::<TypeName>().define_field::<SemId>().complete()
                        })
                        .define_tuple(fname!("extern"), |w| {
                            w.define_field::<TypeName>()
                                .define_field::<LibName>()
                                .define_field::<SemId>()
                                .complete()
                        })
                        .complete();

                    Ok(match self {
                        Self::Inline(ty) => w.write_newtype(fname!("inline"), ty)?,
                        Self::Named(ty_name, id) => w.write_tuple(fname!("named"), |w| {
                            Ok(w.write_field(ty_name)?.write_field(id)?.complete())
                        })?,
                        Self::Extern(ty_name, lib_name, id) => {
                            w.write_tuple(fname!("named"), |w| {
                                Ok(w.write_field(ty_name)?
                                    .write_field(lib_name)?
                                    .write_field(id)?
                                    .complete())
                            })?
                        }
                    }
                    .complete())
                })
            }
        }
        impl StrictDecode for $ty {
            fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
                reader.read_union(|field_name, r| match field_name.as_str() {
                    "inline" => r.read_newtype::<Self, Ty<_>>(),
                    "named" => r.read_tuple(|r| {
                        let name = r.read_field()?;
                        let id = r.read_field()?;
                        Ok(Self::Named(name, id))
                    }),
                    "extern" => r.read_tuple(|r| {
                        let name = r.read_field()?;
                        let lib = r.read_field()?;
                        let id = r.read_field()?;
                        Ok(Self::Extern(name, lib, id))
                    }),
                    _ => unreachable!("invalid field name"),
                })
            }
        }
    };
}

impl_strict!(LibRef);
impl_strict!(InlineRef);
impl_strict!(InlineRef1);
impl_strict!(InlineRef2);

impl StrictDumb for CompileRef {
    fn strict_dumb() -> Self { Self::Embedded(Ty::UnicodeChar.into()) }
}
impl StrictType for CompileRef {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictSum for CompileRef {
    const ALL_VARIANTS: &'static [(u8, &'static str)] =
        &[(0, "embedded"), (1, "named"), (2, "extern")];
    fn variant_name(&self) -> &'static str {
        match self {
            Self::Embedded(_) => Self::ALL_VARIANTS[0].1,
            Self::Named(_) => Self::ALL_VARIANTS[1].1,
            Self::Extern(_, _) => Self::ALL_VARIANTS[2].1,
        }
    }
}
impl StrictUnion for CompileRef {}
impl StrictEncode for CompileRef {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        writer.write_union::<Self>(|d| {
            let w = d
                .define_newtype::<Ty<InlineRef>>(fname!("embedded"))
                .define_tuple(fname!("named"), |w| {
                    w.define_field::<TypeName>().define_field::<SemId>().complete()
                })
                .define_tuple(fname!("extern"), |w| {
                    w.define_field::<TypeName>()
                        .define_field::<LibName>()
                        .define_field::<SemId>()
                        .complete()
                })
                .complete();

            Ok(match self {
                Self::Embedded(ty) => w.write_newtype(fname!("embedded"), ty.as_ref())?,
                Self::Named(ty_name) => w.write_newtype(fname!("named"), ty_name)?,
                Self::Extern(ty_name, lib_name) => w.write_tuple(fname!("named"), |w| {
                    Ok(w.write_field(ty_name)?.write_field(lib_name)?.complete())
                })?,
            }
            .complete())
        })
    }
}
impl StrictDecode for CompileRef {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_union(|field_name, r| match field_name.as_str() {
            "embedded" => r.read_newtype::<Self, Ty<_>>(),
            "named" => r.read_newtype::<Self, TypeName>(),
            "extern" => r.read_tuple(|r| {
                let name = r.read_field()?;
                let lib = r.read_field()?;
                Ok(Self::Extern(name, lib))
            }),
            _ => unreachable!("invalid field name"),
        })
    }
}
