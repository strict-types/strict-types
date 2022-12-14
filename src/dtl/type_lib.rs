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

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::ops::Deref;

use amplify::confinement::{Confined, TinyOrdMap};
use amplify::Wrapper;

use crate::ast::{NestedRef, TranslateError};
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

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref)]
pub struct TypeLibId(blake3::Hash);

impl Ord for TypeLibId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for TypeLibId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Display for TypeLibId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let m = mnemonic::to_string(&self.as_bytes()[14..18]);
            write!(f, "{}#{}", self.0, m)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

pub struct Hasher(blake3::Hasher);

// TODO: Use real tag
pub const LIB_ID_TAG: [u8; 32] = [0u8; 32];

impl Hasher {
    pub fn new() -> Hasher { Hasher(blake3::Hasher::new_keyed(&LIB_ID_TAG)) }

    pub fn input(&mut self, id: TyId) {
        self.0.write_all(id.as_bytes()).expect("hashers do not error")
    }

    pub fn finish(self) -> TypeLibId { TypeLibId(self.0.finalize()) }
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
    pub roots: BTreeSet<TyId>,
    pub dependencies: TinyOrdMap<LibAlias, Dependency>,
    pub types: Confined<BTreeMap<TypeName, Ty<LibTy>>, 1, { u16::MAX as usize }>,
}

impl TryFrom<StenType> for TypeLib {
    type Error = TranslateError;

    fn try_from(root: StenType) -> Result<Self, Self::Error> { root.translate(&mut ()) }
}

impl Display for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (name, ty) in &self.types {
            writeln!(f, "data {:16} :: {}", name, ty)?;
        }
        Ok(())
    }
}

impl TypeLib {
    pub fn id(&self) -> TypeLibId {
        let mut hasher = Hasher::new();
        for id in self.roots.iter() {
            hasher.input(*id);
        }
        hasher.finish()
    }
}
