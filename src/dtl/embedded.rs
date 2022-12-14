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
use std::ops::Deref;

use amplify::confinement;
use amplify::confinement::MediumOrdMap;
use amplify::num::u24;

use crate::ast::NestedRef;
use crate::{Serialize, Ty, TyId, TypeRef};

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub enum EmbeddedTy {
    Ref(TyId),

    #[from]
    Inline(Box<Ty<EmbeddedTy>>),
}

impl Deref for EmbeddedTy {
    type Target = Ty<EmbeddedTy>;

    fn deref(&self) -> &Self::Target {
        match self {
            EmbeddedTy::Ref(_) => &Ty::UNIT,
            EmbeddedTy::Inline(ty) => ty.as_ref(),
        }
    }
}

impl TypeRef for EmbeddedTy {
    fn id(&self) -> TyId {
        match self {
            EmbeddedTy::Ref(id) => *id,
            EmbeddedTy::Inline(ty) => ty.id(),
        }
    }
}

impl NestedRef for EmbeddedTy {
    fn as_ty(&self) -> &Ty<Self> { self.deref() }

    fn into_ty(self) -> Ty<Self> {
        match self {
            EmbeddedTy::Ref(_) => Ty::UNIT,
            EmbeddedTy::Inline(ty) => *ty,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub struct TypeSystem(MediumOrdMap<TyId, Ty<EmbeddedTy>>);

impl Deref for TypeSystem {
    type Target = BTreeMap<TyId, Ty<EmbeddedTy>>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl IntoIterator for TypeSystem {
    type Item = (TyId, Ty<EmbeddedTy>);
    type IntoIter = std::collections::btree_map::IntoIter<TyId, Ty<EmbeddedTy>>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'lib> IntoIterator for &'lib TypeSystem {
    type Item = (&'lib TyId, &'lib Ty<EmbeddedTy>);
    type IntoIter = std::collections::btree_map::Iter<'lib, TyId, Ty<EmbeddedTy>>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl TypeSystem {
    pub fn try_from_iter<T: IntoIterator<Item = Ty<EmbeddedTy>>>(
        iter: T,
    ) -> Result<Self, confinement::Error> {
        let mut lib: BTreeMap<TyId, Ty<EmbeddedTy>> = empty!();
        for ty in iter {
            lib.insert(ty.id(), ty);
        }

        let lib = TypeSystem(MediumOrdMap::try_from_iter(lib)?);
        let len = lib.serialized_len();
        let max_len = u24::MAX.into_usize();
        if len > max_len {
            return Err(confinement::Error::Oversize { len, max_len }.into());
        }
        Ok(lib)
    }

    pub fn count_types(&self) -> u24 { self.0.len_u24() }
}
