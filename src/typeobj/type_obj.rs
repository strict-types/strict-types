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

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};

use amplify::confinement::{Confined, SmallOrdMap, TinyOrdMap, TinyOrdSet};
use amplify::RawArray;
use encoding::{LibName, LIB_EMBEDDED};
use sha2::Digest;
use strict_encoding::{StrictDumb, TypeName, STRICT_TYPES_LIB};

use super::Transpiler;
use crate::ast::{HashId, SEM_ID_TAG};
use crate::typelib::{CompileError, ExternRef, NestedContext, TypeIndex, TypeMap};
use crate::{Dependency, LibRef, SemId, Translate, Ty, TypeLib, TypeLibId, TypeRef};

#[derive(Getters, Clone, Eq, PartialEq, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct TypeObjects {
    name: LibName,
    dependencies: TinyOrdSet<Dependency>,
    extern_types: TinyOrdMap<LibName, SmallOrdMap<TypeName, SemId>>,
    types: SmallOrdMap<TypeName, Ty<TranspileRef>>,
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("{lib_name}.{ty_name}")]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "camelCase")
)]
pub struct SymbolRef {
    pub lib_name: LibName,
    pub ty_name: TypeName,
    pub lib_id: TypeLibId,
    pub sem_id: SemId,
}

impl HashId for SymbolRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) { hasher.update(self.sem_id.as_slice()); }
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub enum TranspileRef {
    #[from(Ty<TranspileRef>)]
    Embedded(Box<Ty<TranspileRef>>),
    #[from]
    Named(TypeName),
    Extern(SymbolRef),
}

impl Default for Box<Ty<TranspileRef>> {
    /// Provided as a workaround required due to derivation of strict types for [`TranspileRef`].
    /// Always panics.
    fn default() -> Self { panic!("default method shouldn't be called on this type") }
}

impl TranspileRef {
    pub fn unit() -> Self { Ty::UNIT.into() }

    pub fn sem_id(&self) -> SemId {
        let mut hasher = sha2::Sha256::new_with_prefix(&SEM_ID_TAG);
        self.hash_id(&mut hasher);
        SemId::from_raw_array(hasher.finalize())
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
    fn is_ascii_char(&self) -> bool {
        match self {
            TranspileRef::Embedded(ty) => ty.is_ascii_char(),
            _ => false,
        }
    }
}

impl HashId for TranspileRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        match self {
            TranspileRef::Embedded(ty) => ty.hash_id(hasher),
            TranspileRef::Named(name) => {
                hasher.update(name.as_bytes());
            }
            TranspileRef::Extern(ext) => ext.hash_id(hasher),
        }
    }
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
}

impl Transpiler {
    pub fn compile_symbols(self) -> Result<TypeObjects, TranspileError> {
        let (name, known_libs, extern_types, types) =
            (self.lib_name, self.known_libs, self.extern_types, self.types);

        for ty in types.values() {
            for subty in ty.type_refs() {
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

        let mut used_dependencies = BTreeSet::<Dependency>::new();
        for lib in extern_types.keys() {
            if lib == &libname!(LIB_EMBEDDED) {
                continue;
            }
            match known_libs.iter().find(|dep| &dep.name == lib) {
                None if !used_dependencies.iter().any(|dep| &dep.name == lib) => {
                    return Err(TranspileError::UnknownLib(lib.clone()))
                }
                None => {}
                Some(dep) => {
                    used_dependencies.insert(dep.clone());
                }
            }
        }

        let dependencies = Confined::try_from(used_dependencies)
            .map_err(|_| TranspileError::TooManyDependencies)?;
        let types = Confined::try_from(types).map_err(|_| TranspileError::TooManyTypes)?;
        let extern_types = Confined::try_from(
            extern_types
                .into_iter()
                .map(|(k, v)| {
                    let v = Confined::try_from(v)
                        .map_err(|_| TranspileError::LibTooLarge(k.clone()))?;
                    Ok((k, v))
                })
                .collect::<Result<_, _>>()?,
        )
        .map_err(|_| TranspileError::TooManyDependencies)?;
        Ok(TypeObjects {
            name,
            extern_types,
            dependencies,
            types,
        })
    }

    pub fn compile(self) -> Result<TypeLib, CompileError> { self.compile_symbols()?.compile() }
}

impl TypeObjects {
    pub fn compile(self) -> Result<TypeLib, CompileError> {
        let name = self.name;
        let dependencies = self.dependencies;
        let mut extern_types = self.extern_types;
        let mut old_types = self.types.into_inner();
        let mut index = TypeIndex::new();
        let mut new_types = BTreeMap::<TypeName, Ty<LibRef>>::new();
        let names = old_types.keys().cloned().collect::<BTreeSet<_>>();

        while !old_types.is_empty() {
            let mut found = false;
            for name in &names {
                let Some(ty) = old_types.get(name) else {
                    continue
                };
                let mut ctx = NestedContext {
                    top_name: name.clone(),
                    index,
                    extern_types,
                    stack: empty!(),
                };
                let ty = match ty.clone().translate(&mut ctx, &()) {
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
                let id = ty.id(Some(name));
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
                    return Err(CompileError::UnknownLib(lib.clone()))
                }
                None => {}
                Some(dep) => {
                    used_dependencies.insert(dep.clone());
                }
            }
        }

        let types = TypeMap::try_from(new_types).expect("same collection size");
        let dependencies = Confined::try_from(used_dependencies).expect("same collection size");

        Ok(TypeLib {
            name,
            dependencies,
            types,
        })
    }
}