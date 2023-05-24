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
use std::ops::Index;

use amplify::confinement;
use amplify::confinement::{MediumOrdSet, SmallOrdSet};
use encoding::{StrictDeserialize, StrictSerialize, STRICT_TYPES_LIB};

use crate::typesys::{SymTy, TypeFqn, TypeSymbol};
use crate::{Dependency, SemId, TypeSystem};

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct SymbolSystem {
    libs: SmallOrdSet<Dependency>,
    symbols: MediumOrdSet<TypeSymbol>,
}

impl StrictSerialize for SymbolSystem {}
impl StrictDeserialize for SymbolSystem {}

impl SymbolSystem {
    pub(crate) fn with(
        libs: impl IntoIterator<Item = Dependency>,
    ) -> Result<Self, confinement::Error> {
        Ok(Self {
            libs: SmallOrdSet::try_from_iter(libs)?,
            symbols: empty!(),
        })
    }

    pub(crate) fn update_unchecked(
        &mut self,
        sem_id: SemId,
        orig: Option<TypeFqn>,
    ) -> Result<(), confinement::Error> {
        self.symbols.push(TypeSymbol {
            id: sem_id,
            fqn: orig,
        })
    }

    pub fn get(&self, name: &'static str) -> Option<&SemId> {
        let needle = TypeFqn::from(name);
        self.symbols.iter().find(|fqid| fqid.fqn.as_ref() == Some(&needle)).map(|fqid| &fqid.id)
    }
}

impl Index<&'static str> for SymbolSystem {
    type Output = SemId;

    fn index(&self, index: &'static str) -> &Self::Output {
        self.get(index).unwrap_or_else(|| panic!("type {index} is absent in the type system"))
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct SymbolicTypes {
    symbols: SymbolSystem,
    types: TypeSystem,
}

impl StrictSerialize for SymbolicTypes {}
impl StrictDeserialize for SymbolicTypes {}

impl SymbolicTypes {
    pub(crate) fn with(
        libs: impl IntoIterator<Item = Dependency>,
        types: BTreeMap<SemId, SymTy>,
    ) -> Result<Self, confinement::Error> {
        let mut sys = TypeSystem::new();
        let mut sym = SymbolSystem::with(libs)?;

        for (sem_id, info) in types {
            sys.insert_unchecked(sem_id, info.ty)?;
            sym.update_unchecked(sem_id, info.orig)?;
        }

        Ok(Self {
            symbols: sym,
            types: sys,
        })
    }

    pub fn new(types: TypeSystem, symbols: SymbolSystem) -> Self { Self { symbols, types } }

    pub fn get(&self, name: &'static str) -> Option<&SemId> { self.symbols.get(name) }

    pub fn into_type_system(self) -> TypeSystem { self.types }
}

impl Index<&'static str> for SymbolicTypes {
    type Output = SemId;

    fn index(&self, index: &'static str) -> &Self::Output { &self.symbols[index] }
}
