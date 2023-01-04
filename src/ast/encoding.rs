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
use std::ops::Deref;

use amplify::confinement::TinyOrdMap;
use strict_encoding::{
    DecodeError, DefineTuple, DefineUnion, FieldName, Primitive, ReadStruct, ReadTuple, ReadUnion,
    Sizing, StrictDecode, StrictDumb, StrictEncode, StrictEnum, StrictProduct, StrictStruct,
    StrictSum, StrictTuple, StrictType, StrictUnion, TypedRead, TypedWrite, Variant, WriteStruct,
    WriteTuple, WriteUnion, STEN_LIB,
};

use crate::ast::ty::{UnionVariants, UnnamedFields};
use crate::ast::{EnumVariants, Field, NamedFields, Step};
use crate::{Cls, KeyTy, SemId, Ty, TypeRef};

strict_newtype!(SemId, STEN_LIB);
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

impl StrictDumb for Step {
    fn strict_dumb() -> Self { Step::Index }
}
impl StrictType for Step {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictSum for Step {
    const ALL_VARIANTS: &'static [(u8, &'static str)] = &[
        (0, "namedField"),
        (1, "unnamedField"),
        (2, "index"),
        (3, "list"),
        (4, "set"),
        (5, "map"),
    ];

    fn variant_name(&self) -> &'static str {
        match self {
            Step::NamedField(_) => Self::ALL_VARIANTS[0].1,
            Step::UnnamedField(_) => Self::ALL_VARIANTS[1].1,
            Step::Index => Self::ALL_VARIANTS[2].1,
            Step::List => Self::ALL_VARIANTS[3].1,
            Step::Set => Self::ALL_VARIANTS[4].1,
            Step::Map => Self::ALL_VARIANTS[5].1,
        }
    }
}
impl StrictUnion for Step {}
impl StrictEncode for Step {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_union::<Self>(|udef| {
            let uwriter = udef
                .define_newtype::<FieldName>(fname!("namedField"))
                .define_newtype::<u8>(fname!("unnamedField"))
                .define_unit(fname!("index"))
                .define_unit(fname!("list"))
                .define_unit(fname!("set"))
                .define_unit(fname!("map"))
                .complete();

            Ok(match self {
                Step::NamedField(name) => uwriter.write_newtype(fname!("namedField"), name),
                Step::UnnamedField(ord) => uwriter.write_newtype(fname!("unnamedField"), ord),
                Step::Index => uwriter.write_unit(fname!("index")),
                Step::List => uwriter.write_unit(fname!("list")),
                Step::Set => uwriter.write_unit(fname!("set")),
                Step::Map => uwriter.write_unit(fname!("map")),
            }?
            .complete())
        })
    }
}
impl StrictDecode for Step {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_union(|field_name, r| match field_name.as_str() {
            s if s == Self::ALL_VARIANTS[0].1 => r.read_newtype::<Self, FieldName>(),
            s if s == Self::ALL_VARIANTS[1].1 => r.read_newtype::<Self, u8>(),
            s if s == Self::ALL_VARIANTS[2].1 => Ok(Self::Index),
            s if s == Self::ALL_VARIANTS[3].1 => Ok(Self::List),
            s if s == Self::ALL_VARIANTS[4].1 => Ok(Self::Set),
            s if s == Self::ALL_VARIANTS[5].1 => Ok(Self::Map),
            _ => unreachable!("a new enum variant is added without covering its decoding"),
        })
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

impl<Ref: TypeRef> StrictDumb for Field<Ref> {
    fn strict_dumb() -> Self {
        Field {
            name: fname!("dumb"),
            ty: Ref::strict_dumb(),
        }
    }
}
impl<Ref: TypeRef> StrictType for Field<Ref> {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl<Ref: TypeRef> StrictProduct for Field<Ref> {}
impl<Ref: TypeRef> StrictStruct for Field<Ref> {
    const ALL_FIELDS: &'static [&'static str] = &["name", "ty"];
}
impl<Ref: TypeRef> StrictEncode for Field<Ref> {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_struct::<Self>(|w| {
            Ok(w.write_field(fname!("name"), &self.name)?
                .write_field(fname!("ty"), &self.ty)?
                .complete())
        })
    }
}
impl<Ref: TypeRef> StrictDecode for Field<Ref> {
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
impl<Ref: TypeRef> StrictType for NamedFields<Ref> {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl<Ref: TypeRef> StrictProduct for NamedFields<Ref> {}
impl<Ref: TypeRef> StrictTuple for NamedFields<Ref> {
    const FIELD_COUNT: u8 = 1;
}
impl<Ref: TypeRef> StrictEncode for NamedFields<Ref> {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_newtype::<Self>(self.deref())
    }
}
impl<Ref: TypeRef> StrictDecode for NamedFields<Ref> {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_newtype()
    }
}

