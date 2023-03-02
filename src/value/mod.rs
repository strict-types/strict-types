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

use std::collections::HashMap;
use std::io;

use amplify::num::{i1024, u1024};
use encoding::constants::UNIT;
use encoding::TypeName;

use crate::{SemId, Ty, TypeSystem};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StyObj {
    pub name: Option<TypeName>,
    pub val: Box<StyVal<StyObj>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum StyNum {
    Uint(u128),
    BigUint(u1024),
    Int(i128),
    BitInt(i1024),
    Float(f64),
    // big float
    // non-zero
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum StyVal<T = Box<StyVal>> {
    Unit,
    Number(StyNum),
    String(String),
    List(Vec<T>),
    Table(HashMap<String, T>),
}

pub trait TypeResolver {
    fn ty_by_id(&self, id: SemId) -> Option<Ty<SemId>>;
}

trait StrictValue: Sized {
    fn encode(
        &self,
        ty: Ty<SemId>,
        resolver: &impl TypeResolver,
        e: impl io::Write,
    ) -> io::Result<()>;
    fn decode(ty: Ty<SemId>, resolver: &impl TypeResolver, d: impl io::Read)
        -> Result<Self, Error>;
}

impl TypeSystem {
    pub fn typify(
        &self,
        id: SemId,
        value: &impl StrictValue,
        e: impl io::Write,
    ) -> Result<(), Error> {
    }
    pub fn reify(&self, id: SemId, d: impl io::Read) -> Result<StyObj, Error> {}
}

impl StrictValue for StyVal {
    fn encode(
        &self,
        ty: Ty<SemId>,
        resolver: &impl TypeResolver,
        e: impl io::Write,
    ) -> io::Result<()> {
        match (self, ty) {
            (StyVal::Unit, Ty::Primitive(prim)) if prim == UNIT => {}
            (StyVal::Number(StyNum::Uint(val)), Ty::Primitive(prim))
                if prim.is_small_unsigned() => {}
            (StyVal::Table(map), Ty::Struct(fields)) if map.len() == fields.len() => {}
        }
        Ok(())
    }
}

pub enum Error {}
