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

use std::io::Write;
use std::ops::Deref;

use crate::ast::inner::TyInner;
use crate::ast::{Alternative, Alternatives, FieldName, Fields, Ty, Variant, Variants};
use crate::util::Sizing;
use crate::KeyTy;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display("urn:ubideco:sten:{0}")]
pub struct TyId(blake3::Hash);

pub struct TyHasher(blake3::Hasher);

pub const STEN_ID_TAG: [u8; 32] = [0u8; 32];

impl TyHasher {
    pub fn new() -> TyHasher { TyHasher(blake3::Hasher::new_keyed(&STEN_ID_TAG)) }

    pub fn compute_id(ty: &impl TyCommit) -> TyId {
        let mut hasher = TyHasher::new();
        ty.ty_commit(&mut hasher);
        hasher.finish()
    }

    pub fn input(&mut self, data: impl AsRef<[u8]>) {
        self.0.write_all(data.as_ref()).expect("hashers do  not error")
    }

    pub fn finish(self) -> TyId { TyId(self.0.finalize()) }
}

pub trait TyCommit {
    fn ty_commit(&self, hasher: &mut TyHasher);
}

impl Ty {
    pub fn id(&self) -> TyId { TyHasher::compute_id(self) }
}

impl TyCommit for Ty {
    fn ty_commit(&self, hasher: &mut TyHasher) {
        let cls = self.cls() as u8;
        hasher.input([cls]);
        match self.deref() {
            TyInner::Primitive(code) => hasher.input([*code]),
            TyInner::Enum(vars) => vars.ty_commit(hasher),
            TyInner::Union(alts) => alts.ty_commit(hasher),
            TyInner::Struct(fields) => fields.ty_commit(hasher),
            TyInner::Array(ty, len) => {
                ty.ty_commit(hasher);
                hasher.input(len.to_le_bytes());
            }
            TyInner::Ascii(sizing) => sizing.ty_commit(hasher),
            TyInner::Unicode(sizing) => sizing.ty_commit(hasher),
            TyInner::List(ty, sizing) => {
                ty.ty_commit(hasher);
                sizing.ty_commit(hasher);
            }
            TyInner::Set(ty, sizing) => {
                ty.ty_commit(hasher);
                sizing.ty_commit(hasher);
            }
            TyInner::Map(key, ty, sizing) => {
                key.ty_commit(hasher);
                ty.ty_commit(hasher);
                sizing.ty_commit(hasher);
            }
        }
    }
}

impl TyCommit for KeyTy {
    fn ty_commit(&self, hasher: &mut TyHasher) {
        let cls = self.cls() as u8;
        hasher.input([cls]);
        match self.deref() {
            KeyTy::Primitive(code) => hasher.input([*code]),
            KeyTy::Enum(vars) => vars.ty_commit(hasher),
            KeyTy::Array(len) => hasher.input(len.to_le_bytes()),
            KeyTy::Ascii(sizing) => sizing.ty_commit(hasher),
            KeyTy::Unicode(sizing) => sizing.ty_commit(hasher),
            KeyTy::Bytes(sizing) => sizing.ty_commit(hasher),
        }
    }
}

impl TyCommit for Sizing {
    fn ty_commit(&self, hasher: &mut TyHasher) {
        hasher.input(self.min.to_le_bytes());
        hasher.input(self.max.to_le_bytes());
    }
}

impl TyCommit for Variants {
    fn ty_commit(&self, hasher: &mut TyHasher) {
        hasher.input([self.len_u8()]);
        for var in self {
            var.ty_commit(hasher);
        }
    }
}

impl TyCommit for Variant {
    fn ty_commit(&self, hasher: &mut TyHasher) {
        self.name.ty_commit(hasher);
        hasher.input([self.value]);
    }
}

impl TyCommit for Alternatives {
    fn ty_commit(&self, hasher: &mut TyHasher) {
        hasher.input([self.len_u8()]);
        for (name, alt) in self {
            name.ty_commit(hasher);
            alt.ty_commit(hasher);
        }
    }
}

impl TyCommit for Alternative {
    fn ty_commit(&self, hasher: &mut TyHasher) {
        hasher.input([self.id]);
        self.ty.ty_commit(hasher);
    }
}

impl TyCommit for Fields {
    fn ty_commit(&self, hasher: &mut TyHasher) {
        hasher.input([self.len_u8()]);
        for (name, ty) in self {
            name.ty_commit(hasher);
            ty.ty_commit(hasher);
        }
    }
}

impl TyCommit for FieldName {
    fn ty_commit(&self, hasher: &mut TyHasher) { hasher.input(self.as_bytes()) }
}