impl<Ref: TypeRef> StrictDumb for UnnamedFields<Ref> {
    fn strict_dumb() -> Self { fields!(Ref::strict_dumb()) }
}
impl<Ref: TypeRef> StrictType for UnnamedFields<Ref> {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl<Ref: TypeRef> StrictProduct for UnnamedFields<Ref> {}
impl<Ref: TypeRef> StrictTuple for UnnamedFields<Ref> {
    const FIELD_COUNT: u8 = 1;
}
impl<Ref: TypeRef> StrictEncode for UnnamedFields<Ref> {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_newtype::<Self>(self.deref())
    }
}
impl<Ref: TypeRef> StrictDecode for UnnamedFields<Ref> {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_newtype()
    }
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
impl StrictType for EnumVariants {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictProduct for EnumVariants {}
impl StrictTuple for EnumVariants {
    const FIELD_COUNT: u8 = 1;
}
impl StrictEncode for EnumVariants {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_newtype::<Self>(self.deref())
    }
}
impl StrictDecode for EnumVariants {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_newtype()
    }
}

impl StrictDumb for Cls {
    fn strict_dumb() -> Self { Cls::Primitive }
}
impl StrictType for Cls {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictSum for Cls {
    const ALL_VARIANTS: &'static [(u8, &'static str)] = &[
        (0, "primitive"),
        (1, "unicode"),
        (2, "ascii"),
        (3, "enum"),
        (4, "union"),
        (5, "struct"),
        (6, "tuple"),
        (7, "array"),
        (8, "list"),
        (9, "set"),
        (10, "map"),
    ];

    fn variant_name(&self) -> &'static str { Self::ALL_VARIANTS[*self as u8 as usize].1 }
}
impl StrictEnum for Cls {}
impl StrictEncode for Cls {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> { writer.write_enum(*self) }
}
impl StrictDecode for Cls {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_enum()
    }
}

impl<Ref: TypeRef> StrictDumb for Ty<Ref> {
    fn strict_dumb() -> Self { Ty::UnicodeChar }
}
impl<Ref: TypeRef> StrictType for Ty<Ref> {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl<Ref: TypeRef> StrictSum for Ty<Ref> {
    const ALL_VARIANTS: &'static [(u8, &'static str)] = &[
        (0, "primitive"),
        (1, "unicode"),
        (3, "enum"),
        (4, "union"),
        (5, "struct"),
        (6, "tuple"),
        (7, "array"),
        (8, "list"),
        (9, "set"),
        (10, "map"),
    ];
    fn variant_name(&self) -> &'static str { self.cls().variant_name() }
}
impl<Ref: TypeRef> StrictUnion for Ty<Ref> {}
impl<Ref: TypeRef> StrictEncode for Ty<Ref> {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_union::<Self>(|u| {
            let u = u
                .define_newtype::<u8>(fname!("primitive"))
                .define_unit(fname!("unicode"))
                .define_newtype::<EnumVariants>(fname!("enum"))
                .define_newtype::<UnionVariants<Ref>>(fname!("union"))
                .define_newtype::<NamedFields<Ref>>(fname!("struct"))
                .define_newtype::<UnnamedFields<Ref>>(fname!("tuple"))
                .define_tuple(fname!("array"), |d| {
                    d.define_field::<Ref>().define_field::<u16>().complete()
                })
                .define_tuple(fname!("list"), |d| {
                    d.define_field::<Ref>().define_field::<Sizing>().complete()
                })
                .define_tuple(fname!("set"), |d| {
                    d.define_field::<Ref>().define_field::<Sizing>().complete()
                })
                .define_tuple(fname!("map"), |d| {
                    d.define_field::<KeyTy>()
                        .define_field::<Ref>()
                        .define_field::<Sizing>()
                        .complete()
                })
                .complete();

            let u = match self {
                Ty::Primitive(prim) => u.write_newtype(fname!("primitive"), &prim.into_code())?,
                Ty::UnicodeChar => u.write_unit(fname!("unicode"))?,
                Ty::Enum(vars) => u.write_newtype(fname!("enum"), vars)?,
                Ty::Union(fields) => u.write_newtype(fname!("union"), fields)?,
                Ty::Struct(fields) => u.write_newtype(fname!("struct"), fields)?,
                Ty::Tuple(fields) => u.write_newtype(fname!("tuple"), fields)?,
                Ty::Array(ty, len) => u.write_tuple(fname!("array"), |w| {
                    Ok(w.write_field(ty)?.write_field(len)?.complete())
                })?,
                Ty::List(ty, sizing) => u.write_tuple(fname!("list"), |w| {
                    Ok(w.write_field(ty)?.write_field(sizing)?.complete())
                })?,
                Ty::Set(ty, sizing) => u.write_tuple(fname!("set"), |w| {
                    Ok(w.write_field(ty)?.write_field(sizing)?.complete())
                })?,
                Ty::Map(key, ty, sizing) => u.write_tuple(fname!("map"), |w| {
                    Ok(w.write_field(key)?.write_field(ty)?.write_field(sizing)?.complete())
                })?,
            };

            Ok(u.complete())
        })
    }
}
impl<Ref: TypeRef> StrictDecode for Ty<Ref> {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_union(|field_name, r| {
            let cls = Cls::from_variant_name(&field_name)
                .expect("inconsistency between Cls and Ty variants");
            match cls {
                Cls::Primitive => r.read_newtype::<Ty<Ref>, Primitive>(),
                Cls::Unicode => Ok(Ty::UnicodeChar),
                Cls::AsciiStr => unreachable!("ASCII string is only used by KeyTy"),
                Cls::Enum => r.read_newtype::<Ty<Ref>, EnumVariants>(),
                Cls::Union => r.read_newtype::<Ty<Ref>, UnionVariants<Ref>>(),
                Cls::Struct => r.read_newtype::<Ty<Ref>, NamedFields<Ref>>(),
                Cls::Tuple => r.read_newtype::<Ty<Ref>, UnnamedFields<Ref>>(),
                Cls::Array => r.read_tuple(|tr| {
                    let ty = tr.read_field()?;
                    let len = tr.read_field()?;
                    Ok(Ty::Array(ty, len))
                }),
                Cls::List => r.read_tuple(|tr| {
                    let ty = tr.read_field()?;
                    let sizing = tr.read_field()?;
                    Ok(Ty::List(ty, sizing))
                }),
                Cls::Set => r.read_tuple(|tr| {
                    let ty = tr.read_field()?;
                    let sizing = tr.read_field()?;
                    Ok(Ty::Set(ty, sizing))
                }),
                Cls::Map => r.read_tuple(|tr| {
                    let key = tr.read_field()?;
                    let ty = tr.read_field()?;
                    let sizing = tr.read_field()?;
                    Ok(Ty::Map(key, ty, sizing))
                }),
            }
        })
    }
}

