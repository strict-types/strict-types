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

use std::fmt::{self, Display, Formatter};
use std::io;

use amplify::num::{i1024, u1024};
use encoding::constants::UNIT;
use encoding::Primitive;
use indexmap::IndexMap;

use crate::typesys::TypeFqn;
use crate::{SemId, Ty, TypeSystem};

#[derive(Clone, Eq, PartialEq, Hash, Debug, From, Display)]
#[display(inner)]
pub enum TypeSpec {
    #[from]
    SemId(SemId),
    #[from]
    Fqn(TypeFqn),
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display("{val}@{name}")]
pub struct StrictObj {
    name: TypeSpec,
    val: StrictVal,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display(inner)]
#[non_exhaustive]
pub enum StrictNum {
    Uint(u128),
    BigUint(u1024),
    Int(i128),
    BitInt(i1024),
    // float
    // non-zero
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(inner)]
pub enum EnumTag {
    Name(String),
    Ord(u8),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum StrictVal {
    Unit,
    Number(StrictNum),
    String(String),
    Tuple(Vec<StrictVal>),
    Struct(IndexMap<String, StrictVal>),
    Enum(EnumTag),
    Union(EnumTag, Box<StrictVal>),
    List(Vec<StrictVal>),
    Table(IndexMap<String, StrictVal>),
}

impl Display for StrictVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { todo!() }
}

pub trait TypeResolver {
    fn ty_by_id(&self, id: SemId) -> Option<Ty<SemId>>;
}

impl TypeSystem {
    pub fn typify(&self, id: SemId, obj: StrictObj, e: impl io::Write) -> Result<(), Error> {
        todo!()
    }
    pub fn reify(&self, id: SemId, d: impl io::Read) -> Result<StrictObj, Error> { todo!() }
}

trait PrimitiveValue {
    fn is_small_unsigned(&self) -> bool;
}

impl PrimitiveValue for Primitive {
    fn is_small_unsigned(&self) -> bool { self.into_code() <= 16 }
}

impl StrictVal {
    fn encode(
        &self,
        ty: Ty<SemId>,
        resolver: &impl TypeResolver,
        e: impl io::Write,
    ) -> io::Result<()> {
        match (self, ty) {
            (StrictVal::Unit, Ty::Primitive(prim)) if prim == UNIT => {}
            (StrictVal::Number(StrictNum::Uint(val)), Ty::Primitive(prim))
                if prim.is_small_unsigned() => {}
            (StrictVal::Table(map), Ty::Struct(fields)) if map.len() == fields.len() => {}
            _ => todo!(),
        }
        Ok(())
    }

    fn decode(
        ty: Ty<SemId>,
        resolver: &impl TypeResolver,
        d: impl io::Read,
    ) -> Result<Self, Error> {
        todo!()
    }
}

pub enum Error {}
