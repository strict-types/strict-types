// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 UBIDECO Institute
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

use std::iter::Sum;
use std::ops::{Add, AddAssign};

use amplify::confinement::{Collection, Confined};
use encoding::StrictEncode;

/// Measure of a type size in bytes.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Display)]
pub enum TypeSize {
    /// Type has a fixed size known at compile time.
    #[display(inner)]
    Fixed(u16),

    /// Type has variable size.
    #[display("variable")]
    Variable { max: usize },
}

impl TypeSize {
    pub fn new(ty: &impl StrictEncode) -> Self { todo!() }

    pub fn with<C: Collection, const MIN: usize, const MAX: usize>(
        collection: &Confined<C, MIN, MAX>,
    ) -> Self {
        todo!()
    }

    pub fn exact_size(self) -> Option<u16> {
        match self {
            TypeSize::Fixed(exact) => Some(exact),
            TypeSize::Variable { .. } => None,
        }
    }

    pub fn max_size(self) -> usize {
        match self {
            TypeSize::Fixed(exact) => exact as usize,
            TypeSize::Variable { max } => max,
        }
    }
}

impl Add for TypeSize {
    type Output = TypeSize;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (TypeSize::Fixed(a), TypeSize::Fixed(b)) => TypeSize::Fixed(a + b),
            (TypeSize::Fixed(fixed), TypeSize::Variable { max })
            | (TypeSize::Variable { max }, TypeSize::Fixed(fixed)) => TypeSize::Variable {
                max: fixed as usize + max,
            },
            (TypeSize::Variable { max: a }, TypeSize::Variable { max: b }) => {
                TypeSize::Variable { max: a + b }
            }
        }
    }
}

impl AddAssign for TypeSize {
    fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
}

impl Sum for TypeSize {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = TypeSize::Fixed(0);
        for item in iter {
            acc += item;
        }
        acc
    }
}
