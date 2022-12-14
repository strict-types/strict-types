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

use std::collections::BTreeSet;

use amplify::confinement::{Confined, SmallOrdMap};
use amplify::{confinement, Wrapper};

use crate::ast::{NestedRef, TranslateError, TyInner};
use crate::dtl::embedded::SystemBuilder;
use crate::dtl::{EmbeddedTy, LibTy, TypeIndex, TypeLib};
use crate::{StenType, Translate, Ty, TyId, TypeName};

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

    pub(crate) fn finalize(self) -> Result<TypeLib, confinement::Error> {
        let types = Confined::try_from(self.types.into_inner())?;
        Ok(TypeLib {
            dependencies: none!(),
            types,
        })
    }
}

impl Translate<TypeLib> for StenType {
    type Context = ();
    type Error = TranslateError;

    fn translate(self, _: &mut Self::Context) -> Result<TypeLib, Self::Error> {
        let id = self.ty.id();

        let index = self.build_index()?;

        let mut ctx = LibBuilder::with(index);
        let root = self.ty.translate(&mut ctx)?;

        let name = ctx.index.remove(&id).ok_or(TranslateError::UnknownId(id))?;
        let mut lib = ctx.finalize(bset!(id))?;
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
                LibTy::Named(name.clone())
            }
            None => LibTy::Inline(Box::new(ty)),
        })
    }
}

impl Translate<EmbeddedTy> for LibTy {
    type Context = SystemBuilder;
    type Error = ();

    fn translate(self, ctx: &mut Self::Context) -> Result<EmbeddedTy, Self::Error> {
        let lib_name =
            &ctx.dependencies.get(lib_alias).expect("internal builder inconsistency").name;
        match self {
            LibTy::Named(name) => Ok(EmbeddedTy::Name(lib_name.clone(), name)),
            LibTy::Inline(inline_ty) => {
                inline_ty.translate(ctx).map(Box::new).map(EmbeddedTy::Inline)
            }
            LibTy::Extern(ext_ty_name, ext_lib) => {
                todo!()
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
