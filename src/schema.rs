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
