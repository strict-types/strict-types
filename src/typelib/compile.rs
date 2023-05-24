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

use amplify::confinement::Confined;
use amplify::RawArray;
use encoding::{LibName, LIB_EMBEDDED};
use sha2::Digest;
use strict_encoding::{StrictDumb, TypeName, STRICT_TYPES_LIB};

use crate::ast::{HashId, SEM_ID_TAG};
use crate::typelib::build::LibBuilder;
use crate::typelib::translate::{NestedContext, TranslateError, TypeIndex};
use crate::typelib::type_lib::TypeMap;
use crate::typelib::ExternRef;
use crate::{Dependency, LibRef, SemId, Translate, Ty, TypeLib, TypeLibId, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("{lib_name}.{ty_name}")]
pub struct LinkRef {
    pub lib_name: LibName,
    pub ty_name: TypeName,
    pub lib_id: TypeLibId,
    pub sem_id: SemId,
}

impl HashId for LinkRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) { hasher.update(self.sem_id.as_slice()); }
}

impl LinkRef {
    pub fn with(lib_name: LibName, ty_name: TypeName, lib_id: TypeLibId, sem_id: SemId) -> LinkRef {
        LinkRef {
            lib_name,
            ty_name,
            lib_id,
            sem_id,
        }
    }
}

impl From<LinkRef> for ExternRef {
    fn from(r: LinkRef) -> Self { ExternRef::with(r.lib_id, r.sem_id) }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { CompileRef::Named(TypeName::strict_dumb()) })]
pub enum CompileRef {
    #[from(Ty<CompileRef>)]
    Embedded(Box<Ty<CompileRef>>),
    #[from]
    Named(TypeName),
    Extern(LinkRef),
}

impl Default for Box<Ty<CompileRef>> {
    /// Provided as a workaround required due to derivation of strict types for [`CompileRef`].
    /// Always panics.
    fn default() -> Self { panic!("default method shouldn't be called on this type") }
}

impl CompileRef {
    pub fn unit() -> Self { Ty::UNIT.into() }

    pub fn sem_id(&self) -> SemId {
        let mut hasher = sha2::Sha256::new_with_prefix(&SEM_ID_TAG);
        self.hash_id(&mut hasher);
        SemId::from_raw_array(hasher.finalize())
    }
}

impl TypeRef for CompileRef {
    fn as_ty(&self) -> Option<&Ty<Self>> {
        match self {
            CompileRef::Embedded(ty) => Some(ty),
            CompileRef::Named(_) | CompileRef::Extern(_) => None,
        }
    }

    fn is_compound(&self) -> bool {
        match self {
            CompileRef::Embedded(ty) => ty.is_compound(),
            _ => false,
        }
    }
    fn is_byte(&self) -> bool {
        match self {
            CompileRef::Embedded(ty) => ty.is_byte(),
            _ => false,
        }
    }
    fn is_unicode_char(&self) -> bool {
        match self {
            CompileRef::Embedded(ty) => ty.is_unicode_char(),
            _ => false,
        }
    }
    fn is_ascii_char(&self) -> bool {
        match self {
            CompileRef::Embedded(ty) => ty.is_ascii_char(),
            _ => false,
        }
    }
}

impl HashId for CompileRef {
    fn hash_id(&self, hasher: &mut sha2::Sha256) {
        match self {
            CompileRef::Embedded(ty) => ty.hash_id(hasher),
            CompileRef::Named(name) => {
                hasher.update(name.as_bytes());
            }
            CompileRef::Extern(ext) => ext.hash_id(hasher),
        }
    }
}

impl Display for CompileRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CompileRef::Embedded(ty) => Display::fmt(ty, f),
            CompileRef::Named(name) => Display::fmt(name, f),
            CompileRef::Extern(ext) => Display::fmt(ext, f),
        }
    }
}

impl LibBuilder {
    pub fn compile(self) -> Result<TypeLib, TranslateError> {
        let name = self.name();

        let (known_libs, extern_types, types) = (self.known_libs, self.extern_types, self.types);
        let mut extern_types = Confined::try_from(extern_types).expect("too many dependencies");
        for el in types.values() {
            for subty in el.type_refs() {
                if let CompileRef::Named(name) = subty {
                    if !types.contains_key(name) {
                        return Err(TranslateError::UnknownType {
                            unknown: name.clone(),
                            within: el.clone(),
                        });
                    }
                }
            }
        }

        let mut old_types = types.into_inner();
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
                    Err(TranslateError::Continue) => {
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
            match known_libs.iter().find(|dep| &dep.name == lib) {
                None if !used_dependencies.iter().any(|dep| &dep.name == lib) => {
                    return Err(TranslateError::UnknownLib(lib.clone()))
                }
                None => {}
                Some(dep) => {
                    used_dependencies.insert(dep.clone());
                }
            }
        }

        let types = TypeMap::try_from(new_types)?;
        Ok(TypeLib {
            name,
            dependencies: Confined::try_from_iter(used_dependencies)?,
            types,
        })
    }
}
