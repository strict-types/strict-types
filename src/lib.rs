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

#[macro_use]
extern crate amplify;

mod util;
pub mod primitive;
pub mod ast;
#[macro_use]
mod macros;
mod path;

pub use ast::{KeyTy, Ty};
pub use path::{Path, PathError, Step, TyIter};
pub use util::TypeName;

/// A type which can be deterministically represented in terms of
/// strict encoding schema.
pub trait StenType {
    /// Strict encoding type name.
    const STEN_TYPE_NAME: &'static str;

    /// Returns type representing strict encoding of the data.
    fn sten_type() -> Ty;
}
