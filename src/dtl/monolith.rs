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
use crate::dtl::{Gravel, GravelAlias, GravelTy};
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
    pub uses: BTreeMap<GravelAlias, Dependency>,
    pub types: BTreeMap<TypeName, Ty<GravelTy>>,
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

impl MonolithBuilder {
    pub fn new() -> MonolithBuilder { MonolithBuilder::default() }

    pub fn import(&mut self, _gravel: Gravel) {}

    pub fn finalize(self) -> Result<(Monolith, Vec<Warning>), Vec<Error>> { todo!() }
}
