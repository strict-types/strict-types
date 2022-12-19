// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 Ubideco Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;
use std::io;

use amplify::ascii::AsciiString;
use amplify::confinement::Confined;
use amplify::num::apfloat::ieee::{Double, Half, Oct, Quad, Single, X87DoubleExtended};
use amplify::num::apfloat::Float;
use amplify::num::{i1024, i256, i512, u1024, u24, u256, u512};
use half::bf16;

use crate::ast::{IntoIter, Step};
use crate::util::Sizing;
use crate::{Encode, StenSchema, StenType, StenWrite, StructWriter, Ty};

pub struct CheckedWriter {
    iter: IntoIter<StenType>,
    count: u16,
}

impl CheckedWriter {
    pub fn new(ty: Ty<StenType>) -> Self {
        CheckedWriter {
            iter: IntoIter::from(ty),
            count: 0,
        }
    }

    pub fn size(&self) -> u16 { self.count as u16 }
}

macro_rules! write_num {
    ($ty:ident, $name:ident, $con:ident) => {
        fn $name(&mut self, _: $ty) -> Result<(), io::Error> {
            self.iter.check_expect(&Ty::$con);
            self.count += $ty::BITS as u16 / 8;
            Ok(())
        }
    };
}

macro_rules! write_float {
    ($ty:ident, $id:ident, $name:ident, $con:ident) => {
        fn $name(&mut self, _: $ty) -> Result<(), io::Error> {
            self.iter.check_expect(&Ty::$con);
            self.count += $ty::BITS as u16 / 8;
            Ok(())
        }
    };
}

