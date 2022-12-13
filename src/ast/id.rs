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

use crate::ast::ty::NestedRef;
use crate::{Encode, Ty};

#[derive(Wrapper, Copy, Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref)]
#[display("urn:ubideco:sten:{0}")]
pub struct TyId(blake3::Hash);

impl Ord for TyId {
    fn cmp(&self, other: &Self) -> Ordering { self.0.as_bytes().cmp(other.0.as_bytes()) }
}

impl PartialOrd for TyId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

pub const STEN_ID_TAG: [u8; 32] = [0u8; 32];

impl<Ref: NestedRef> Ty<Ref> {
    pub fn id(&self) -> TyId {
        let mut hasher = blake3::Hasher::new_keyed(&STEN_ID_TAG);
        self.encode(&mut hasher).expect("hasher do  not error");
        TyId(hasher.finalize())
    }
}
