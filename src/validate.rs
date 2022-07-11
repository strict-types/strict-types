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

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::{Read, Seek, SeekFrom};

use strict_encoding::StrictDecode;

use crate::{
    KeyType, PrimitiveType, StructField, StructType, TypeConstr, TypeName, TypeRef, TypeSystem,
};

pub trait Validate {
    fn validate(&self, ts: &TypeSystem, buf: &mut (impl Read + Seek)) -> bool;
}

impl Validate for TypeName {
    fn validate(&self, ts: &TypeSystem, buf: &mut (impl Read + Seek)) -> bool {
        match ts.get(self) {
            None => return false,
            Some(ty) => ty.validate(ts, buf),
        }
    }
}

impl Validate for StructType {
    fn validate(&self, ts: &TypeSystem, buf: &mut (impl Read + Seek)) -> bool {
        for field in self {
            if !field.validate(ts, buf) {
                return false;
            }
        }
        true
    }
}

impl Validate for StructField {
    fn validate(&self, ts: &TypeSystem, mut buf: &mut (impl Read + Seek)) -> bool {
        if self.optional {
            match u8::strict_decode(&mut buf) {
                Err(_) => return false,
                Ok(0) => return true,
                Ok(1) => self.ty.validate(ts, buf),
                Ok(_) => return false,
            }
        } else {
            self.ty.validate(ts, buf)
        }
    }
}

impl Validate for TypeRef {
    fn validate(&self, ts: &TypeSystem, buf: &mut (impl Read + Seek)) -> bool {
        match self {
            TypeRef::InPlace(ty) => ty.validate(ts, buf),
            TypeRef::NameRef(ty) => ty.validate(ts, buf),
        }
    }
}

impl<T> Validate for TypeConstr<T>
where T: Clone + Ord + Eq + Hash + Debug + Validate
{
    fn validate(&self, ts: &TypeSystem, mut buf: &mut (impl Read + Seek)) -> bool {
        macro_rules! pos {
            () => {
                buf.stream_position().expect("medium without stream position")
            };
        }
        macro_rules! read {
            ($pos_from:expr, $pos_to:expr) => {{
                let len = $pos_to - $pos_from;
                buf.seek(SeekFrom::Current(-(len as i64))).expect("medium without seek operation");
                let mut vec = vec![0u8; len as usize];
                buf.read_exact(&mut vec).expect("medium without seek operation");
                vec
            }};
        }

        match self {
            TypeConstr::Plain(ty) => ty.validate(ts, buf),
            TypeConstr::Array(len, ty) => {
                for _ in 0..*len {
                    if !ty.validate(ts, buf) {
                        return false;
                    }
                }
                true
            }
            TypeConstr::List(ty) => {
                let len = match u16::strict_decode(&mut buf) {
                    Err(_) => return false,
                    Ok(len) => len,
                };
                for _ in 0..len {
                    if !ty.validate(ts, buf) {
                        return false;
                    }
                }
                true
            }
            TypeConstr::Set(ty) => {
                let len = match u16::strict_decode(&mut buf) {
                    Err(_) => return false,
                    Ok(len) => len,
                };
                let mut set = BTreeSet::new();
                for _ in 0..len {
                    let pos_from = pos!();
                    if !ty.validate(ts, buf) {
                        return false;
                    }
                    // Ensure lexicographic key uniqueness and sort order
                    let pos_to = pos!();
                    let val = read!(pos_from, pos_to);
                    if let Some(last) = set.iter().last() {
                        if val <= *last {
                            return false;
                        }
                    }
                    if !set.insert(val) {
                        return false;
                    }
                }
                true
            }
            TypeConstr::Map(key, val) => {
                let len = match u16::strict_decode(&mut buf) {
                    Err(_) => return false,
                    Ok(len) => len,
                };
                let mut set = BTreeSet::new();
                for _ in 0..len {
                    let pos_from = pos!();
                    if !key.validate(ts, buf) {
                        return false;
                    }
                    // Ensure lexicographic key uniqueness and sort order
                    let pos_to = pos!();
                    let k = read!(pos_from, pos_to);
                    if let Some(last) = set.iter().last() {
                        if k <= *last {
                            return false;
                        }
                    }
                    if !set.insert(k) {
                        return false;
                    }

                    if !val.validate(ts, buf) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl Validate for KeyType {
    fn validate(&self, ts: &TypeSystem, buf: &mut (impl Read + Seek)) -> bool {
        match self {
            KeyType::Primitive(ty) => ty.validate(ts, buf),
            KeyType::Array(len, ty) => TypeConstr::Array(*len, *ty).validate(ts, buf),
            KeyType::List(ty) => TypeConstr::List(*ty).validate(ts, buf),
        }
    }
}

impl Validate for PrimitiveType {
    fn validate(&self, _: &TypeSystem, mut buf: &mut (impl Read + Seek)) -> bool {
        let len = match self {
            PrimitiveType::U8 => 1,
            PrimitiveType::U16 => 2,
            PrimitiveType::U32 => 4,
            PrimitiveType::U64 => 8,
            PrimitiveType::U128 => 16,
            PrimitiveType::U256 => 32,
            PrimitiveType::U512 => 64,
            PrimitiveType::U1024 => 128,
            PrimitiveType::I8 => 1,
            PrimitiveType::I16 => 2,
            PrimitiveType::I32 => 4,
            PrimitiveType::I64 => 8,
            PrimitiveType::I128 => 16,
            PrimitiveType::I256 => 32,
            PrimitiveType::I512 => 64,
            PrimitiveType::I1024 => 128,
            PrimitiveType::F16b => 2,
            PrimitiveType::F16 => 2,
            PrimitiveType::F32 => 4,
            PrimitiveType::F64 => 8,
            PrimitiveType::F80 => 10,
            PrimitiveType::F128 => 16,
            PrimitiveType::F256 => 32,
            PrimitiveType::F512 => 64,
            PrimitiveType::AsciiChar | PrimitiveType::UnicodeChar => {
                match u16::strict_decode(&mut buf) {
                    Err(_) => return false,
                    Ok(len) => len,
                }
            }
        };
        match buf.seek(SeekFrom::Current(len as i64)) {
            Err(_) => false,
            Ok(_) => true,
        }
    }
}
