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

#[macro_export]
macro_rules! tn {
    ($name:literal) => {
        ::amplify::confinement::Confined::try_from(
            ::amplify::ascii::AsciiString::from_ascii($name).expect("invalid type name"),
        )
        .expect("invalid type name")
    };
}

#[macro_export]
macro_rules! fields {
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert(tn!($key), Box::new($value)).is_none(), "repeated field");
            )+
            $crate::ast::Fields::try_from(m).expect("too many fields")
        }
    }
}

#[macro_export]
macro_rules! alternatives {
    { $($key:expr => $val:expr => $ty:expr),+ $(,)? } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert(tn!($key), $crate::ast::Alternative::new($val, $ty)).is_none(), "repeated union alternative");
            )+
            $crate::ast::Alternatives::try_from(m).expect("too many union alternatives")
        }
    }
}

#[macro_export]
macro_rules! variants {
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut m = ::std::collections::BTreeSet::new();
            $(
                assert!(m.insert($crate::ast::Variant::new(tn!($key), $value)), "repeated enum variant");
            )+
            $crate::ast::Variants::try_from(m).expect("too many enum variants")
        }
    }
}
