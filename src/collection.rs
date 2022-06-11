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

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::io::Read;
use std::ops::Deref;

use strict_encoding::{StrictDecode, StrictEncode};

// TODO: Move mod to strict_encoding crate

pub const STRICT_COLLECTION_MAX_LEN: u16 = u16::MAX;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Display, Error)]
#[display("operation results in collection size {0} exceeding 0xFFFF, which is prohibited")]
pub struct OversizeError(usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Display, Error)]
#[display(
    "operation results in collection size {len} less than lower boundary of {min_len}, which is \
     prohibited"
)]
pub struct UndersizeError {
    pub len: u16,
    pub min_len: u16,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum RemoveError {
    #[from]
    #[display(inner)]
    Undersize(UndersizeError),

    /// index {index} is out of bounds of the collection size {len}.
    IndexOutOfBounds { index: u16, len: u16 },
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
#[derive(StrictEncode)]
pub struct StrictVec<T, const MIN_LEN: u16 = 0>(Vec<T>)
where T: StrictEncode + StrictDecode;

impl<T> Default for StrictVec<T, 0>
where T: StrictEncode + StrictDecode
{
    fn default() -> Self { Self(default!()) }
}

impl<T> StrictVec<T, 0>
where T: StrictEncode + StrictDecode
{
    pub fn new() -> Self { default!() }
}

impl<T, const MIN_LEN: u16> Deref for StrictVec<T, MIN_LEN>
where T: StrictEncode + StrictDecode
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T, const MIN_LEN: u16> StrictVec<T, MIN_LEN>
where T: StrictEncode + StrictDecode
{
    pub fn len(&self) -> u16 { self.0.len() as u16 }

    pub fn push(&mut self, item: T) -> Result<u16, OversizeError> {
        let len = self.0.len();
        if len > STRICT_COLLECTION_MAX_LEN as usize {
            return Err(OversizeError(len));
        }
        self.0.push(item);
        return Ok(len as u16);
    }

    pub fn remove(&mut self, index: u16) -> Result<T, RemoveError> {
        let len = self.len();
        if self.len() == MIN_LEN {
            return Err(UndersizeError {
                len,
                min_len: MIN_LEN,
            }
            .into());
        }
        if index > len {
            return Err(RemoveError::IndexOutOfBounds { index, len });
        }
        Ok(self.0.remove(index as usize))
    }
}

impl<T, const MIN_LEN: u16> StrictDecode for StrictVec<T, MIN_LEN>
where T: StrictEncode + StrictDecode
{
    fn strict_decode<D: Read>(mut d: D) -> Result<Self, strict_encoding::Error> {
        let len = u16::strict_decode(&mut d)?;
        if len < MIN_LEN {
            return Err(strict_encoding::Error::ValueOutOfRange(
                "array length",
                MIN_LEN as u128..STRICT_COLLECTION_MAX_LEN as u128,
                len as u128,
            ));
        }
        let mut data = Vec::<T>::with_capacity(len as usize);
        for _ in 0..len {
            data.push(T::strict_decode(&mut d)?);
        }
        Ok(Self(data))
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
#[derive(StrictEncode)]
pub struct StrictSet<T, const MIN_LEN: u16 = 0>(BTreeSet<T>)
where T: Eq + Ord + Debug + StrictEncode + StrictDecode;
// TODO: Remove `Debug` requirement after strict_encoding update

impl<T> Default for StrictSet<T, 0>
where T: Eq + Ord + Debug + StrictEncode + StrictDecode
{
    fn default() -> Self { Self(default!()) }
}

impl<T> StrictSet<T, 0>
where T: Eq + Ord + Debug + StrictEncode + StrictDecode
{
    pub fn new() -> Self { default!() }
}

impl<T, const MIN_LEN: u16> Deref for StrictSet<T, MIN_LEN>
where T: Eq + Ord + Debug + StrictEncode + StrictDecode
{
    type Target = BTreeSet<T>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T, const MIN_LEN: u16> StrictSet<T, MIN_LEN>
where T: Eq + Ord + Debug + StrictEncode + StrictDecode
{
    pub fn len(&self) -> u16 { self.0.len() as u16 }

    pub fn insert(&mut self, item: T) -> Result<u16, OversizeError> {
        let len = self.0.len();
        if len > STRICT_COLLECTION_MAX_LEN as usize {
            return Err(OversizeError(len));
        }
        self.0.insert(item);
        return Ok(len as u16);
    }

    pub fn remove<Q: ?Sized>(&mut self, item: &Q) -> Result<bool, UndersizeError>
    where
        T: Borrow<Q> + Ord,
        Q: Ord,
    {
        let len = self.len();
        if self.len() == MIN_LEN {
            return Err(UndersizeError {
                len,
                min_len: MIN_LEN,
            });
        }
        Ok(self.0.remove(item))
    }
}

impl<T, const MIN_LEN: u16> StrictDecode for StrictSet<T, MIN_LEN>
where T: Eq + Ord + Debug + StrictEncode + StrictDecode
{
    fn strict_decode<D: Read>(mut d: D) -> Result<Self, strict_encoding::Error> {
        let len = u16::strict_decode(&mut d)?;
        if len < MIN_LEN {
            return Err(strict_encoding::Error::ValueOutOfRange(
                "set length",
                MIN_LEN as u128..STRICT_COLLECTION_MAX_LEN as u128,
                len as u128,
            ));
        }
        let mut data = BTreeSet::<T>::new();
        for pos in 0..len {
            let item = T::strict_decode(&mut d)?;
            if !data.insert(item) {
                return Err(strict_encoding::Error::RepeatedValue(format!(
                    "non-unique set element at position {}",
                    pos
                )));
            }
        }
        Ok(Self(data))
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
#[derive(StrictEncode)]
pub struct StrictMap<K, V, const MIN_LEN: u16 = 0>(BTreeMap<K, V>)
where
    K: Clone + Eq + Ord + Debug + StrictEncode + StrictDecode,
    V: Clone + StrictEncode + StrictDecode;
// TODO: Remove `Debug` and `Clone` requirements after strict_encoding update

impl<K, V> Default for StrictMap<K, V, 0>
where
    K: Clone + Eq + Ord + Debug + StrictEncode + StrictDecode,
    V: Clone + StrictEncode + StrictDecode,
{
    fn default() -> Self { Self(default!()) }
}

impl<K, V> StrictMap<K, V, 0>
where
    K: Clone + Eq + Ord + Debug + StrictEncode + StrictDecode,
    V: Clone + StrictEncode + StrictDecode,
{
    pub fn new() -> Self { default!() }
}

impl<K, V, const MIN_LEN: u16> Deref for StrictMap<K, V, MIN_LEN>
where
    K: Clone + Eq + Ord + Debug + StrictEncode + StrictDecode,
    V: Clone + StrictEncode + StrictDecode,
{
    type Target = BTreeMap<K, V>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<K, V, const MIN_LEN: u16> StrictMap<K, V, MIN_LEN>
where
    K: Clone + Eq + Ord + Debug + StrictEncode + StrictDecode,
    V: Clone + StrictEncode + StrictDecode,
{
    pub fn len(&self) -> u16 { self.0.len() as u16 }
}

impl<K, V, const MIN_LEN: u16> StrictDecode for StrictMap<K, V, MIN_LEN>
where
    K: Clone + Eq + Ord + Debug + StrictEncode + StrictDecode,
    V: Clone + StrictEncode + StrictDecode,
{
    fn strict_decode<D: Read>(mut d: D) -> Result<Self, strict_encoding::Error> {
        let len = u16::strict_decode(&mut d)?;
        if len < MIN_LEN {
            return Err(strict_encoding::Error::ValueOutOfRange(
                "map length",
                MIN_LEN as u128..STRICT_COLLECTION_MAX_LEN as u128,
                len as u128,
            ));
        }
        let mut data = BTreeMap::<K, V>::new();
        for _ in 0..len {
            let key = K::strict_decode(&mut d)?;
            if data.insert(key.clone(), V::strict_decode(&mut d)?).is_some() {
                return Err(strict_encoding::Error::RepeatedValue(format!(
                    "non-unique map key {:?}",
                    key
                )));
            }
        }
        Ok(Self(data))
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
#[derive(StrictEncode)]
pub struct StrictStr<const MIN_LEN: u16 = 0>(String);

impl Default for StrictStr<0> {
    fn default() -> Self { Self(default!()) }
}

impl StrictStr<0> {
    pub fn new() -> Self { default!() }
}

impl<const MIN_LEN: u16> Deref for StrictStr<MIN_LEN> {
    type Target = String;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<const MIN_LEN: u16> StrictStr<MIN_LEN> {
    pub fn len(&self) -> u16 { self.0.len() as u16 }
}

impl<const MIN_LEN: u16> StrictDecode for StrictStr<MIN_LEN> {
    fn strict_decode<D: Read>(mut d: D) -> Result<Self, strict_encoding::Error> {
        let len = u16::strict_decode(&mut d)?;
        if len < MIN_LEN {
            return Err(strict_encoding::Error::ValueOutOfRange(
                "string length",
                MIN_LEN as u128..STRICT_COLLECTION_MAX_LEN as u128,
                len as u128,
            ));
        }
        let bytes = Vec::<u8>::strict_decode(d)?;
        Ok(Self(String::from_utf8(bytes)?))
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Display)]
#[derive(StrictEncode)]
#[display(inner)]
pub struct AsciiString<const MIN_LEN: u16 = 0, const MAX_LEN: u16 = { u16::MAX }>(String);

impl<const MAX_LEN: u16> Default for AsciiString<0, MAX_LEN> {
    fn default() -> Self { Self(default!()) }
}

impl<const MAX_LEN: u16> AsciiString<0, MAX_LEN> {
    pub fn new() -> Self { default!() }
}

impl<const MIN_LEN: u16, const MAX_LEN: u16> Deref for AsciiString<MIN_LEN, MAX_LEN> {
    type Target = String;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<const MIN_LEN: u16, const MAX_LEN: u16> AsciiString<MIN_LEN, MAX_LEN> {
    pub fn len(&self) -> u16 { self.0.len() as u16 }
}

impl<const MIN_LEN: u16, const MAX_LEN: u16> StrictDecode for AsciiString<MIN_LEN, MAX_LEN> {
    fn strict_decode<D: Read>(mut d: D) -> Result<Self, strict_encoding::Error> {
        let len = u16::strict_decode(&mut d)?;
        if len < MIN_LEN {
            return Err(strict_encoding::Error::ValueOutOfRange(
                "ASCII string length",
                MIN_LEN as u128..STRICT_COLLECTION_MAX_LEN as u128,
                len as u128,
            ));
        }
        if len > MAX_LEN {
            return Err(strict_encoding::Error::ExceedMaxItems(MAX_LEN as usize));
        }
        let mut bytes = Vec::with_capacity(len as usize);
        d.read_exact(&mut bytes)?;
        for byte in &bytes {
            if !byte.is_ascii() {
                return Err(strict_encoding::Error::ValueOutOfRange(
                    "ASCII char",
                    0x20..0x80,
                    *byte as u128,
                ));
            }
        }
        Ok(Self(unsafe { String::from_utf8_unchecked(bytes) }))
    }
}
