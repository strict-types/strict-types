// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Project.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.

use std::collections::{BTreeMap, BTreeSet};

use amplify::confinement::{Confined, SmallOrdMap};
use amplify::{confinement, Wrapper};

use crate::ast::{NestedRef, TranslateError, TyInner};
use crate::dtl::type_lib::Dependency;
use crate::dtl::{EmbeddedTy, LibAlias, LibName, LibTy, TypeIndex, TypeLib, TypeSystem};
use crate::{StenType, Translate, Ty, TyId, TypeName};

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum Warning {
    /// unused import `{0}` for `{1}`
    UnusedImport(LibAlias, Dependency),

    /// type {1} from library {0} with id {2} is already known
    RepeatedType(LibAlias, TypeName, TyId),
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
        expected: TyId,
        found: TyId,
    },
}

#[derive(Default)]
pub struct LibBuilder {
    index: TypeIndex,
    types: SmallOrdMap<TypeName, Ty<LibTy>>,
}

impl LibBuilder {
    pub(crate) fn with(index: TypeIndex) -> LibBuilder {
        LibBuilder {
            index,
            types: default!(),
        }
    }

    pub(crate) fn finalize(self, name: LibName) -> Result<TypeLib, confinement::Error> {
        let types = Confined::try_from(self.types.into_inner())?;
        Ok(TypeLib {
            name,
            dependencies: none!(),
            types,
        })
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SystemBuilder {
    pub(super) dependencies: BTreeMap<LibAlias, Dependency>,
    types: BTreeMap<(LibAlias, TypeName), Ty<LibTy>>,
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum ImportError {
    /// type library {0} is not a dependency and can't be imported
    Absent(LibName),
}

impl SystemBuilder {
    pub fn new() -> SystemBuilder { SystemBuilder::default() }

    pub fn import(&mut self, name: LibName, lib: TypeLib) -> Result<(), ImportError> {
        let Some((alias, _)) = self.dependencies.iter().find(|(_, d)| d.name == name) else {
            return Err(ImportError::Absent(name))
        };
        let alias = alias.clone();
        self.dependencies.remove(&alias);
        self.types
            .extend(lib.types.into_iter().map(|(ty_name, ty)| ((alias.clone(), ty_name), ty)));
        self.dependencies.extend(lib.dependencies);

        Ok(())
    }

    pub fn finalize(mut self) -> Result<(TypeSystem, Vec<Warning>), Vec<Error>> {
        let mut warnings: Vec<Warning> = empty!();
        let mut errors: Vec<Error> = empty!();
        let mut lib: BTreeSet<Ty<EmbeddedTy>> = empty!();

        for ((lib_alias, ty_name), ty) in self.types.clone() {
            match ty.clone().translate(&mut self) {
                Err(err) => errors.push(err),
                Ok(ty) => {
                    let id = ty.id();
                    if !lib.insert(ty) {
                        warnings.push(Warning::RepeatedType(lib_alias, ty_name, id))
                    }
                }
            }
        }

        match TypeSystem::try_from_iter(lib) {
            Err(err) => {
                errors.push(err.into());
                return Err(errors);
            }
            Ok(lib) if errors.is_empty() => Ok((lib, warnings)),
            Ok(_) => Err(errors),
        }
    }
}

impl Translate<TypeLib> for StenType {
    type Context = LibName;
    type Error = TranslateError;

    fn translate(self, lib_name: &mut Self::Context) -> Result<TypeLib, Self::Error> {
        let id = self.ty.id();

        let index = self.build_index()?;

        let mut ctx = LibBuilder::with(index);
        let root = self.ty.translate(&mut ctx)?;

        let name = ctx.index.remove(&id).ok_or(TranslateError::UnknownId(id))?;
        let mut lib = ctx.finalize(lib_name.clone())?;
        if lib.types.insert(name.clone(), root)?.is_some() {
            return Err(TranslateError::DuplicateName(name));
        }

        Ok(lib)
    }
}

impl Translate<LibTy> for StenType {
    type Context = LibBuilder;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<LibTy, Self::Error> {
        let id = self.id();
        let ty = self.into_ty().translate(ctx)?;
        Ok(match ctx.index.get(&id) {
            Some(name) => {
                if !ctx.types.contains_key(name) {
                    ctx.types.insert(name.clone(), ty)?;
                }
                LibTy::Named(name.clone(), id)
            }
            None => LibTy::Inline(Box::new(ty)),
        })
    }
}

impl Translate<EmbeddedTy> for LibTy {
    type Context = SystemBuilder;
    type Error = Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<EmbeddedTy, Self::Error> {
        match self {
            LibTy::Named(_, id) => Ok(EmbeddedTy::Ref(id)),
            LibTy::Inline(inline_ty) => {
                inline_ty.translate(ctx).map(Box::new).map(EmbeddedTy::Inline)
            }
            LibTy::Extern(ty_name, lib_alias, id) => {
                let dep =
                    ctx.dependencies.get(&lib_alias).ok_or(Error::UnknownLib(lib_alias.clone()))?;
                let ty = ctx
                    .types
                    .get(&(lib_alias.clone(), ty_name.clone()))
                    .ok_or_else(|| Error::TypeAbsent(lib_alias, dep.clone(), ty_name.clone()))?;
                if id != ty.id() {
                    return Err(Error::TypeMismatch {
                        dependency: dep.clone(),
                        name: ty_name,
                        expected: id,
                        found: ty.id(),
                    });
                }
                Ok(EmbeddedTy::Ref(id))
            }
        }
    }
}

impl StenType {
    pub fn build_index(&self) -> Result<TypeIndex, TranslateError> {
        let mut index = empty!();
        self.index(&mut index).map(|_| index)
    }

    pub fn index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
        if self.name.is_empty() {
            return Ok(());
        }

        let id = self.ty.id();
        let name = TypeName::from(self.name);
        match index.get(&id) {
            None => index.insert(id, name),
            Some(n) if n != &name => return Err(TranslateError::DuplicateName(name)),
            _ => None,
        };

        self.ty.build_index(index)?;

        Ok(())
    }
}

impl Ty<StenType> {
    pub fn build_index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
        match self.as_inner() {
            TyInner::Union(fields) => {
                for ty in fields.values() {
                    ty.index(index)?;
                }
            }
            TyInner::Struct(fields) => {
                for ty in fields.values() {
                    ty.index(index)?;
                }
            }
            TyInner::Array(ty, _)
            | TyInner::List(ty, _)
            | TyInner::Set(ty, _)
            | TyInner::Map(_, ty, _) => ty.index(index)?,
            _ => {}
        }
        Ok(())
    }
}
