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

use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use amplify::confinement::Confined;
use amplify::num::u24;
use encoding::constants::UNIT;
use encoding::Ident;
use vesper::{AttrVal, Attribute, Expression, Predicate, TExpr};

use crate::ast::ItemCase;
use crate::typesys::{NestedCase, TypeInfo, TypeTree};
use crate::{Cls, Ty};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TypeLayout {
    items: Vec<TypeInfo>,
}

impl From<TypeTree<'_>> for TypeLayout {
    fn from(tree: TypeTree) -> Self {
        let mut layout = TypeLayout::new();
        layout.items.extend(&tree);
        layout
    }
}

impl<'a> From<&'a TypeTree<'_>> for TypeLayout {
    fn from(tree: &'a TypeTree) -> Self {
        let mut layout = TypeLayout::new();
        layout.items.extend(tree);
        layout
    }
}

impl Display for TypeLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_vesper().display(), f)
    }
}

impl TypeLayout {
    fn new() -> Self { Self { items: vec![] } }

    pub fn to_vesper(&self) -> TypeVesper {
        let mut root = None;
        let mut path: Vec<usize> = vec![];
        for item in &self.items {
            let expr = item.to_vesper();
            let depth = item.depth;

            if path.is_empty() && depth == 0 {
                debug_assert_eq!(root, None);
                root = Some(expr);
                continue;
            }

            debug_assert!(depth > 0);
            if path.len() < depth - 1 {
                panic!("invalid type layout with skipped levels")
            }
            // if the stack top is the same depth or deeper:
            // - remove everything down from the depth
            // - take the remaining top and add the item as a new child
            // - create new item and push it to stack
            else if path.len() >= depth {
                let _ = path.split_off(depth - 1);
            }
            // if the stack top is one level up
            // - create new item and add it as a child to the stack top item
            // - push the newly created item to stack
            let mut head = root.as_mut().expect("already set");
            for el in &path {
                head = head.content.get_mut(*el).expect("algorithm inconsistency");
            }
            path.push(head.content.len());
            head.content
                .push(Box::new(expr))
                .expect("invalid type layout containing too much items");
        }
        root.expect("invalid type layout with zero items")
    }
}

impl TypeInfo {
    pub fn to_vesper(&self) -> TypeVesper {
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
        let fqn = fqn.as_ref().map(|f| f.to_string());
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

pub type TypeVesper = TExpr<Pred>;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(lowercase)]
pub enum Pred {
    /// Type alias
    As,
    Tuple,
    Rec,
    Enum,
    Union,
    // Composites
    Char,
    Str,
    Ascii,
    Bytes,
    // Collections:
    Array,
    List,
    Set,
    Map,
}

impl Predicate for Pred {
    type Attr = Attr;
}

impl From<Cls> for Pred {
    fn from(cls: Cls) -> Self {
        match cls {
            Cls::Primitive => Pred::As,
            Cls::Unicode => Pred::Str,
            Cls::AsciiStr => Pred::Ascii,
            Cls::Enum => Pred::Enum,
            Cls::Union => Pred::Union,
            Cls::Struct => Pred::Rec,
            Cls::Tuple => Pred::Tuple,
            Cls::Array => Pred::Array,
            Cls::List => Pred::List,
            Cls::Set => Pred::Set,
            Cls::Map => Pred::Map,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
pub enum AttrExpr {
    Tag(u8),
    EnumVariant(u8),
    Len(u16),
    LenRange(LenRange),
}

impl Expression for AttrExpr {}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Attr {
    TypeName(Ident),
    Wrapped(Option<Ident>),
    Option,
    Tag(u8),
    AsciiEnum(Ident),
    EnumVariant(u8, Ident),
    Len(u16),
    LenRange(LenRange),
}

impl Attribute for Attr {
    type Expression = AttrExpr;

    fn name(&self) -> Option<Ident> {
        match self {
            Attr::TypeName(_) => None,
            Attr::Wrapped(name) if name.is_some() => ident!("aka"),
            Attr::Wrapped(_) => None,
            Attr::Option => None,
            Attr::Tag(_) => ident!("tag"),
            Attr::Len(_) => ident!("len"),
            Attr::LenRange(_) => ident!("len"),
            Attr::AsciiEnum(_) => ident!("charset"),
            Attr::EnumVariant(_, name) => Some(name.clone()),
        }
    }

    fn value(&self) -> AttrVal<Self::Expression> {
        match self {
            Attr::TypeName(tn) => AttrVal::Ident(tn.clone()),
            Attr::Wrapped(name) => AttrVal::Ident(name.clone().unwrap_or(ident!("wrapped"))),
            Attr::Option => AttrVal::Ident(ident!("option")),
            Attr::Tag(tag) => AttrVal::Expr(AttrExpr::Tag(*tag)),
            Attr::Len(len) => AttrVal::Expr(AttrExpr::Len(*len)),
            Attr::LenRange(range) => AttrVal::Expr(AttrExpr::LenRange(range.clone())),
            Attr::AsciiEnum(name) => AttrVal::Ident(name.clone()),
            Attr::EnumVariant(pos, _) => AttrVal::Expr(AttrExpr::Tag(*pos)),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct LenRange(Range<u64>);

impl Display for LenRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match (self.0.start, self.0.end) {
            (min, max) if max == u8::MAX as u64 => write!(f, "{min}..MAX8"),
            (min, max) if max == u16::MAX as u64 => write!(f, "{min}..MAX16"),
            (min, max) if max == u24::MAX.into_u64() => write!(f, "{min}..MAX24"),
            (min, max) if max == u32::MAX as u64 => write!(f, "{min}..MAX32"),
            // TODO: Add more numbers
            (min, u64::MAX) => write!(f, "{min}..MAX64"),
            (min, max) => write!(f, "{min}..{max}"),
        }
    }
}
