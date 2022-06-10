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

use crate::{KeyType, StrictVec};

pub enum DataStep {
    StructField(u16),
    ArrayIndex(u16),
    MapKey(KeyType),
}

pub struct DataPath(StrictVec<DataStep, 0>);
