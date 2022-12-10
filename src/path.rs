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

use std::ops::Deref;

use amplify::confinement::{SmallVec, TinyOrdSet};

use crate::ast::inner::TyInner;
use crate::{FieldName, Ty};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Step {
    NamedField(FieldName),
    UnnamedField(u8),
    Index(u16),
    List,
    Set,
    Map,
}

#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
pub struct Path(SmallVec<Step>);

impl Path {
    pub fn new() -> Path { Path::default() }

    pub fn with(step: Step) -> Path { Path(small_vec!(step)) }
}

#[derive(Debug, Display, Error)]
#[display("")]
pub struct PathError<'ty> {
    pub ty: &'ty Ty,
    pub path: Path,
}

impl<'ty> PathError<'ty> {
    pub fn new(ty: &'ty Ty, path: Path) -> Self { PathError { ty, path } }
}

impl Ty {
    pub fn path(&self, path: &Path) -> Result<&Ty, PathError> {
        let mut ty = self;
        let mut path = path.clone();
        let mut path_so_far = Path::new();
        while let Some(step) = path.pop() {
            let res = match (self.deref(), &step) {
                (TyInner::Struct(fields), Step::NamedField(name)) => {
                    fields.iter().find(|(f, _)| f.name.as_ref() == Some(name)).map(|(_, ty)| ty)
                }
                (TyInner::Union(variants), Step::NamedField(name)) => {
                    variants.iter().find(|(f, _)| f.name.as_ref() == Some(name)).map(|(_, ty)| ty)
                }
                (TyInner::Struct(fields), Step::UnnamedField(ord)) => {
                    fields.iter().find(|(f, _)| f.ord == *ord).map(|(_, ty)| ty)
                }
                (TyInner::Union(variants), Step::UnnamedField(ord)) => {
                    variants.iter().find(|(f, _)| f.ord == *ord).map(|(_, ty)| ty)
                }
                (TyInner::Array(_, len), Step::Index(index)) if index >= len => None,
                (TyInner::Array(ty, _), Step::Index(_)) => Some(ty),
                (TyInner::List(ty, _), Step::List) => Some(ty),
                (TyInner::Set(ty, _), Step::Set) => Some(ty),
                (TyInner::Map(_, ty, _), Step::Map) => Some(ty),
                (_, _) => None,
            };
            path_so_far.push(step).expect("confinement collection guarantees");
            ty = res.ok_or_else(|| PathError::new(self, path_so_far.clone()))?;
        }
        Ok(ty)
    }
}

pub struct TyIter {
    ty: Ty,
    fields: TinyOrdSet<&'static str>,
    current: Path,
}

impl TyIter {
    pub fn check(&mut self, ty: &Ty) {
        let real_ty = self.ty.path(&self.current).expect("non-existing path");
        assert_eq!(real_ty, ty, "type mismatch");
    }

    pub fn step_in(&mut self, step: Step) { self.current.push(step).expect("too deep structure"); }

    pub fn step_out(&mut self) {
        // TODO: Check that all fields were enumerated
        self.current.pop().expect("at top level of the type");
    }
}

impl From<Ty> for TyIter {
    fn from(ty: Ty) -> Self {
        TyIter {
            ty,
            fields: empty!(),
            current: empty!(),
        }
    }
}

// TODO: Complete type iterator
