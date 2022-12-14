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

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use amplify::ascii::AsciiString;
use amplify::confinement::{Confined, TinyOrdMap};

use crate::ast::{NestedRef, TranslateError};
use crate::dtl::id::TypeLibId;
use crate::{Ident, SemVer, StenType, Translate, Ty, TyId, TypeName, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum LibTy {
    Named(TypeName, TyId),

    #[from]
    Inline(Box<Ty<LibTy>>),

    Extern(TypeName, LibAlias, TyId),
}

impl TypeRef for LibTy {
    fn id(&self) -> TyId {
        match self {
            LibTy::Named(_, id) | LibTy::Extern(_, _, id) => *id,
            LibTy::Inline(ty) => ty.id(),
        }
    }
}

impl Deref for LibTy {
    type Target = Ty<Self>;

    fn deref(&self) -> &Self::Target { self.as_ty() }
}

impl NestedRef for LibTy {
    fn as_ty(&self) -> &Ty<Self> {
        match self {
            LibTy::Named(_, _) => &Ty::UNIT,
            LibTy::Inline(ty) => ty.as_ref(),
            LibTy::Extern(_, _, _) => &Ty::UNIT,
        }
    }

    fn into_ty(self) -> Ty<Self> {
        match self {
            LibTy::Named(_, _) => Ty::UNIT,
            LibTy::Inline(ty) => *ty,
            LibTy::Extern(_, _, _) => Ty::UNIT,
        }
    }
}

impl Display for LibTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LibTy::Named(name, _) => write!(f, "{}", name),
            LibTy::Inline(ty) if ty.is_compound() => write!(f, "({})", ty),
            LibTy::Inline(ty) => write!(f, "{}", ty),
            LibTy::Extern(name, lib, _) => write!(f, "{}.{}", lib, name),
        }
    }
}

pub type LibAlias = Ident;
pub type LibName = Ident;

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display("typelib {name}@{ver} {id:#}")]
pub struct Dependency {
    pub id: TypeLibId,
    pub name: LibName,
    pub ver: SemVer,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TypeLib {
    pub name: LibName,
    pub dependencies: TinyOrdMap<LibAlias, Dependency>,
    pub types: Confined<BTreeMap<TypeName, Ty<LibTy>>, 1, { u16::MAX as usize }>,
}

impl TypeLib {
    pub fn with(name: String, root: StenType) -> Result<Self, TranslateError> {
        let mut name = LibName::try_from(
            AsciiString::from_ascii(name.clone())
                .map_err(|_| TranslateError::InvalidLibName(name.clone()))?,
        )
        .map_err(|_| TranslateError::InvalidLibName(name))?;
        root.translate(&mut name)
    }
}

impl Display for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typemod {}", self.name)?;
        writeln!(f)?;
        for (alias, dep) in &self.dependencies {
            if alias != &dep.name {
                writeln!(f, "{} as {}", dep, alias)?;
            } else {
                Display::fmt(dep, f)?;
            }
        }
        writeln!(f)?;
        for (name, ty) in &self.types {
            writeln!(f, "data {:16} :: {}", name, ty)?;
        }
        Ok(())
    }
}
