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
        $crate::Ident::try_from($name).expect("invalid type name")
    };
}

#[macro_export]
macro_rules! fields {
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut c = 0u8;
            let mut m = ::std::collections::BTreeMap::new();
            $(
                assert!(m.insert($crate::ast::Field::new(tn!($key), c), $value.into()).is_none(), "repeated field");
                #[allow(unused_assignments)] {
                    c += 1;
                }
            )+
            amplify::confinement::Confined::try_from(m).expect("too many fields").into()
        }
    }
}

#[macro_export]
macro_rules! variants {
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut m = ::std::collections::BTreeSet::new();
            $(
                assert!(m.insert($crate::ast::Field::new(tn!($key), $value)), "repeated enum variant");
            )+
            amplify::confinement::Confined::try_from(m).expect("too many enum variants").into()
        }
    }
}
