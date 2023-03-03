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

//! Checks strict values against provied strict type specification.

use std::collections::BTreeSet;

use amplify::ascii::{AsAsciiStrError, AsciiString};
use amplify::Wrapper;
use encoding::constants::UNIT;
use encoding::{FieldName, InvalidIdent, Primitive, Sizing};
use indexmap::IndexMap;

use super::StrictVal;
use crate::ast::EnumVariants;
use crate::typesys::{TypeFqn, TypeInfo};
use crate::value::{EnumTag, StrictNum};
use crate::{SemId, Ty, TypeRef, TypeSystem};

#[derive(Clone, Eq, PartialEq, Hash, Debug, From, Display)]
#[display(inner)]
pub enum TypeSpec {
    #[from]
    SemId(SemId),

    #[from]
    #[from(&'static str)]
    // TODO: Add optional checkword suffix
    Fqn(TypeFqn /* , Option<CheckWords> */),
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display("{val}@{spec}")]
pub struct TypedVal {
    spec: TypeSpec,
    val: StrictVal,
}

impl TypedVal {
    pub fn as_spec(&self) -> &TypeSpec { &self.spec }
    pub fn as_val(&self) -> &StrictVal { &self.val }
}

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum Error {
    /// unknown type `{0}`.
    TypeAbsent(TypeSpec),

    /// collection `{0}` has size {1} which is out of type required bounds {2}.
    OutOfBounds(TypeSpec, usize, Sizing),

    /// invalid ASCII string "{0}".
    #[from]
    InvalidAsciiString(AsAsciiStrError),

    /// repeated value {1} in set `{0}`.
    RepeatedSetValue(TypeSpec, StrictVal),

    /// repeated key {1} in map `{0}`.
    RepeatedKeyValue(TypeSpec, StrictVal),

    /// type `{spec}` requires {expected} fields, while {found} fields were given.
    FieldNumberMismatch {
        spec: TypeSpec,
        expected: usize,
        found: usize,
    },

    /// unexpected field `{0}`.
    ExtraField(FieldName),

    /// value `{value}` doesn't match type requirements `{expected}`.
    TypeMismatch {
        value: StrictVal,
        expected: Ty<SemId>,
    },

    #[display(inner)]
    #[from]
    InvalidFieldName(InvalidIdent),

    /// invalid enum tag `{0}`; allowed variants are {1}.
    EnumTagInvalid(EnumTag, EnumVariants),

    /// invalid union tag `{0}`; allowed variants are {1}.
    UnionTagInvalid(EnumTag, EnumVariants),

    /// mapping found where a structure value was expected.
    MapNotStructure,
}

trait PrimitiveValue {
    fn is_small_unsigned(&self) -> bool;
    fn is_large_unsigned(&self) -> bool;
    fn is_small_signed(&self) -> bool;
    fn is_large_signed(&self) -> bool;
}

impl PrimitiveValue for Primitive {
    fn is_small_unsigned(&self) -> bool { self.into_code() <= 16 }
    fn is_large_unsigned(&self) -> bool { self.into_code() > 16 && self.into_code() < 0x40 }
    fn is_small_signed(&self) -> bool { self.into_code() >= 0x40 && self.into_code() <= 0x4f }
    fn is_large_signed(&self) -> bool { self.into_code() > 0x4f && self.into_code() < 0x80 }
}

impl TypeSystem {
    pub fn find(&self, spec: &TypeSpec) -> Option<&TypeInfo> {
        match spec {
            TypeSpec::SemId(find_id) => {
                self.as_inner().iter().find(|(my_id, _)| *my_id == find_id).map(|(_, info)| info)
            }
            TypeSpec::Fqn(fqn) => {
                self.as_inner().values().find(|info| info.fqn.as_ref() == Some(fqn))
            }
        }
    }

