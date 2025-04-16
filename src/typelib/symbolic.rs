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

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{self, Display, Formatter};

use amplify::confinement::{Confined, NonEmptyOrdMap, SmallOrdMap, TinyOrdMap, TinyOrdSet};
use amplify::ByteArray;
use encoding::{LibName, LIB_EMBEDDED};
use sha2::Digest;
use strict_encoding::{StrictDumb, TypeName, STRICT_TYPES_LIB};

use super::{LibBuilder, SymbolContext};
use crate::ast::{PrimitiveRef, SemCommit, SEM_ID_TAG};
use crate::typelib::{CompileError, ExternRef, NestedContext, SymbolError, TypeIndex, TypeMap};
use crate::{Dependency, LibRef, SemId, Translate, Ty, TypeLib, TypeLibId, TypeRef};

pub type ExternTypes = TinyOrdMap<LibName, SmallOrdMap<SemId, TypeName>>;

#[derive(Getters, Clone, Eq, PartialEq, Debug)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub struct SymbolicLib {
    name: LibName,
    dependencies: TinyOrdSet<Dependency>,
    extern_types: ExternTypes,
    types: NonEmptyOrdMap<TypeName, Ty<TranspileRef>>,
}

impl StrictDumb for SymbolicLib {
    fn strict_dumb() -> Self {
        SymbolicLib {
            name: strict_dumb!(),
            dependencies: strict_dumb!(),
            extern_types: strict_dumb!(),
            types: NonEmptyOrdMap::with_key_value(strict_dumb!(), strict_dumb!()),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("{lib_name}.{ty_name}", alt = "{lib_name}.{ty_name}#{sem_id:#}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename = "camelCase"))]
pub struct SymbolRef {
    pub lib_name: LibName,
    pub ty_name: TypeName,
    pub lib_id: TypeLibId,
    pub sem_id: SemId,
}

impl SymbolRef {
    pub fn with(
        lib_name: LibName,
        ty_name: TypeName,
        lib_id: TypeLibId,
        sem_id: SemId,
    ) -> SymbolRef {
        SymbolRef {
            lib_name,
            ty_name,
            lib_id,
            sem_id,
        }
    }
}

impl From<SymbolRef> for ExternRef {
    fn from(r: SymbolRef) -> Self { ExternRef::with(r.lib_id, r.sem_id) }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { Self::Named(strict_dumb!()) })]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TranspileRef {
    #[from(Ty<TranspileRef>)]
    Embedded(Box<Ty<TranspileRef>>),
    #[from]
    Named(TypeName),
    Extern(SymbolRef),
}

impl StrictDumb for Box<Ty<TranspileRef>> {
    fn strict_dumb() -> Self { Box::new(Ty::UNIT) }
}

impl TranspileRef {
    pub fn unit() -> Self { Ty::UNIT.into() }

    pub fn id(&self) -> SemId {
        if let TranspileRef::Extern(r) = self {
            r.sem_id
        } else {
            let tag = sha2::Sha256::new_with_prefix(SEM_ID_TAG).finalize();
            let mut hasher = sha2::Sha256::new();
            hasher.update(tag);
            hasher.update(tag);
            self.sem_commit(&mut hasher);
            SemId::from_byte_array(hasher.finalize())
        }
    }
}

impl TypeRef for TranspileRef {
    fn as_ty(&self) -> Option<&Ty<Self>> {
        match self {
            TranspileRef::Embedded(ty) => Some(ty),
            TranspileRef::Named(_) | TranspileRef::Extern(_) => None,
        }
    }

    fn is_compound(&self) -> bool {
        match self {
            TranspileRef::Embedded(ty) => ty.is_compound(),
            _ => false,
        }
    }
    fn is_byte(&self) -> bool {
        match self {
            TranspileRef::Embedded(ty) => ty.is_byte(),
            _ => false,
        }
    }
    fn is_unicode_char(&self) -> bool {
        match self {
            TranspileRef::Embedded(ty) => ty.is_unicode_char(),
            _ => false,
        }
    }
}

impl PrimitiveRef for TranspileRef {
    fn byte() -> Self { TranspileRef::Embedded(Box::new(Ty::BYTE)) }
    fn unicode_char() -> Self { TranspileRef::Embedded(Box::new(Ty::UNICODE)) }
}

impl Display for TranspileRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TranspileRef::Embedded(ty) => Display::fmt(ty, f),
            TranspileRef::Named(name) => Display::fmt(name, f),
            TranspileRef::Extern(ext) => Display::fmt(ext, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum TranspileError {
    /// type `{unknown}` referenced inside `{within}` is not known.
    UnknownType {
        unknown: TypeName,
        within: Ty<TranspileRef>,
    },

    /// unknown library `{0}` absent from dependencies.
    UnknownLib(LibName),

    /// too many dependencies.
    TooManyDependencies,

    /// too many types
    TooManyTypes,

    /// library `{0}` contains too many types.
    LibTooLarge(LibName),

    /// library `{0}` used as a dependency doesn't provide type `{2}` with id {1}.
    DependencyMissesType(LibName, SemId, TypeName),
}

impl LibBuilder {
    pub fn compile_symbols(self) -> Result<SymbolicLib, TranspileError> {
        assert!(
            !self.types.is_empty(),
            "library builder has no types; use `transpile` method to add types to it"
        );

        let (name, known_libs, extern_types, types) =
            (self.lib_name, self.known_libs, self.extern_types, self.types);

        for ty in types.values() {
            for (subty, _) in ty.type_refs() {
                if let TranspileRef::Named(name) = subty {
                    if !types.contains_key(name) {
                        return Err(TranspileError::UnknownType {
                            unknown: name.clone(),
                            within: ty.clone(),
                        });
                    }
                }
            }
        }

        let mut used_dependencies = HashMap::<LibName, Dependency>::new();
        for lib in extern_types.keys() {
            if lib == &libname!(LIB_EMBEDDED) {
                continue;
            }
            match known_libs.keys().find(|dep| &dep.name == lib) {
                None if !used_dependencies.contains_key(lib) => {
                    return Err(TranspileError::UnknownLib(lib.clone()));
                }
                None => {}
                Some(dep) => {
                    used_dependencies.insert(lib.clone(), dep.clone());
                }
            }
        }

        let types = Confined::try_from_iter(types).map_err(|_| TranspileError::TooManyTypes)?;
        let mut type_map = BTreeMap::new();
        for (dep_name, dep_types) in extern_types {
            for (sem_id, type_name) in &dep_types {
                let dependency_types = used_dependencies
                    .get(&dep_name)
                    .and_then(|dep| known_libs.get(dep))
                    .expect("the presence checked above");
                if !dependency_types.contains(sem_id) {
                    return Err(TranspileError::DependencyMissesType(
                        dep_name,
                        sem_id.clone(),
                        type_name.clone(),
                    ));
                }
            }
            let dep_types = SmallOrdMap::try_from_iter(dep_types)
                .map_err(|_| TranspileError::LibTooLarge(dep_name.clone()))?;
            type_map.insert(dep_name, dep_types);
        }
        let extern_types =
            TinyOrdMap::try_from(type_map).map_err(|_| TranspileError::TooManyDependencies)?;
        let dependencies = TinyOrdSet::try_from_iter(used_dependencies.into_values())
            .map_err(|_| TranspileError::TooManyDependencies)?;

        Ok(SymbolicLib {
            name,
            extern_types,
            dependencies,
            types,
        })
    }

    pub fn compile(self) -> Result<TypeLib, CompileError> { self.compile_symbols()?.compile() }
}

impl SymbolicLib {
    pub fn compile(self) -> Result<TypeLib, CompileError> {
        let name = self.name;
        let dependencies = self.dependencies;
        let mut extern_types = self.extern_types;
        let mut old_types = self.types.release();
        let mut index = TypeIndex::new();
        let mut new_types = BTreeMap::<TypeName, Ty<LibRef>>::new();
        let names = old_types.keys().cloned().collect::<BTreeSet<_>>();

        while !old_types.is_empty() {
            let mut found = false;
            for name in &names {
                let Some(ty) = old_types.get(name) else {
                    continue;
                };
                let mut ctx = NestedContext {
                    top_name: name.clone(),
                    index,
                    extern_types,
                    stack: empty!(),
                };
                let ty: Ty<LibRef> = match ty.clone().translate(&mut ctx, &()) {
                    Ok(ty) => ty,
                    Err(CompileError::Continue) => {
                        index = ctx.index;
                        extern_types = ctx.extern_types;
                        continue;
                    }
                    Err(err) => return Err(err),
                };
                index = ctx.index;
                extern_types = ctx.extern_types;
                found = true;
                let id = ty.sem_id_named(name);
                index.insert(name.clone(), id);
                new_types.insert(name.clone(), ty);
                old_types.remove(name);
            }
            debug_assert!(found, "incomplete type definition found in the library");
        }

        let mut used_dependencies = BTreeSet::<Dependency>::new();
        for lib in extern_types.keys() {
            if lib == &libname!(LIB_EMBEDDED) {
                continue;
            }
            match dependencies.iter().find(|dep| &dep.name == lib) {
                None if !used_dependencies.iter().any(|dep| &dep.name == lib) => {
                    return Err(CompileError::UnknownLib(lib.clone()));
                }
                None => {}
                Some(dep) => {
                    used_dependencies.insert(dep.clone());
                }
            }
        }

        let types = TypeMap::from_checked(new_types);
        let dependencies = Confined::from_checked(used_dependencies);

        Ok(TypeLib {
            name,
            dependencies,
            extern_types,
            types,
        })
    }
}

impl TypeLib {
    pub fn to_symbolic(&self) -> Result<SymbolicLib, SymbolError> {
        let lib_index = self.dependencies.iter().map(|dep| (dep.id, dep.name.clone())).collect();
        let reverse_index =
            self.types.iter().map(|(name, ty)| (ty.sem_id_named(name), name.clone())).collect();
        let ctx = SymbolContext {
            reverse_index,
            lib_index,
        };
        let mut extern_types = self.extern_types.clone();
        let types = Confined::try_from(
            self.types
                .iter()
                .map(|(name, ty)| {
                    Ok((name.clone(), ty.clone().translate(&mut extern_types, &ctx)?))
                })
                .collect::<Result<_, _>>()?,
        )
        .expect("same collection size");
        Ok(SymbolicLib {
            name: self.name.clone(),
            dependencies: self.dependencies.clone(),
            extern_types,
            types,
        })
    }
}
