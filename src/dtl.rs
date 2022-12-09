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

use amplify::confinement;
use amplify::confinement::SmallOrdMap;

use crate::{Translate, Ty, TyId, TypeName, TypeRef};

impl TypeRef for TypeName {}
impl TypeRef for TyId {}

pub struct TypeLib {
    // TODO: Require at least 1 type in the type library
    pub types: SmallOrdMap<TypeName, Ty<TypeName>>,
}

pub struct TypeTable {
    // TODO: Require at least 1 type in the type library
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
