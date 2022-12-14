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

//! Embedded lib is a set of compiled type libraries having no external
//! dependencies

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use amplify::confinement;
use amplify::confinement::MediumOrdMap;
use amplify::num::u24;

use crate::ast::NestedRef;
use crate::dtl::type_lib::Dependency;
use crate::dtl::{LibAlias, LibName, LibTy, TypeLib};
use crate::{Serialize, Ty, TyId, TypeName, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum EmbeddedTy {
    Name(LibName, TypeName),

    #[from]
    Inline(Box<Ty<EmbeddedTy>>),
}

impl Deref for EmbeddedTy {
    type Target = Ty<EmbeddedTy>;

    fn deref(&self) -> &Self::Target {
        match self {
            EmbeddedTy::Name(_, _) => &Ty::UNIT,
            EmbeddedTy::Inline(ty) => ty.as_ref(),
        }
    }
}

impl TypeRef for EmbeddedTy {}

impl NestedRef for EmbeddedTy {
    fn as_ty(&self) -> &Ty<Self> { self.deref() }

    fn into_ty(self) -> Ty<Self> {
        match self {
            EmbeddedTy::Name(_, _) => Ty::UNIT,
            EmbeddedTy::Inline(ty) => *ty,
        }
    }
}

impl Display for EmbeddedTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddedTy::Name(lib, name) => write!(f, "{}.{}", lib, name),
            EmbeddedTy::Inline(ty) if ty.is_compound() => write!(f, "({})", ty),
            EmbeddedTy::Inline(ty) => write!(f, "{}", ty),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub struct EmbeddedLib(MediumOrdMap<TyId, Ty<EmbeddedTy>>);

impl Deref for EmbeddedLib {
    type Target = BTreeMap<TyId, Ty<EmbeddedTy>>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl IntoIterator for EmbeddedLib {
    type Item = (TyId, Ty<EmbeddedTy>);
    type IntoIter = std::collections::btree_map::IntoIter<TyId, Ty<EmbeddedTy>>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'lib> IntoIterator for &'lib EmbeddedLib {
    type Item = (&'lib TyId, &'lib Ty<EmbeddedTy>);
    type IntoIter = std::collections::btree_map::Iter<'lib, TyId, Ty<EmbeddedTy>>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl EmbeddedLib {
    pub fn try_from_iter<T: IntoIterator<Item = (TyId, Ty<EmbeddedTy>)>>(
        iter: T,
    ) -> Result<Self, confinement::Error> {
        let mut lib: BTreeMap<TyId, Ty<EmbeddedTy>> = empty!();
        for (id, ty) in iter {
            lib.insert(id, ty);
        }

        let lib = EmbeddedLib(MediumOrdMap::try_from_iter(lib)?);
        let len = lib.serialized_len();
        let max_len = u24::MAX.into_usize();
        if len > max_len {
            return Err(confinement::Error::Oversize { len, max_len }.into());
        }
        Ok(lib)
    }

    pub fn count_types(&self) -> u24 { self.0.len_u24() }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct EmbeddedBuilder {
    pub(super) dependencies: BTreeMap<LibAlias, Dependency>,
    types: BTreeMap<(LibAlias, TypeName), Ty<LibTy>>,
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum Warning {
    /// unused import `{0}` for `{1}`
    UnusedImport(LibAlias, Dependency),
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
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum ImportError {
    /// type library {0} is not a dependency and can't be imported
    Absent(LibName),
}

impl EmbeddedBuilder {
    pub fn new() -> EmbeddedBuilder { EmbeddedBuilder::default() }

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

    pub fn finalize(self) -> Result<(EmbeddedLib, Vec<Warning>), Vec<Error>> {
        let mut warnings: Vec<Warning> = empty!();
        let mut errors: Vec<Error> = empty!();
        let mut lib: BTreeSet<Ty<EmbeddedTy>> = empty!();

        for ((lib_alias, ty_name), ty) in &self.types {
            let lib_name =
                &self.dependencies.get(lib_alias).expect("internal builder inconsistency").name;
            for st in &ty {
                match st {
                    LibTy::Named(name) => EmbeddedTy::Name(lib_name.clone(), name),
                    LibTy::Inline(inline_ty) => EmbeddedTy::Inline(inline_ty),
                    LibTy::Extern(ext_ty_name, ext_lib) => {}
                }
            }
        }

        match EmbeddedLib::try_from_iter(lib) {
            Err(err) => {
                errors.push(err.into());
                return Err(errors);
            }
            Ok(lib) if errors.is_empty() => Ok((lib, warnings)),
            Ok(_) => Err(errors),
        }
    }
}
