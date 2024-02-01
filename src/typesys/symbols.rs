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
use std::fmt::{self, Display, Formatter};
use std::mem::swap;
use std::ops::Index;

use amplify::confinement::{self, MediumOrdSet, SmallOrdSet};
use encoding::constants::UNIT;
use encoding::{StrictDeserialize, StrictSerialize, STRICT_TYPES_LIB};

use crate::ast::ItemCase;
use crate::stl::LIB_ID_STD;
use crate::typesys::{translate, SymTy, TypeFqn, TypeSymbol, TypeSysId};
use crate::typify::TypeSpec;
use crate::{ast, Dependency, SemId, Translate, Ty, TypeSystem};

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

#[cfg(feature = "base64")]
impl fmt::UpperHex for Symbols {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::confinement::U32;
        use base64::Engine;

        writeln!(f, "-----BEGIN STRICT SYMBOL SYSTEM-----")?;
        writeln!(f)?;

        let data = self.to_strict_serialized::<U32>().expect("in-memory");
        let engine = base64::engine::general_purpose::STANDARD;
        let data = engine.encode(data);
        let mut data = data.as_str();
        while data.len() >= 64 {
            let (line, rest) = data.split_at(64);
            writeln!(f, "{}", line)?;
            data = rest;
        }
        writeln!(f, "{}", data)?;

        writeln!(f, "\n-----END STRICT SYMBOL SYSTEM-----")?;
        Ok(())
    }
}

#[derive(Getters, Clone, Eq, PartialEq, Debug)]
#[getter(prefix = "as_")]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct SymbolicSys {
    symbols: Symbols,
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
        Some(TypeTree { sem_id, sys: self })
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
                    writeln!(f, "-- {id:0}")?;
                    writeln!(f, "data {fqn} :: {:0}", ty)?;
                }
                None => writeln!(f, "data {id:0} :: {:0}", ty)?,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "base64")]
impl fmt::UpperHex for SymbolicSys {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::confinement::U32;
        use base64::Engine;

        let id = self.id();

        writeln!(f, "-----BEGIN STRICT SYMBOLIC TYPES-----")?;
        writeln!(f, "Id: {}", id)?;
        writeln!(f)?;

        let data = self.to_strict_serialized::<U32>().expect("in-memory");
        let engine = base64::engine::general_purpose::STANDARD;
        let data = engine.encode(data);
        let mut data = data.as_str();
        while data.len() >= 64 {
            let (line, rest) = data.split_at(64);
            writeln!(f, "{}", line)?;
            data = rest;
        }
        writeln!(f, "{}", data)?;

        writeln!(f, "\n-----END STRICT SYMBOLIC TYPES-----")?;
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TypeTree<'sys> {
    sem_id: SemId,
    sys: &'sys SymbolicSys,
}

impl<'sys> TypeTree<'sys> {
    pub fn get(&self) -> &Ty<SemId> { self.sys.get(self.sem_id).expect("inconsistent type tree") }