    pub fn typify(&self, val: StrictVal, spec: TypeSpec) -> Result<TypedVal, Error> {
        let ty = &self.find(&spec).ok_or_else(|| Error::TypeAbsent(spec.clone()))?.ty;
        let val = match (val, ty) {
            // Primitive direct matches:
            (val @ StrictVal::Unit, Ty::Primitive(prim)) if *prim == UNIT => val,
            (val @ StrictVal::Number(StrictNum::Uint(_)), Ty::Primitive(prim))
                if prim.is_small_unsigned() =>
            {
                val
            }
            (val @ StrictVal::Number(StrictNum::BigUint(_)), Ty::Primitive(prim))
                if prim.is_large_unsigned() =>
            {
                val
            }
            (val @ StrictVal::Number(StrictNum::Int(_)), Ty::Primitive(prim))
                if prim.is_small_signed() =>
            {
                val
            }
            (val @ StrictVal::Number(StrictNum::BigInt(_)), Ty::Primitive(prim))
                if prim.is_large_signed() =>
            {
                val
            }

            // Collection bounds check:
            (StrictVal::String(s), Ty::List(_, sizing)) if !sizing.check(s.len()) => {
                return Err(Error::OutOfBounds(spec, s.len(), *sizing))
            }
            (StrictVal::List(s), Ty::List(_, sizing)) | (StrictVal::Set(s), Ty::Set(_, sizing))
                if !sizing.check(s.len()) =>
            {
                return Err(Error::OutOfBounds(spec, s.len(), *sizing))
            }
            (StrictVal::Map(s), Ty::Map(_, _, sizing)) if !sizing.check(s.len()) => {
                return Err(Error::OutOfBounds(spec, s.len(), *sizing))
            }

            // Collection items type checks:
            (val @ StrictVal::String(_), Ty::List(id, _)) if id.is_unicode_char() => val,
            (StrictVal::String(s), Ty::List(id, _)) if id.is_ascii_char() => {
                AsciiString::from_ascii(s.as_bytes()).map_err(|err| err.ascii_error())?;
                StrictVal::String(s)
            }
            (StrictVal::List(s), Ty::List(ty, _)) => {
                let mut new = Vec::with_capacity(s.len());
                for item in s {
                    let checked = self.typify(item, TypeSpec::SemId(*ty))?;
                    new.push(checked.val);
                }
                StrictVal::List(new)
            }
            (StrictVal::Set(s), Ty::Set(ty, _)) => {
                let mut new = Vec::with_capacity(s.len());
                for item in s {
                    let checked = self.typify(item, TypeSpec::SemId(*ty))?;
                    if new.contains(&checked.val) {
                        return Err(Error::RepeatedSetValue(spec, checked.val));
                    }
                    new.push(checked.val);
                }
                StrictVal::Set(new)
            }
            (StrictVal::Map(s), Ty::Map(key_ty, ty, _)) => {
                let mut new = Vec::<(StrictVal, StrictVal)>::with_capacity(s.len());
                let key_id = key_ty.to_ty().id(None);
                for (key, item) in s {
                    let checked_key = self.typify(key, TypeSpec::SemId(key_id))?;
                    let checked_val = self.typify(item, TypeSpec::SemId(*ty))?;
                    if new.iter().find(|(k, _)| k == &checked_key.val).is_some() {
                        return Err(Error::RepeatedKeyValue(spec, checked_key.val));
                    }
                    new.push((checked_key.val, checked_val.val));
                }
                StrictVal::Map(new)
            }

            // Enums:
            (StrictVal::Enum(tag), Ty::Enum(variants)) => {
                let vname = match &tag {
                    EnumTag::Name(name) => variants.tag_by_name(name).map(|_| name),
                    EnumTag::Ord(ord) => variants.name_by_tag(*ord),
                };
                match vname {
                    None => return Err(Error::EnumTagInvalid(tag, variants.clone())),
                    Some(name) => StrictVal::enumer(name.clone()),
                }
            }
            (StrictVal::Number(StrictNum::Uint(tag)), Ty::Enum(variants)) if tag < 0x100 => {
                let tag = tag as u8;
                let vname = variants.name_by_tag(tag);
                match vname {
                    None => return Err(Error::EnumTagInvalid(tag.into(), variants.clone())),
                    Some(name) => StrictVal::enumer(name.clone()),
                }
            }
            (StrictVal::Union(tag, val), Ty::Union(vars_req)) => {
                let Some(id) = (match &tag {
                    EnumTag::Name(name) => vars_req.ty_by_name(name),
                    EnumTag::Ord(ord) => vars_req.ty_by_ord(*ord),
                }) else {
                    return Err(Error::UnionTagInvalid(
                        tag,
                        EnumVariants::try_from(vars_req.keys().cloned().collect::<BTreeSet<_>>())
                            .expect("same collection size"),
                    ))
                };
                let checked = self.typify(*val, *id)?;
                StrictVal::Union(tag, Box::new(checked.val))
            }

            // Field count check:
            (StrictVal::Tuple(fields), Ty::Tuple(fields_req))
                if fields.len() != fields_req.len() =>
            {
                return Err(Error::FieldNumberMismatch {
                    spec,
                    expected: fields_req.len(),
                    found: fields.len(),
                })
            }
            (StrictVal::Struct(fields), Ty::Struct(fields_req))
                if fields.len() != fields_req.len() =>
            {
                return Err(Error::FieldNumberMismatch {
                    spec,
                    expected: fields_req.len(),
                    found: fields.len(),
                })
            }

            // Check specific field types:
            (StrictVal::Tuple(s) | StrictVal::List(s), Ty::Tuple(fields_req)) => {
                let mut new = Vec::with_capacity(s.len());
                for (item, id) in s.into_iter().zip(fields_req) {
                    let checked = self.typify(item, TypeSpec::SemId(*id))?;
                    new.push(checked.val);
                }
                StrictVal::Tuple(new)
            }
            (StrictVal::Struct(s), Ty::Struct(fields_req)) => {
                let mut new = IndexMap::with_capacity(s.len());
                for (fname, item) in s.into_iter() {
                    let Some(field) = fields_req.ty_by_name(&fname) else {
                        return Err(Error::ExtraField(fname));
                    };
                    let checked = self.typify(item, TypeSpec::SemId(*field))?;
                    new.insert(fname, checked.val);
                }
                StrictVal::Struct(new)
            }
            (StrictVal::Map(s), Ty::Struct(fields_req)) => {
                let mut new = IndexMap::with_capacity(s.len());
                for (fname, item) in s.into_iter() {
                    let StrictVal::String(fname) = fname else {
                        return Err(Error::MapNotStructure)
                    };
                    let fname = FieldName::try_from(fname)?;
                    let Some(field) = fields_req.ty_by_name(&fname) else {
                        return Err(Error::ExtraField(fname));
                    };
                    let checked = self.typify(item, TypeSpec::SemId(*field))?;
                    new.insert(fname, checked.val);
                }
                StrictVal::Struct(new)
            }

            (val, ty) => {
                return Err(Error::TypeMismatch {
                    value: val,
                    expected: ty.clone(),
                })
            }
        };
        Ok(TypedVal { spec, val })
    }
}
