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

//! DTL stands for "Data type library".

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};

use amplify::confinement;
use amplify::confinement::SmallOrdMap;

use crate::ast::inner::TyInner;
use crate::ast::TranslateError;
use crate::{StenType, Translate, Ty, TyId, TypeName, TypeRef};

pub type TypeIndex = BTreeMap<TyId, TypeName>;

impl TypeRef for TypeName {}
impl TypeRef for TyId {}

pub struct TypeLib {
    // TODO: Require at least 1 type in a type library
    pub types: SmallOrdMap<TypeName, Ty<TypeName>>,
}

impl TypeLib {
    pub fn with(
        table: TypeTable,
        mut index: BTreeMap<TyId, TypeName>,
    ) -> Result<Self, TranslateError> {
        table.translate(&mut index)
    }
}

impl Display for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (name, ty) in &self.types {
            writeln!(f, "data {:16} :: {}", name, ty)?;
        }
        Ok(())
    }
}

pub struct TypeTable {
    // TODO: Require at least 1 type in a type library
    pub types: SmallOrdMap<TyId, Ty<TyId>>,
}

impl TryFrom<Ty> for TypeTable {
    type Error = confinement::Error;

    fn try_from(root: Ty) -> Result<Self, Self::Error> {
        let mut types = SmallOrdMap::new();
        let id = root.id();
        let ty = root.translate(&mut types)?;
        types.insert(id, ty)?;
        Ok(TypeTable { types })
    }
}

impl Translate<TypeLib> for TypeTable {
    type Context = TypeIndex;
    type Error = TranslateError;

    fn translate(self, ctx: &mut Self::Context) -> Result<TypeLib, Self::Error> {
        let mut types = SmallOrdMap::new();
        for (id, ty) in self.types {
            let ty2 = ty.translate(ctx)?;
            let name = ctx.get(&id).ok_or(TranslateError::UnknownId(id))?;
            if ty2.is_primitive() {
                continue;
            }
            if types.insert(name.clone(), ty2).expect("equal size maps").is_some() {
                return Err(TranslateError::DuplicateName(name.clone()));
            }
        }
        Ok(TypeLib { types })
    }
}

impl StenType {
    pub fn build_index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
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

    pub fn try_into_lib(self) -> Result<TypeLib, TranslateError> {
        let mut index = TypeIndex::new();
        self.build_index(&mut index)?;
        let root = self.ty.translate(&mut ()).expect("infallible");
        let table = TypeTable::try_from(root)?;
        TypeLib::with(table, index)
    }
}

impl Ty<StenType> {
    pub fn build_index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
        match self.as_inner() {
            TyInner::Union(fields) => {
                for ty in fields.values() {
                    ty.build_index(index)?;
                }
            }
            TyInner::Struct(fields) => {
                for ty in fields.values() {
                    ty.build_index(index)?;
                }
            }
            TyInner::Array(ty, _)
            | TyInner::List(ty, _)
            | TyInner::Set(ty, _)
            | TyInner::Map(_, ty, _) => ty.build_index(index)?,
            _ => {}
        }
        Ok(())
    }
}
