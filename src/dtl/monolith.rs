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

//! Monolith is a set of compiled type libraries having no external dependencies

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use amplify::Wrapper;

use crate::dtl::gravel::Dependency;
use crate::dtl::{Gravel, GravelAlias, GravelName, GravelTy};
use crate::{Ty, TyId, TypeName, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum MonolithTy {
    #[from]
    Name(TypeName),

    #[from]
    Inline(Box<Ty<MonolithTy>>),
}

impl TypeRef for MonolithTy {}

impl Display for MonolithTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MonolithTy::Name(name) => write!(f, "{}", name),
            MonolithTy::Inline(ty) if ty.is_compound() => write!(f, "({})", ty),
            MonolithTy::Inline(ty) => write!(f, "{}", ty),
        }
    }
}

/// Number of types withing monolith is constrained by its serialized size
#[derive(Wrapper, Clone, Eq, PartialEq, Debug, From)]
#[wrapper(Deref)]
pub struct Monolith(BTreeMap<TyId, Ty<MonolithTy>>);

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct MonolithBuilder {
    dependencies: BTreeMap<GravelAlias, Dependency>,
    types: BTreeMap<(GravelAlias, TypeName), Ty<GravelTy>>,
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum Warning {
    /// unused import `{0}` for `{1}`
    UnusedImport(GravelAlias, Dependency),
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum Error {
    /// type library {0} is not imported.
    UnknownGravel(GravelAlias),

    /// type {2} is not present in the type library {0}. The current version of the library is {1},
    /// perhaps you need to import a different version.
    TypeAbsent(GravelAlias, Dependency, TypeName),
}

#[derive(Clone, Eq, PartialEq, Debug, Display)]
#[display(doc_comments)]
pub enum ImportError {
    /// type library {0} is not a dependency and can't be imported
    Absent(GravelName),
}

impl MonolithBuilder {
    pub fn new() -> MonolithBuilder { MonolithBuilder::default() }

    pub fn import(&mut self, name: GravelName, gravel: Gravel) -> Result<(), ImportError> {
        let Some((alias, _)) = self.dependencies.iter().find(|(_, d)| d.name == name) else {
            return Err(ImportError::Absent(name))
        };
        let alias = alias.clone();
        self.dependencies.remove(&alias);
        self.types
            .extend(gravel.types.into_iter().map(|(ty_name, ty)| ((alias.clone(), ty_name), ty)));
        self.dependencies.extend(gravel.dependencies);

        Ok(())
    }

    pub fn finalize(self) -> Result<(Monolith, Vec<Warning>), Vec<Error>> {
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
