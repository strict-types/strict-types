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

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::io::Write;

use amplify::confinement;
use amplify::confinement::{SmallOrdMap, SmallOrdSet};

use crate::ast::inner::TyInner;
use crate::ast::TranslateError;
use crate::{StenType, Translate, Ty, TyId, TypeName, TypeRef};

pub type TypeIndex = BTreeMap<TyId, TypeName>;

impl TypeRef for TypeName {}
impl TypeRef for TyId {}

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref)]
#[display("urn:ubideco:setl:{0}")]
pub struct LibId(blake3::Hash);

impl Ord for LibId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for LibId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

pub struct LibHasher(blake3::Hasher);

pub const SETL_ID_TAG: [u8; 32] = [0u8; 32];

impl LibHasher {
    pub fn new() -> LibHasher { LibHasher(blake3::Hasher::new_keyed(&SETL_ID_TAG)) }

    pub fn input(&mut self, id: TyId) {
        self.0.write_all(id.as_bytes()).expect("hashers do  not error")
    }

    pub fn finish(self) -> LibId { LibId(self.0.finalize()) }
}

pub struct TypeLib {
    // TODO: Require at least 1 type in a type library
    pub roots: SmallOrdMap<TypeName, TyId>,
    pub types: SmallOrdMap<TypeName, Ty<TypeName>>,
}

impl TypeLib {
    pub fn with(
        table: TypeTable,
        mut index: BTreeMap<TyId, TypeName>,
    ) -> Result<Self, TranslateError> {
        table.translate(&mut index)
    }

    pub fn id(&self) -> LibId {
        let mut hasher = LibHasher::new();
        for id in self.roots.values() {
            hasher.input(*id);
        }
        hasher.finish()
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
    pub roots: SmallOrdSet<TyId>,
    pub types: SmallOrdMap<TyId, Ty<TyId>>,
}

impl TryFrom<Ty> for TypeTable {
    type Error = confinement::Error;

    fn try_from(root: Ty) -> Result<Self, Self::Error> {
        let mut types = SmallOrdMap::new();
        let id = root.id();
        let ty = root.translate(&mut types)?;
        types.insert(id, ty)?;

        let mut roots = SmallOrdSet::new();
        roots.push(id).expect("single type");

        Ok(TypeTable { roots, types })
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
        let mut roots = SmallOrdMap::new();
        for id in self.roots {
            let name = ctx.get(&id).ok_or(TranslateError::UnknownId(id))?;
            roots.insert(name.clone(), id).expect("equal size maps");
        }
        Ok(TypeLib { roots, types })
    }
}

impl StenType {
    pub fn build_index(&self, index: &mut TypeIndex) -> Result<(), TranslateError> {
        let id = self.ty.id();
        let name =
            if !self.name.is_empty() { TypeName::from(self.name) } else { TypeName::from(id) };
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