impl StrictDumb for KeyTy {
    fn strict_dumb() -> Self { KeyTy::Array(1) }
}
impl StrictType for KeyTy {
    const STRICT_LIB_NAME: &'static str = STEN_LIB;
}
impl StrictSum for KeyTy {
    const ALL_VARIANTS: &'static [(u8, &'static str)] =
        &[(0, "primitive"), (1, "unicode"), (2, "ascii"), (3, "enum"), (7, "array"), (8, "bytes")];

    fn variant_name(&self) -> &'static str {
        match self {
            KeyTy::Primitive(_) => Self::ALL_VARIANTS[0].1,
            KeyTy::Enum(_) => Self::ALL_VARIANTS[3].1,
            KeyTy::Array(_) => Self::ALL_VARIANTS[4].1,
            KeyTy::UnicodeStr(_) => Self::ALL_VARIANTS[1].1,
            KeyTy::AsciiStr(_) => Self::ALL_VARIANTS[2].1,
            KeyTy::Bytes(_) => Self::ALL_VARIANTS[5].1,
        }
    }
}
impl StrictUnion for KeyTy {}
impl StrictEncode for KeyTy {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_union::<Self>(|u| {
            let u = u
                .define_newtype::<u8>(fname!("primitive"))
                .define_newtype::<EnumVariants>(fname!("enum"))
                .define_newtype::<u16>(fname!("array"))
                .define_newtype::<Sizing>(fname!("unicode"))
                .define_newtype::<Sizing>(fname!("ascii"))
                .define_newtype::<Sizing>(fname!("bytes"))
                .complete();

            let u = match self {
                KeyTy::Primitive(prim) => {
                    u.write_newtype(fname!("primitive"), &prim.into_code())?
                }
                KeyTy::Enum(vars) => u.write_newtype(fname!("enum"), vars)?,
                KeyTy::Array(len) => u.write_newtype(fname!("array"), len)?,
                KeyTy::UnicodeStr(sizing) => u.write_newtype(fname!("unicode"), sizing)?,
                KeyTy::AsciiStr(sizing) => u.write_newtype(fname!("ascii"), sizing)?,
                KeyTy::Bytes(sizing) => u.write_newtype(fname!("bytes"), sizing)?,
            };

            Ok(u.complete())
        })
    }
}
impl StrictDecode for KeyTy {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_union(|mut field_name, r| {
            if field_name == fname!("bytes") {
                field_name = fname!("list")
            }
            let cls = Cls::from_variant_name(&field_name)
                .expect("inconsistency between Cls and KeyTy variants");
            match cls {
                Cls::Primitive => r.read_newtype::<KeyTy, Primitive>(),
                Cls::Enum => r.read_newtype::<KeyTy, EnumVariants>(),
                Cls::Array => r.read_newtype::<KeyTy, u16>(),
                Cls::List => r.read_tuple(|r| Ok(KeyTy::Bytes(r.read_field()?))),
                Cls::Unicode => r.read_tuple(|r| Ok(KeyTy::UnicodeStr(r.read_field()?))),
                Cls::AsciiStr => r.read_tuple(|r| Ok(KeyTy::AsciiStr(r.read_field()?))),
                _ => unreachable!("inconsistency between Cls and KeyTy variants"),
            }
        })
    }
}
