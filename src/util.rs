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

use std::cmp::Ordering;
use std::iter::Sum;
use std::ops::{Add, AddAssign};

use amplify::ascii::AsciiString;
use amplify::confinement;
use amplify::confinement::Confined;

#[derive(Wrapper, WrapperMut, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, Display)]
#[wrapper_mut(DerefMut)]
pub struct TypeName(Confined<AsciiString, 1, 32>);

impl TypeName {
    pub fn len_u8(&self) -> u8 { self.0.len() as u8 }
}

impl From<&'static str> for TypeName {
    fn from(s: &'static str) -> Self {
        TypeName(
            AsciiString::from_ascii(s)
                .map_err(|_| confinement::Error::Oversize { len: 0, max_len: 0 })
                .and_then(Confined::try_from)
                .expect("invalid static string for TypeName"),
        )
    }
}

// TODO: Replace with Range<usize>
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Sizing {
    pub min: usize,
    pub max: usize,
}

impl Sizing {
    pub const U8: Sizing = Sizing {
        min: 0,
        max: u8::MAX as usize,
    };

    pub const U16: Sizing = Sizing {
        min: 0,
        max: u16::MAX as usize,
    };

    pub const U8_NONEMPTY: Sizing = Sizing {
        min: 1,
        max: u8::MAX as usize,
    };
}

/// Measure of a type size in bytes
#[derive(Copy, Clone, PartialEq, Eq, Debug, Display)]
pub enum Size {
    /// Type has a fixed size known at compile time
    #[display(inner)]
    Fixed(u16),

    /// Type has variable size
    #[display("variable")]
    Variable,
}

impl PartialOrd for Size {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Size {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Size::Variable, Size::Variable) => Ordering::Equal,
            (Size::Variable, _) => Ordering::Greater,
            (_, Size::Variable) => Ordering::Less,
            (Size::Fixed(a), Size::Fixed(b)) => a.cmp(b),
        }
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Size::Fixed(a), Size::Fixed(b)) => Size::Fixed(a + b),
            _ => Size::Variable,
        }
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
}

impl Sum for Size {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Size::Fixed(0);
        for item in iter {
            acc += item;
        }
        acc
    }
}
