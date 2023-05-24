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

use amplify::confinement;
use amplify::confinement::{MediumOrdMap, SmallOrdSet, TinyOrdSet};
use encoding::{StrictDeserialize, StrictSerialize, STRICT_TYPES_LIB};

use crate::typesys::{SymTy, TypeFqn};
use crate::{Dependency, SemId, TypeSystem};

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct SymbolSystem {
    libs: SmallOrdSet<Dependency>,
    symbols: MediumOrdMap<SemId, TinyOrdSet<TypeFqn>>,
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
        symbols: TinyOrdSet<TypeFqn>,
    ) -> Result<(), confinement::Error> {
        let mut sym = self.symbols.remove(&sem_id).ok().flatten().unwrap_or_default();
        sym.extend(symbols)?;
        self.symbols.insert(sem_id, sym).map(|_| ())
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
            sym.update_unchecked(sem_id, info.symbols)?;
        }

        Ok(Self {
            symbols: sym,
            types: sys,
        })
    }

    pub fn new(types: TypeSystem, symbols: SymbolSystem) -> Self { Self { symbols, types } }

    pub fn into_type_system(self) -> TypeSystem { self.types }
}
