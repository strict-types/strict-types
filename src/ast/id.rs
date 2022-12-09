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

use crate::ast::Ty;

pub struct TyId();

pub struct TyHasher();

impl TyHasher {
    pub fn new() -> TyHasher { TyHasher() }

    pub fn compute_id(ty: impl TyCommit) -> TyId {
        let mut hasher = TyHasher::new();
        ty.ty_commit(&mut hasher);
        hasher.finish()
    }

    pub fn finish(self) -> TyId {}
}

pub trait TyCommit {
    fn ty_commit(&self, hasher: &mut TyHasher);
}

impl Ty {
    pub fn id(&self) -> TyId { TyHasher::compute_id(self) }
}
