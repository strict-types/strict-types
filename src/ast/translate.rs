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
use std::convert::Infallible;

use amplify::confinement;
use amplify::confinement::SmallOrdMap;

use crate::ast::inner::TyInner;
use crate::ast::ty::{RecursiveRef, SubTy};
use crate::ast::Fields;
use crate::{StenType, Ty, TyId, TypeName, TypeRef};

pub trait Translate<To: Sized> {
    type Context;
    type Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<To, Self::Error>;
}

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum TranslateError {
    /// a different type with name `{0}` is already present
    DuplicateName(TypeName),

    /// unknown type with id `{0}`
    UnknownId(TyId),

    #[from]
    #[display(inner)]
    Confinement(confinement::Error),
}
impl Translate<SubTy> for StenType {
    type Context = ();
    type Error = Infallible;

    fn translate(self, ctx: &mut Self::Context) -> Result<SubTy, Self::Error> {
        self.ty.translate(ctx).map(SubTy::from)
    }
}

impl Translate<TyId> for SubTy {
    type Context = SmallOrdMap<TyId, Ty<TyId>>;
    type Error = confinement::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<TyId, Self::Error> {
        let id = self.id();
        if !ctx.contains_key(&id) {
            let ty = self.into_ty().translate(ctx)?;
            ctx.insert(id, ty)?;
        }
        Ok(id)
    }
}

impl Translate<TypeName> for TyId {
    type Context = BTreeMap<TyId, TypeName>;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<TypeName, Self::Error> {
        ctx.get(&self).ok_or(TranslateError::UnknownId(self)).cloned()
    }
}

impl<Ref: TypeRef, ToRef: TypeRef> Translate<Ty<ToRef>> for Ty<Ref>
where Ref: Translate<ToRef>
{
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<Ty<ToRef>, Self::Error> {
        Ok(match self.into_inner() {
            TyInner::Primitive(prim) => TyInner::Primitive(prim),
            TyInner::Enum(vars) => TyInner::Enum(vars),
            TyInner::Union(fields) => TyInner::Union(fields.translate(ctx)?),
            TyInner::Struct(fields) => TyInner::Struct(fields.translate(ctx)?),
            TyInner::Array(ty, len) => TyInner::Array(ty.translate(ctx)?, len),
            TyInner::Unicode(sizing) => TyInner::Unicode(sizing),
            TyInner::List(ty, sizing) => TyInner::List(ty.translate(ctx)?, sizing),
            TyInner::Set(ty, sizing) => TyInner::Set(ty.translate(ctx)?, sizing),
            TyInner::Map(key, ty, sizing) => TyInner::Map(key, ty.translate(ctx)?, sizing),
        }
        .into())
    }
}

impl<Ref: TypeRef, ToRef: TypeRef, const OP: bool> Translate<Fields<ToRef, OP>> for Fields<Ref, OP>
where Ref: Translate<ToRef>
{
    type Context = <Ref as Translate<ToRef>>::Context;
    type Error = <Ref as Translate<ToRef>>::Error;

    fn translate(self, ctx: &mut Self::Context) -> Result<Fields<ToRef, OP>, Self::Error> {
        let mut fields = BTreeMap::new();
        for (name, rf) in self {
            fields.insert(name, rf.translate(ctx)?);
        }
        Ok(Fields::try_from(fields).expect("re-packing existing fields structure"))
    }
}
