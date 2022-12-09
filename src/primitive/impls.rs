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

use amplify::ascii::AsciiString;
use amplify::confinement::Confined;
use amplify::num::apfloat::ieee;
use amplify::num::{i1024, i256, i512, u1024, u24, u256, u512};
use half::bf16;

use crate::util::Sizing;
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
st_impl!(U16, u16);
st_impl!(U24, u24);
st_impl!(U32, u32);
st_impl!(U64, u64);
st_impl!(U128, u128);
st_impl!(U256, u256);
st_impl!(U512, u512);
st_impl!(U1024, u1024);

st_impl!(I8, i8);
st_impl!(I16, i16);
//st_impl!(I24, i24);
st_impl!(I32, i32);
st_impl!(I64, i64);
st_impl!(I128, i128);
st_impl!(I256, i256);
st_impl!(I512, i512);
st_impl!(I1024, i1024);

st_impl!(F16B, bf16);
st_impl!(F16, ieee::Half);
st_impl!(F32, ieee::Single);
st_impl!(F64, ieee::Double);
st_impl!(F80, ieee::X87DoubleExtended);
st_impl!(F128, ieee::Quad);
st_impl!(F256, ieee::Oct);

// We can't restrict max value for the const expression, however we will have a
// panic on `as u16` in the implementation, so the StenType for arrays longer
// than u16::MAX will not be resolvable.
impl<const LEN: usize> StenType for [u8; LEN] {
    const STEN_TYPE_NAME: &'static str = stringify!("[Byte ^ ", LEN, "]");

    fn sten_type() -> Ty { Ty::byte_array(LEN as u16) }
}

impl StenType for () {
    const STEN_TYPE_NAME: &'static str = "()";

    fn sten_type() -> Ty { Ty::UNIT }
}

impl<const MIN: usize, const MAX: usize> StenType for Confined<AsciiString, MIN, MAX> {
    const STEN_TYPE_NAME: &'static str = stringify!("[Ascii ^ ", MIN, "..=", MAX, "]");

    fn sten_type() -> Ty { Ty::ascii(Some(Sizing::new(MIN as u16, MAX as u16))) }
}