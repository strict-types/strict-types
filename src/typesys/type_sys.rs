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

//! Embedded lib is a set of compiled type libraries having no external
//! dependencies

use std::fmt::{self, Display, Formatter};
use std::ops::Index;

use amplify::confinement::{self, MediumOrdMap, SmallOrdSet};
use amplify::num::u24;
use encoding::{LibName, StrictDeserialize, StrictSerialize, TypeName};
use strict_encoding::STRICT_TYPES_LIB;

use crate::{SemId, Ty, TypeLibId};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("{lib}.{name}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct TypeFqn {
    pub lib: LibName,
    pub name: TypeName,
}

impl TypeFqn {
    pub fn with(lib: impl Into<LibName>, name: impl Into<TypeName>) -> TypeFqn {
        TypeFqn {
            lib: lib.into(),
            name: name.into(),
        }
    }
}

impl From<&'static str> for TypeFqn {
    fn from(value: &'static str) -> Self {
        let Some((lib, name)) = value.split_once('.') else {
            panic!("invalid fully qualified type name `{value}`");
        };
        TypeFqn {
            lib: LibName::from(lib),
            name: TypeName::from(name),
        }
    }
}

/// Type coupled with symbolic information.
#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct SymTy {
    pub lib: TypeLibId,
    /// Type origin providing information from which library and under which name the type is
    /// originating from. The origin information may be empty for the unnamed types.
    pub orig: Option<TypeFqn>,
    /// Type definition, potentially referencing other types via semantic type id.
    pub ty: Ty<SemId>,
}

impl SymTy {
    pub fn named(lib_id: TypeLibId, lib_name: LibName, ty_name: TypeName, ty: Ty<SemId>) -> SymTy {
        Self::with(lib_id, Some(TypeFqn::with(lib_name, ty_name)), ty)
    }

    pub fn unnamed(lib_id: TypeLibId, ty: Ty<SemId>) -> SymTy {
        Self::with(lib_id, None::<TypeFqn>, ty)
    }

    pub fn with(lib_id: TypeLibId, orig: Option<impl Into<TypeFqn>>, ty: Ty<SemId>) -> SymTy {
        SymTy {
            lib: lib_id,
            orig: orig.map(|o| o.into()),
            ty,
        }
    }
}

/// Type system represents a set of strict types assembled from multiple
/// libraries. It is designed to provide all necessary type information to
/// analyze a type with all types it depends on.
///
/// Type system contains and commits to the information on type semantic ids,
/// as well as source libraries. The reason in simple: since the semantic id
/// doesn't commit to the library name, and two semantically-distinct
/// libraries may contain the same-named type, for the complete semantic
/// information we have to take into account library ids as well.
///
/// # Type guarantees
///
/// - Total number of types do not exceed 2^24-1;
/// - Strict-serialized size is less than 2^24 bytes;
/// - A type with the same semantic id can't appear in more than 256 libraries;
/// - Type system is complete (i.e. no type references a type which is not a part of the system).
#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct TypeSystem {
    libs: SmallOrdSet<TypeLibId>,
    types: MediumOrdMap<SemId, Ty<SemId>>,
}

impl StrictSerialize for TypeSystem {}
impl StrictDeserialize for TypeSystem {}

impl TypeSystem {
    pub fn new() -> Self { Self::default() }

    pub fn count_libs(&self) -> u16 { self.libs.len_u16() }
    pub fn count_types(&self) -> u24 { self.types.len_u24() }

    pub(super) fn insert_unchecked(
        &mut self,
        lib_id: TypeLibId,
        sem_id: SemId,
        ty: Ty<SemId>,
    ) -> Result<bool, confinement::Error> {
        self.libs.push(lib_id)?;
        self.types.insert(sem_id, ty).map(|r| r.is_some())
    }

    pub fn lib_ids(&self) -> impl Iterator<Item = TypeLibId> + '_ { self.libs.iter().cloned() }
    pub fn sem_ids(&self) -> impl Iterator<Item = SemId> + '_ { self.types.keys().cloned() }
    pub fn types(&self) -> impl Iterator<Item = (&SemId, &Ty<SemId>)> + '_ { self.types.iter() }

    pub fn includes(&self, lib_id: TypeLibId) -> bool { self.libs.contains(&lib_id) }
    pub fn contains(&self, sem_id: SemId) -> bool { self.types.contains_key(&sem_id) }
    pub fn get(&self, sem_id: SemId) -> Option<&Ty<SemId>> { self.types.get(&sem_id) }
}

impl Index<SemId> for TypeSystem {
    type Output = Ty<SemId>;

    fn index(&self, index: SemId) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!("type with semantic id {index} is not a part of the type system")
        })
    }
}

impl Display for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typesys -- {}", self.id())?;
        writeln!(f)?;
        for id in &self.libs {
            writeln!(f, "uses {id:-}")?;
        }
        writeln!(f)?;
        for (id, ty) in &self.types {
            writeln!(f, "data {id:-}: {ty:-}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "armor")]
impl armor::StrictArmor for TypeSystem {
    type Id = crate::TypeSysId;
    const PLATE_TITLE: &'static str = "STRICT TYPE SYSTEM";

    fn armor_id(&self) -> Self::Id { self.id() }
}
