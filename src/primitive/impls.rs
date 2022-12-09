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

use crate::{StenType, Ty};

macro_rules! st_impl {
    ($name:ident, $ty:ty) => {
        impl StenType for $ty {
            const STEN_TYPE_NAME: &'static str = stringify!($name);
            fn sten_type() -> Ty { Ty::$name }
        }
    };
}

st_impl!(U8, u8);
