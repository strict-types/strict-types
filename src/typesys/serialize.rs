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

use std::collections::BTreeSet;
use std::io;

use amplify::num::u24;

use crate::{
    Decode, DecodeError, Deserialize, EmbeddedRef, Encode, SemId, Serialize, StenWrite, Ty,
    TypeSystem,
};

impl Serialize for TypeSystem {}
impl Deserialize for TypeSystem {}

impl Encode for TypeSystem {
    fn encode(&self, writer: impl StenWrite) -> Result<(), io::Error> {
        self.count_types().encode(writer)?;
        for ty in self.values() {
            ty.encode(writer)?;
        }
        Ok(())
    }
}

impl Decode for TypeSystem {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        let count = u24::decode(reader)?;
        let mut lib: BTreeSet<Ty<EmbeddedRef>> = empty!();
        let mut prev: Option<SemId> = None;
        for _ in 0..count.into_usize() {
            let ty = Ty::decode(reader)?;
            if matches!(prev, Some(id) if id > ty.id()) {
                return Err(DecodeError::WrongTypeOrdering(ty.id()));
            }
            let id = ty.id();
            prev = Some(id);
            if !lib.insert(ty) {
                return Err(DecodeError::RepeatedType(id));
            }
        }
        TypeSystem::try_from_iter(lib).map_err(DecodeError::from)
    }
}

impl Encode for EmbeddedRef {
    fn encode(&self, writer: impl StenWrite) -> Result<(), io::Error> {
        match self {
            EmbeddedRef::SemId(id) => {
                0u8.encode(writer)?;
                id.encode(writer)
            }
            EmbeddedRef::Inline(ty) => {
                1u8.encode(writer)?;
                ty.encode(writer)
            }
        }
    }
}

impl Decode for EmbeddedRef {
    fn decode(reader: &mut impl io::Read) -> Result<Self, DecodeError> {
        match u8::decode(reader)? {
            0u8 => Ok(EmbeddedRef::SemId(Decode::decode(reader)?)),
            1u8 => Decode::decode(reader).map(EmbeddedRef::Inline),
            wrong => Err(DecodeError::WrongRef(wrong)),
        }
    }
}