impl<'a> StenWrite for CheckedWriter {
    fn step_in(&mut self, step: Step) {
        self.iter.step_in(step).expect("invalid type construction")
    }
    fn step_out(&mut self) { self.iter.step_out().expect("invalid type construction") }

    write_num!(u8, write_u8, U8);
    write_num!(u16, write_u16, U16);
    write_num!(u24, write_u24, U24);
    write_num!(u32, write_u32, U32);
    write_num!(u64, write_u64, U64);
    write_num!(u128, write_u128, U128);
    write_num!(u256, write_u256, U256);
    write_num!(u512, write_u512, U512);
    write_num!(u1024, write_u1024, U1024);

    write_num!(i8, write_i8, I8);
    write_num!(i16, write_i16, I16);
    write_num!(i32, write_i32, I32);
    write_num!(i64, write_i64, I64);
    write_num!(i128, write_i128, I128);
    write_num!(i256, write_i256, I256);
    write_num!(i512, write_i512, I512);
    write_num!(i1024, write_i1024, I1024);

    fn write_f16b(&mut self, _: bf16) -> Result<(), io::Error> {
        self.iter.check_expect(&Ty::F16B);
        self.count += 2;
        Ok(())
    }

    write_float!(Half, f16, write_f16, F16);
    write_float!(Single, f32, write_f32, F32);
    write_float!(Double, f64, write_f64, F64);
    write_float!(X87DoubleExtended, f80, write_f80, F80);
    write_float!(Quad, f128, write_f128, F128);
    write_float!(Oct, f256, write_f256, F256);

    fn write_enum(&mut self, val: u8, ty: StenType) -> Result<(), io::Error> {
        let Some(variants) = ty.into_enum_variants() else {
            panic!("write_enum requires Ty::Enum type")
        };
        if variants.iter().find(|variant| variant.ord == val).is_none() {
            panic!("invalid enum variant {}", val);
        }
        self.count += 1;
        Ok(())
    }

    fn write_union<T: Encode>(
        &mut self,
        name: &'static str,
        ty: StenType,
        inner: &T,
    ) -> Result<(), io::Error> {
        let Some(alts) = ty.into_union_fields() else {
            panic!("write_union requires Ty::Union type")
        };
        let Some((field, alt)) = alts.iter().find(|(field, _)| field.name == Some(tn!(name))) else {
            panic!("invalid union variant {}", name);
        };
        if alt != &ty {
            panic!("wrong enum type for variant {}", name);
        }
        self.count += 1;
        inner.encode(self)?;
        Ok(())
    }

    fn write_option<T: Encode>(&mut self, val: Option<&T>) -> Result<(), io::Error> {
        self.count += 1;
        if let Some(val) = val {
            val.encode(self)?;
        }
        Ok(())
    }

    fn write_byte_array<const LEN: usize>(&mut self, _: [u8; LEN]) -> Result<(), io::Error> {
        self.iter.check_expect(&Ty::<StenType>::byte_array(LEN as u16));
        self.count += LEN as u16;
        Ok(())
    }

    fn write_bytes<const MIN: usize, const MAX: usize>(
        &mut self,
        data: impl AsRef<[u8]>,
    ) -> Result<(), io::Error> {
        let len = data.as_ref().len();
        assert!(len <= u16::MAX as usize, "writing more than U16::MAX bytes");
        self.count += len as u16;
        self.count += 2;
        Ok(())
    }

    fn write_ascii<const MIN: usize, const MAX: usize>(
        &mut self,
        s: &Confined<AsciiString, MIN, MAX>,
    ) -> Result<(), io::Error> {
        assert!(MAX <= u16::MAX as usize, "confinement size must be below u16::MAX");
        self.count += if MAX < u8::MAX as usize { 1 } else { 2 };
        self.count += s.len() as u16;
        self.iter.check_expect(&Ty::<StenType>::ascii_string(Sizing {
            min: MIN as u16,
            max: MAX as u16,
        }));
        Ok(())
    }

    fn write_string<const MIN: usize, const MAX: usize>(
        &mut self,
        s: &Confined<String, MIN, MAX>,
    ) -> Result<(), io::Error> {
        assert!(MAX <= u16::MAX as usize, "confinement size must be below u16::MAX");
        self.count += if MAX < u8::MAX as usize { 1 } else { 2 };
        self.count += s.len() as u16;
        self.iter.check_expect(&Ty::<StenType>::ascii_string(Sizing {
            min: MIN as u16,
            max: MAX as u16,
        }));
        Ok(())
    }

    fn write_list<T, const MIN: usize, const MAX: usize>(
        &mut self,
        data: &Confined<Vec<T>, MIN, MAX>,
    ) -> Result<(), io::Error>
    where
        T: Encode + StenSchema,
    {
        assert!(MAX <= u16::MAX as usize, "confinement size must be below u16::MAX");
        self.iter.check_expect(&Ty::<StenType>::list(T::sten_type(), Sizing {
            min: MIN as u16,
            max: MAX as u16,
        }));
        self.count += if MAX < u8::MAX as usize { 1 } else { 2 };
        self.step_in(Step::List);
        for item in data {
            item.encode(&mut *self)?;
        }
        self.step_out();
        Ok(())
    }

    fn write_set<T, const MIN: usize, const MAX: usize>(
        &mut self,
        data: &Confined<BTreeSet<T>, MIN, MAX>,
    ) -> Result<(), io::Error>
    where
        T: Encode + StenSchema + Hash + Ord,
    {
        assert!(MAX <= u16::MAX as usize, "confinement size must be below u16::MAX");
        self.iter.check_expect(&Ty::<StenType>::set(T::sten_type(), Sizing {
            min: MIN as u16,
            max: MAX as u16,
        }));
        self.count += if MAX < u8::MAX as usize { 1 } else { 2 };
        self.step_in(Step::Set);
        for item in data {
            item.encode(&mut *self)?;
        }
        self.step_out();
        Ok(())
    }

    fn write_map<K, V, const MIN: usize, const MAX: usize>(
        &mut self,
        data: &Confined<BTreeMap<K, V>, MIN, MAX>,
    ) -> Result<(), io::Error>
    where
        K: Encode + StenSchema + Hash + Ord,
        V: Encode + StenSchema,
    {
        assert!(MAX <= u16::MAX as usize, "confinement size must be below u16::MAX");
        self.iter.check_expect(&Ty::map(
            K::sten_type().try_into_key().expect("invalid key type"),
            V::sten_type(),
            Sizing {
                min: MIN as u16,
                max: MAX as u16,
            },
        ));
        self.count += if MAX < u8::MAX as usize { 1 } else { 2 };
        self.step_in(Step::Map);
        for (k, v) in data {
            k.encode(&mut *self)?;
            v.encode(&mut *self)?;
        }
        self.step_out();

        Ok(())
    }

    fn write_struct(self) -> StructWriter<Self> { StructWriter::start(self) }
}
