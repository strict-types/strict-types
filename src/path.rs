// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::confinement::{SmallVec, TinyOrdSet};

use crate::Ty;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Step {
    Field(&'static str),
    Alt(&'static str),
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
            path_so_far.push(step).expect("confinement collection guarantees");
            ty = match (self, step) {
                (Ty::Struct(fields), Step::Field(name)) => fields.get(name).map(Box::as_ref),
                (Ty::Union(variants), Step::Alt(name)) => {
                    variants.get(name).map(|alt| alt.ty.as_ref())
                }
                (Ty::Array(_, len), Step::Index(index)) if index >= *len => None,
                (Ty::Array(ty, _), Step::Index(_)) => Some(ty.as_ref()),
                (Ty::List(ty, _), Step::List) => Some(ty.as_ref()),
                (Ty::Set(ty, _), Step::Set) => Some(ty.as_ref()),
                (Ty::Map(_, ty, _), Step::Map) => Some(ty.as_ref()),
                (_, _) => None,
            }
            .ok_or_else(|| PathError::new(self, path_so_far.clone()))?;
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
