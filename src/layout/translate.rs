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

use amplify::confinement::Confined;
use encoding::constants::UNIT;
use encoding::Ident;

use super::vesper::{Attr, Pred, TypeVesper};
use crate::ast::ItemCase;
use crate::typesys::{NestedCase, TypeInfo};
use crate::Ty;

impl TypeInfo {
    pub(super) fn to_vesper(&self) -> TypeVesper {
        let TypeInfo {
            ty,
            fqn,
            item,
            nested,
            ..
        } = self;

        let mut attributes = vec![];
        let mut comment = None;
        let name = fqn.as_ref().map(|f| f.name.clone()).unwrap_or_else(|| tn!("_"));
        let fqn = fqn.as_ref().map(|f| f.name.to_string());
        let subject = match item {
            Some(ItemCase::UnnamedField(pos)) => {
                if name.as_str() == "_" {
                    comment = fqn;
                    Ident::from_uint(*pos)
                } else {
                    name.into_ident()
                }
            }
            Some(ItemCase::NamedField(_, ref fname)) => {
                comment = fqn;
                fname.to_ident()
            }
            Some(ItemCase::UnionVariant(_, ref vname)) => {
                comment = fqn;
                vname.to_ident()
            }
            Some(ItemCase::MapKey) if fqn.is_some() => {
                comment = Some(s!("map key"));
                name.into_ident()
            }
            Some(ItemCase::MapKey) => tn!("from"),
            Some(ItemCase::MapValue) if fqn.is_some() => {
                comment = Some(s!("mapped onto"));
                name.into_ident()
            }
            Some(ItemCase::MapValue) => tn!("to"),
            _ => name.into_ident(),
        };
        let mut predicate = ty.cls().into();
        match ty {
            Ty::Primitive(prim) if *prim == UNIT => {
                attributes.push(Attr::TypeName(tn!("Unit")));
            }
            Ty::Primitive(prim) => {
                attributes.push(Attr::TypeName(tn!("{}", prim)));
            }
            Ty::Array(_, len) => attributes.push(Attr::Len(*len)),
            _ => {}
        }
        if ty.is_char_enum() {
            predicate = Pred::Char;
        } else if ty.is_byte_array() {
            predicate = Pred::Bytes;
        }
        for case in nested {
            match case {
                NestedCase::AsciiStr(fqn) => {
                    predicate = Pred::Ascii;
                    if let Some(fqn) = fqn {
                        attributes.push(Attr::AsciiEnum(fqn.name.to_ident()));
                    }
                }
                NestedCase::ByteStr => {
                    predicate = Pred::Bytes;
                }
                NestedCase::UniStr => {
                    predicate = Pred::Str;
                }
                NestedCase::NewType(fqn) => {
                    attributes.push(Attr::Wrapped(fqn.as_ref().map(|f| f.name.to_ident())));
                }
                NestedCase::Option => {
                    attributes.push(Attr::Option);
                }
            }
        }

        match ty {
            Ty::Enum(variants) => {
                for var in variants {
                    attributes.push(Attr::EnumVariant(var.tag, var.name.to_ident()))
                }
            }
            _ => {}
        }
        if let Some(ItemCase::UnionVariant(ref pos, _)) = item {
            attributes.push(Attr::Tag(*pos));
        }

        TypeVesper {
            subject,
            predicate,
            attributes: Confined::from_collection_unsafe(attributes),
            content: none!(),
            comment,
        }
    }
}
