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

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use amplify::Wrapper;

use crate::dtl::type_lib::Dependency;
use crate::dtl::{LibAlias, LibName, LibTy, TypeLib};
use crate::{Ty, TyId, TypeName, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum EmbeddedTy {
    #[from]
    Name(TypeName),

    #[from]
    Inline(Box<Ty<EmbeddedTy>>),
}

impl TypeRef for EmbeddedTy {}

impl Display for EmbeddedTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddedTy::Name(name) => write!(f, "{}", name),
            EmbeddedTy::Inline(ty) if ty.is_compound() => write!(f, "({})", ty),
            EmbeddedTy::Inline(ty) => write!(f, "{}", ty),
        }
    }
}

/// Number of types within an embedded lib is constrained by its serialized size
#[derive(Wrapper, Clone, Eq, PartialEq, Debug, From)]
#[wrapper(Deref)]
pub struct EmbeddedLib(BTreeMap<TyId, Ty<EmbeddedTy>>);

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct EmbeddedBuilder {
    dependencies: BTreeMap<LibAlias, Dependency>,
    types: BTreeMap<(LibAlias, TypeName), Ty<LibTy>>,
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum Warning {
    /// unused import `{0}` for `{1}`
    UnusedImport(LibAlias, Dependency),
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum Error {
    /// type library {0} is not imported.
    UnknownLib(LibAlias),

    /// type {2} is not present in the type library {0}. The current version of the library is {1},
    /// perhaps you need to import a different version.
    TypeAbsent(LibAlias, Dependency, TypeName),
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
        /*
        for ty in self.types.values() {
            for st in ty {
                if matches!(st, GravelTy::Extern(n, a) if a == alias && n == name)
                {
                }
            }
        }
         */
        todo!()
    }
}
