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
use strict_encoding::{
    DecodeError, DefineTuple, DefineUnion, Ident, LibName, ReadStruct, ReadTuple, ReadUnion,
    StrictDecode, StrictDumb, StrictEncode, StrictProduct, StrictSum, StrictTuple,
    StrictType, StrictUnion, TypeName, TypedRead, TypedWrite, WriteStruct, WriteTuple, WriteUnion,
    STEN_LIB,
};

use crate::typelib::type_lib::LibType;
use crate::typelib::{CompileRef, InlineRef, InlineRef1, InlineRef2};
use crate::util::{BuildFragment, PreFragment};
use crate::{Dependency, KeyTy, LibRef, SemId, SemVer, Ty, TypeLib, TypeLibId};

impl StrictType for TypeLibId {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictProduct for TypeLibId {}
impl StrictTuple for TypeLibId {
    const FIELD_COUNT: u8 = 1;
}
impl StrictEncode for TypeLibId {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_newtype::<Self>(self.as_bytes())
    }
}
impl StrictDecode for TypeLibId {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_tuple(|r| r.read_field::<[u8; 32]>().map(TypeLibId::from))
    }
}

impl_strict_struct!(TypeLib, STEN_LIB; 
    name => strict_dumb!(), 
    dependencies => strict_dumb!(), 
    types => confined_bmap!(tn!("DumbType") => LibType::strict_dumb()));
impl_strict_struct!(LibType, STEN_LIB; name, ty);
impl_strict_struct!(Dependency, STEN_LIB; id, name, ver);
impl_strict_struct!(SemVer, STEN_LIB; minor, major, patch, pre, build);

impl StrictDumb for PreFragment {
    fn strict_dumb() -> Self { PreFragment::Digits(0) }
}
impl StrictType for PreFragment {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictSum for PreFragment {
    const ALL_VARIANTS: &'static [(u8, &'static str)] = &[(0, "ident"), (1, "digits")];

    fn variant_name(&self) -> &'static str {
        match self {
            PreFragment::Ident(_) => Self::ALL_VARIANTS[0].1,
            PreFragment::Digits(_) => Self::ALL_VARIANTS[1].1,
        }
    }
}
impl StrictUnion for PreFragment {}
impl StrictEncode for PreFragment {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_union::<Self>(|d| {
            let w = d
                .define_newtype::<Ident>(fname!("ident"))
                .define_newtype::<u128>(fname!("digits"))
                .complete();
            Ok(match self {
                PreFragment::Ident(ident) => w.write_newtype(fname!("ident"), ident)?,
                PreFragment::Digits(ident) => w.write_newtype(fname!("digits"), ident)?,
            }
            .complete())
        })
    }
}
impl StrictDecode for PreFragment {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_union(|fname, r| {
            Ok(match fname.as_str() {
                "ident" => r.read_newtype::<_, Ident>()?,
                "digits" => r.read_newtype::<_, u128>()?,
                _ => unreachable!(),
            })
        })
    }
}

impl StrictDumb for BuildFragment {
    fn strict_dumb() -> Self { BuildFragment::Digits(Ident::strict_dumb()) }
}
impl StrictType for BuildFragment {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictSum for BuildFragment {
    const ALL_VARIANTS: &'static [(u8, &'static str)] = &[(0, "ident"), (1, "digits")];

    fn variant_name(&self) -> &'static str {
        match self {
            BuildFragment::Ident(_) => Self::ALL_VARIANTS[0].1,
            BuildFragment::Digits(_) => Self::ALL_VARIANTS[1].1,
        }
    }
}
impl StrictUnion for BuildFragment {}
impl StrictEncode for BuildFragment {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_union::<Self>(|d| {
            let w = d
                .define_newtype::<Ident>(fname!("ident"))
                .define_newtype::<u128>(fname!("digits"))
                .complete();
            Ok(match self {
                BuildFragment::Ident(ident) => w.write_newtype(fname!("ident"), ident)?,
                BuildFragment::Digits(ident) => w.write_newtype(fname!("digits"), ident)?,
            }
            .complete())
        })
    }
}
impl StrictDecode for BuildFragment {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_union(|fname, r| {
            Ok(match fname.as_str() {
                "ident" => r.read_tuple(|t| t.read_field().map(Self::Ident))?,
                "digits" => r.read_tuple(|t| t.read_field().map(Self::Digits))?,
                _ => unreachable!(),
            })
        })
    }
}

macro_rules! impl_strict_ref {
    ($ty:ty, $inner:ty) => {
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
                        .define_newtype::<Ty<$inner>>(fname!("inline"))
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

impl_strict_ref!(LibRef, InlineRef);
impl_strict_ref!(InlineRef, InlineRef1);
impl_strict_ref!(InlineRef1, InlineRef2);
impl_strict_ref!(InlineRef2, KeyTy);

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
