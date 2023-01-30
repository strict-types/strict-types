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

use amplify::confinement;
use strict_encoding::{StrictDumb, TypeName, STRICT_TYPES_LIB};

use crate::ast::NestedRef;
use crate::typelib::build::LibBuilder;
use crate::typelib::translate::{NestedContext, Translate, TranslateError, TypeIndex};
use crate::typelib::type_lib::{LibType, TypeMap};
use crate::{Dependency, LibAlias, SemId, Ty, TypeLib, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display("data {name} :: {ty}")]
pub struct CompileType {
    pub name: TypeName,
    pub ty: Ty<CompileRef>,
}

impl CompileType {
    pub fn new(name: TypeName, ty: Ty<CompileRef>) -> Self { CompileType { name, ty } }
}

impl Default for Box<Ty<CompileRef>> {
    /// Provided as a workaround required due to derivation of strict types for [`CompileRef`].
    /// Always panics.
    fn default() -> Self { panic!("default method shouldn't be called on this type") }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB, tags = order, dumb = { CompileRef::Named(TypeName::strict_dumb()) })]
pub enum CompileRef {
    #[from(Ty<CompileRef>)]
    Embedded(Box<Ty<CompileRef>>),
    #[from]
    Named(TypeName),
    Extern(TypeName, LibAlias),
}

impl CompileRef {
    pub fn unit() -> Self { Ty::UNIT.into() }
}

impl TypeRef for CompileRef {
    const TYPE_NAME: &'static str = "CompileRef";
    fn id(&self) -> SemId { unreachable!("CompileRef must never be called for the id") }
}

impl NestedRef for CompileRef {
    type Ref = CompileRef;

    fn as_ty(&self) -> Option<&Ty<Self>> {
        match self {
            CompileRef::Embedded(ty) => Some(ty),
            CompileRef::Named(_) | CompileRef::Extern(_, _) => None,
        }
    }
}

impl Display for CompileRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CompileRef::Embedded(ty) if ty.is_compound() => write!(f, "({ty})"),
            CompileRef::Embedded(ty) => Display::fmt(ty, f),
            CompileRef::Named(name) => write!(f, "{name}"),
            CompileRef::Extern(name, lib) => write!(f, "{lib}.{name}"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum Warning {
    /// unused import `{0}` for `{1}`
    UnusedImport(LibAlias, Dependency),

    /// type {1} from library {0} with id {2} is already known
    RepeatedType(LibAlias, TypeName, SemId),
}

#[derive(Clone, Eq, PartialEq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// type library {0} is not imported.
    UnknownLib(LibAlias),

    /// type {2} is not present in the type library {0}. The current version of the library is {1},
    /// perhaps you need to import a different version.
    TypeAbsent(LibAlias, Dependency, TypeName),

    #[from]
    #[display(inner)]
    Confinement(confinement::Error),

    /// type {name} in {dependency} expected to have a type id {expected} but {found} is found.
    /// Perhaps a wrong version of the library is used?
    TypeMismatch {
        dependency: Dependency,
        name: TypeName,
        expected: SemId,
        found: SemId,
    },
}

impl LibBuilder {
    pub fn compile(self) -> Result<TypeLib, TranslateError> {
        // TODO: Build dependency list

        let name = self.name();

        let types = self.into_types();
        for el in types.values() {
            for subty in el.ty.type_refs() {
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
        let mut new_types = BTreeMap::<TypeName, LibType>::new();
        let names = old_types.keys().cloned().collect::<BTreeSet<_>>();

        while !old_types.is_empty() {
            let mut found = false;
            for name in &names {
                let Some(ty) = old_types.get(name).map(|c| &c.ty) else {
                    continue
                };
                let mut ctx = NestedContext {
                    top_name: name.clone(),
                    index,
                    stack: empty!(),
                };
                let ty = match ty.clone().translate(&mut ctx) {
                    Ok(ty) => ty,
                    Err(TranslateError::Continue) => {
                        index = ctx.index;
                        continue;
                    }
                    Err(err) => return Err(err),
                };
                index = ctx.index;
                found = true;
                let id = ty.id(Some(name));
                index.insert(name.clone(), id);
                new_types.insert(name.clone(), LibType::with(name.clone(), ty));
                old_types.remove(name);
            }
            debug_assert!(found, "incomplete type definition found in the library");
        }

        let types = TypeMap::try_from(new_types)?;
        Ok(TypeLib {
            name,
            dependencies: none!(),
            types,
        })
    }
}