    pub fn iter(&'sys self) -> TypeTreeIter<'sys> {
        TypeTreeIter {
            sem_id: self.sem_id,
            ty: Some(self.get()),
            item: None,
            depth: 0,
            path: vec![],
            sys: &self.sys,
            wrapped: false,
        }
    }
}

/*
impl<'sys> IntoIterator for TypeTree<'sys> {
    type Item = (usize, &'sys Ty<SemId>, Option<&'sys TypeFqn>);
    type IntoIter = TypeTreeIter<'sys>;

    fn into_iter(self) -> Self::IntoIter { self.iter() }
}
 */

impl<'tree, 'sys> IntoIterator for &'tree TypeTree<'sys>
where 'tree: 'sys
{
    type Item = TypeInfo<'sys>;
    type IntoIter = TypeTreeIter<'sys>;

    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'sys> Display for TypeTree<'sys> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for TypeInfo {
            depth,
            ty,
            fqn,
            item,
            wrapped,
        } in self
        {
            write!(f, "{: ^1$}", "", depth * 2)?;
            let name = fqn.map(|f| f.name.to_string()).unwrap_or_else(|| s!("_"));
            let comment = match item {
                Some(ItemCase::UnnamedField(pos)) => {
                    write!(f, "{name}")?;
                    if name == "_" {
                        write!(f, "_{pos}")?;
                    }
                    None
                }
                Some(ItemCase::NamedField(_, ref fname)) => {
                    write!(f, "{fname}")?;
                    fqn.map(|_| name)
                }
                Some(ItemCase::UnionVariant(_, ref vname)) => {
                    write!(f, "{vname}")?;
                    fqn.map(|_| name)
                }
                Some(ItemCase::MapKey) if fqn.is_some() => {
                    write!(f, "{name}")?;
                    Some(s!("map key"))
                }
                Some(ItemCase::MapKey) => {
                    f.write_str("mapKey")?;
                    None
                }
                Some(ItemCase::MapValue) if fqn.is_some() => {
                    write!(f, "{name}")?;
                    Some(s!("map value"))
                }
                Some(ItemCase::MapValue) => {
                    f.write_str("mapValue")?;
                    None
                }
                _ => {
                    write!(f, "{name}")?;
                    None
                }
            };
            match ty {
                Ty::Primitive(prim) if *prim == UNIT => write!(f, " as Unit")?,
                Ty::Primitive(prim) => write!(f, " as {prim}")?,
                _ => write!(f, " {}", ty.cls())?,
            }
            if wrapped {
                f.write_str(" wrapped")?;
            }
            if let Some(ItemCase::UnionVariant(ref pos, _)) = item {
                write!(f, " tag={pos}")?;
            }
            if let Ty::Enum(vars) = ty {
                const MAX_LINE_VARS: usize = 8;
                if vars.len() > MAX_LINE_VARS {
                    write!(f, " {{\n{: ^1$}", "", depth * 2 + 2)?;
                }
                for (pos, var) in vars.iter().enumerate() {
                    write!(f, " {pos}={var}")?;
                    if pos > 0 && pos % MAX_LINE_VARS == 0 {
                        write!(f, "\n{: ^1$}", "", depth * 2 + 2)?;
                    }
                }
                if vars.len() > MAX_LINE_VARS {
                    write!(f, "\n{: ^1$}}}", "", depth * 2)?;
                }
            }
            if let Some(comment) = comment {
                write!(f, " -- {comment}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/*
#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
#[display(lowercase)]
pub enum NestedCase {
    NewType,
    Option,
    ByteArray,
    ByteStr,
    AsciiStr,
    UniStr,
}

pub struct NestedInfo<'sys> {
    pub inner: Option<&'sys TypeFqn>,
    pub case: NestedCase,
}
 */

#[derive(Clone, Debug)]
pub struct TypeInfo<'sys> {
    pub depth: usize,
    pub ty: &'sys Ty<SemId>,
    pub fqn: Option<&'sys TypeFqn>,
    pub item: Option<ItemCase>,
    pub wrapped: bool,
    // pub nested: Option<NestedInfo<'sys>>,
}

/*
impl<'sys> TypeInfo<'sys> {
    pub fn with(depth: usize, ty: &'sys Ty<SemId>, fqn: Option<&'sys TypeFqn>) -> Self {
        Self {
            depth,
            ty,
            fqn,
            wrapped: false,
            optional: false,
        }
    }
}
 */

pub struct TypeTreeIter<'sys> {
    sem_id: SemId,
    ty: Option<&'sys Ty<SemId>>,
    item: Option<ItemCase>,
    depth: usize,
    path: Vec<(usize, ast::Iter<'sys, SemId>)>,
    sys: &'sys SymbolicSys,
    wrapped: bool,
}

impl<'sys> Iterator for TypeTreeIter<'sys> {
    type Item = TypeInfo<'sys>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ty) = self.ty {
            let fqn = self.sys.symbols.lookup(self.sem_id);
            self.ty = None;
            if !ty.is_newtype() {
                self.depth += 1;
            }
            self.path.push((self.depth, ty.iter()));
            if matches!(fqn, Some(TypeFqn {lib, .. }) if lib.to_string() == LIB_ID_STD) {
                // Skipping standard types
            } else if !ty.is_newtype() {
                let mut item = None;
                swap(&mut item, &mut self.item);
                return Some(TypeInfo {
                    depth: self.depth - 1,
                    ty,
                    fqn,
                    item,
                    wrapped: self.wrapped,
                });
            } else {
                self.wrapped = true
            }
        }
        loop {
            let (depth, iter) = self.path.last_mut()?;
            self.depth = *depth;
            match iter.next() {
                None => {
                    self.path.pop();
                    self.wrapped = false;
                    continue;
                }
                Some((id, item)) => {
                    self.sem_id = *id;
                    if !self.wrapped {
                        self.item = item;
                    }
                    self.ty = self.sys.get(*id);
                    return self.next();
                }
            }
        }
    }
}
