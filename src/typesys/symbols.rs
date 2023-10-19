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

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::ops::Index;

use amplify::confinement::{self, MediumOrdSet, SmallOrdSet};
use encoding::{StrictDeserialize, StrictSerialize, STRICT_TYPES_LIB};

use crate::typesys::{translate, SymTy, TypeFqn, TypeSymbol, TypeSysId, TypeTree};
use crate::typify::TypeSpec;
use crate::{Dependency, SemId, Translate, Ty, TypeSystem};

#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Symbols {
    libs: SmallOrdSet<Dependency>,
    symbols: MediumOrdSet<TypeSymbol>,
}

impl StrictSerialize for Symbols {}
impl StrictDeserialize for Symbols {}

impl Symbols {
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
    ) -> Result<(), translate::Error> {
        let sym = TypeSymbol {
            id: sem_id,
            fqn: orig,
        };
        if let Some(present) = self.symbols.get(&sym) {
            return Err(translate::Error::RepeatedType {
                new: sym,
                present: present.clone(),
            });
        }
        self.symbols.push(sym)?;
        Ok(())
    }

    pub fn get(&self, spec: impl Into<TypeFqn>) -> Option<&SemId> {
        let needle = spec.into();
        self.symbols.iter().find(|fqid| fqid.fqn.as_ref() == Some(&needle)).map(|fqid| &fqid.id)
    }

    pub fn lookup(&self, sem_id: SemId) -> Option<&TypeFqn> {
        self.symbols.iter().find(|sym| sym.id == sem_id).and_then(|sym| sym.fqn.as_ref())
    }
}

impl Index<&'static str> for Symbols {
    type Output = SemId;

    fn index(&self, index: &'static str) -> &Self::Output {
        self.get(index).unwrap_or_else(|| panic!("type {index} is absent in the type system"))
    }
}

impl Display for Symbols {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for dep in &self.libs {
            writeln!(f, "{dep} as {}", dep.name)?;
        }
        writeln!(f, "{{--")?;
        for sym in &self.symbols {
            if let Some(fqn) = &sym.fqn {
                writeln!(f, "  {} => {}", fqn, sym.id)?;
            }
        }
        writeln!(f, "--}}")
    }
}

#[derive(Getters, Clone, Eq, PartialEq, Debug)]
#[getter(prefix = "as_")]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct SymbolicSys {
    pub(super) symbols: Symbols,
    types: TypeSystem,
}

impl StrictSerialize for SymbolicSys {}
impl StrictDeserialize for SymbolicSys {}

impl SymbolicSys {
    pub(crate) fn with(
        libs: impl IntoIterator<Item = Dependency>,
        types: BTreeMap<SemId, SymTy>,
    ) -> Result<Self, translate::Error> {
        let mut sys = TypeSystem::new();
        let mut sym = Symbols::with(libs)?;

        for (sem_id, info) in types {
            sys.insert_unchecked(sem_id, info.ty)?;
            sym.update_unchecked(sem_id, info.orig)?;
        }

        Ok(Self {
            symbols: sym,
            types: sys,
        })
    }

    pub fn new(types: TypeSystem, symbols: Symbols) -> Self { Self { symbols, types } }

    pub fn id(&self) -> TypeSysId { self.types.id() }

    pub fn get(&self, spec: impl Into<TypeSpec>) -> Option<&Ty<SemId>> {
        let sem_id = self.to_sem_id(spec)?;
        self.types.get(sem_id)
    }

    pub fn type_tree(&self, spec: impl Into<TypeSpec>) -> Option<TypeTree<'_>> {
        let sem_id = self.to_sem_id(spec)?;
        let _ = self.types.get(sem_id)?;
        Some(TypeTree::new(sem_id, self))
    }

    pub fn resolve(&self, fqn: impl Into<TypeFqn>) -> Option<&SemId> { self.symbols.get(fqn) }

    pub fn lookup(&self, sem_id: SemId) -> Option<&TypeFqn> { self.symbols.lookup(sem_id) }

    pub fn to_sem_id(&self, spec: impl Into<TypeSpec>) -> Option<SemId> {
        match spec.into() {
            TypeSpec::SemId(sem_id) => Some(sem_id),
            TypeSpec::Fqn(fqn) => self.resolve(fqn).copied(),
        }
    }

    pub fn into_type_system(self) -> TypeSystem { self.types }
}

impl Display for SymbolicSys {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typesys -- {:+}", self.id())?;
        writeln!(f)?;
        for (id, ty) in self.types.as_inner() {
            let ty = ty.clone().translate(&mut (), self).expect("type system inconsistency");
            match self.lookup(*id) {
                Some(fqn) => {
                    writeln!(f, "-- {id:-}")?;
                    writeln!(f, "data {fqn}: {:0}", ty)?;
                }
                None => writeln!(f, "data {id:-}: {:0}", ty)?,
            }
        }
        Ok(())
    }
}
