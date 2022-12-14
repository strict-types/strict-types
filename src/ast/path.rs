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

use std::fmt::{Display, Formatter};

use amplify::confinement::SmallVec;
use amplify::Wrapper;

use crate::ast::{NestedRef, SubTy, TyInner};
use crate::{Cls, FieldName, Ty};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
pub enum Step {
    #[display(".{0}")]
    NamedField(FieldName),

    #[display(".{0}")]
    UnnamedField(u8),

    #[display("#")]
    Index,

    #[display("[]")]
    List,

    #[display("{}")]
    Set,

    #[display("->")]
    Map,
}

#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, From)]
#[wrapper(Deref)]
#[wrapper_mut(DerefMut)]
pub struct Path(SmallVec<Step>);

impl Path {
    pub fn new() -> Path { Path::default() }

    pub fn with(step: Step) -> Path { Path(small_vec!(step)) }

    pub fn iter(&self) -> std::slice::Iter<Step> { self.0.iter() }
}

impl<'path> IntoIterator for &'path Path {
    type Item = &'path Step;
    type IntoIter = std::slice::Iter<'path, Step>;

    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for step in self {
            Display::fmt(step, f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Display, Error)]
#[display("no type path {path} exists within type {ty:?}")]
pub struct PathError<'ty, Ref: NestedRef> {
    pub ty: &'ty Ty<Ref>,
    pub path: Path,
}

impl<'ty, Ref: NestedRef> PathError<'ty, Ref> {
    pub fn new(ty: &'ty Ty<Ref>, path: Path) -> Self { PathError { ty, path } }
}

#[derive(Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(doc_comments)]
pub enum CheckError {
    /// the type {0} at the current path {1} doesn't have subtypes
    NoSubtypes(Cls, Path),

    /// type {found} found when {expected} was expected at path {path}
    TypeMismatch {
        expected: Cls,
        found: Cls,
        path: Path,
    },

    /// only {checked} fields were checked out of {total} fields in total
    UncheckedFields { checked: u8, total: u8 },
}

pub struct TyIter<'ty, Ref: NestedRef> {
    ty: &'ty Ty<Ref>,
    pos: u8,
    current: Path,
}

impl<'ty, Ref: NestedRef> TyIter<'ty, Ref> {
    pub fn check(&mut self, expect: &Ty<Ref>) -> Result<(), CheckError> {
        let found = self.ty.at_path(&self.current).expect("non-existing path");
        if found != expect {
            Err(CheckError::TypeMismatch {
                found: found.cls(),
                expected: expect.cls(),
                path: self.current.clone(),
            })
        } else {
            Ok(())
        }
    }

    pub fn step_in(&mut self, step: Step) -> Result<(), CheckError> {
        self.current.push(step).expect("Ty guarantees on the structure depth are broken");
        self.ty
            .at_path(&self.current)
            .map(|_| ())
            .map_err(|_| CheckError::NoSubtypes(self.ty.cls(), self.current.clone()))
    }

    pub fn step_out(&mut self) -> Result<(), CheckError> {
        let total = self.ty.count_subtypes();
        if self.pos < total {
            return Err(CheckError::UncheckedFields {
                checked: self.pos,
                total,
            });
        }
        self.current.pop().expect("at top level of the type");
        Ok(())
    }
}

impl<'ty, Ref: NestedRef> From<&'ty Ref> for TyIter<'ty, Ref> {
    fn from(ty: &'ty Ref) -> Self {
        TyIter {
            ty,
            pos: 0,
            current: empty!(),
        }
    }
}

impl SubTy {
    pub fn iter(&self) -> TyIter<SubTy> {
        TyIter {
            ty: self,
            pos: 0,
            current: empty!(),
        }
    }
}

impl<'ty, Ref: NestedRef> IntoIterator for &'ty Ty<Ref> {
    type Item = &'ty Ref;
    type IntoIter = TyIter<'ty, Ref>;

    fn into_iter(self) -> Self::IntoIter {
        TyIter {
            ty: self,
            pos: 0,
            current: empty!(),
        }
    }
}

impl<'ty, Ref: NestedRef + 'ty> Iterator for TyIter<'ty, Ref> {
    type Item = &'ty Ref;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = match self.ty.as_inner() {
            TyInner::Union(fields) => fields.ty_at(self.pos),
            TyInner::Struct(fields) => fields.ty_at(self.pos),
            TyInner::Array(ty, _)
            | TyInner::List(ty, _)
            | TyInner::Set(ty, _)
            | TyInner::Map(_, ty, _)
                if self.pos > 0 =>
            {
                Some(ty)
            }
            _ => return None,
        };
        self.pos += 1;
        ret
    }
}

impl<Ref: NestedRef> Ty<Ref> {
    pub fn at_path(&self, path: &Path) -> Result<&Self, PathError<Ref>> {
        let mut ty = self;
        let mut path = path.clone();
        let mut path_so_far = Path::new();
        while let Some(step) = path.pop() {
            let res = match (self.as_inner(), &step) {
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
                (TyInner::Array(ty, _), Step::Index) => Some(ty),
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

    pub fn count_subtypes(&self) -> u8 {
        match self.as_inner() {
            TyInner::Primitive(_) => 0,
            TyInner::Enum(_) => 0,
            TyInner::Union(fields) => fields.len_u8(),
            TyInner::Struct(fields) => fields.len_u8(),
            TyInner::Array(_, _) => 1,
            TyInner::Unicode(_) => 0,
            TyInner::List(_, _) | TyInner::Set(_, _) | TyInner::Map(_, _, _) => 1,
        }
    }
}
