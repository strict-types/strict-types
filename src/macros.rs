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

#[macro_export]
macro_rules! fields {
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert($key, Box::new($value)).is_none(), "repeated field");
            )+
            $crate::ty::Fields::try_from(m).expect("too many fields")
        }
    }
}

#[macro_export]
macro_rules! alternatives {
    { $($key:expr => $val:expr => $ty:expr),+ } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert($key, $crate::ty::Alternative::new($val, $ty)).is_none(), "repeated union alternative");
            )+
            $crate::ty::Alternatives::try_from(m).expect("too many union alternatives")
        }
    }
}

#[macro_export]
macro_rules! variants {
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::BTreeSet::new();
            $(
                assert!(m.insert($crate::ty::Variant::new($key, $value)), "repeated enum variant");
            )+
            $crate::ty::Variants::try_from(m).expect("too many enum variants")
        }
    }
}
