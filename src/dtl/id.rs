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
use std::fmt::{self, Display, Formatter};

use crate::dtl::TypeLib;

// TODO: Use real tag
pub const LIB_ID_TAG: [u8; 32] = [0u8; 32];

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

impl TypeLib {
    pub fn id(&self) -> TypeLibId {
        let mut hasher = blake3::Hasher::new_keyed(&LIB_ID_TAG);
        for (name, ty) in self.types.iter() {
            hasher.update(name.as_bytes());
            hasher.update(ty.id().as_bytes());
        }
        TypeLibId(hasher.finalize())
    }
}
