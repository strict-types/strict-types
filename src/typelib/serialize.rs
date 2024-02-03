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

use std::fmt::{Display, Formatter};
use std::path::Path;
use std::{fmt, io};

use amplify::num::u24;
use encoding::{StrictDeserialize, StrictEncode, StrictSerialize, StrictWriter};

use crate::{StlFormat, SymbolicLib, TypeLib};

impl StrictSerialize for TypeLib {}
impl StrictDeserialize for TypeLib {}

impl TypeLib {
    pub fn serialize(
        &self,
        format: StlFormat,
        dir: Option<impl AsRef<Path>>,
        ver: &'static str,
        header: Option<&'static str>,
    ) -> io::Result<()> {
        use std::fs;
        use std::io::stdout;

        let id = self.id();
        let mut file = match dir {
            None => Box::new(stdout()) as Box<dyn io::Write>,
            Some(dir) => {
                let mut filename = dir.as_ref().to_owned();
                filename.push(format!("{}@{ver}.{format}", self.name));
                Box::new(fs::File::create(filename)?) as Box<dyn io::Write>
            }
        };

        match format {
            StlFormat::Binary => {
                self.strict_encode(StrictWriter::with(u24::MAX.into_usize(), file))?;
            }
            #[cfg(feature = "base64")]
            StlFormat::Armored => {
                writeln!(file, "{self:X}")?;
            }
            StlFormat::Source => {
                writeln!(
                    file,
                    "{{-\n  Id: {id:+}\n  Name: {}\n  Version: {ver}{}\n-}}\n",
                    self.name,
                    header.unwrap_or_default()
                )?;
                writeln!(file, "{:#}", self.to_symbolic().expect("invalid library data"))?;
            }
        }

        Ok(())
    }
}

impl StrictSerialize for SymbolicLib {}
impl StrictDeserialize for SymbolicLib {}

impl SymbolicLib {
    pub fn serialize(
        &self,
        dir: Option<impl AsRef<Path>>,
        ver: &'static str,
        header: Option<&'static str>,
    ) -> io::Result<()> {
        use std::fs;
        use std::io::stdout;

        let mut file = match dir {
            None => Box::new(stdout()) as Box<dyn io::Write>,
            Some(dir) => {
                let mut filename = dir.as_ref().to_owned();
                filename.push(format!("{}@{ver}.sty", self.name()));
                Box::new(fs::File::create(filename)?) as Box<dyn io::Write>
            }
        };

        writeln!(
            file,
            "{{-\n  Name: {}\n  Version: {ver}{}\n-}}\n",
            self.name(),
            header.unwrap_or_default()
        )?;
        writeln!(file, "{self:#}")?;

        Ok(())
    }
}

impl Display for SymbolicLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typelib {}", self.name())?;
        writeln!(f)?;
        for dep in self.dependencies() {
            writeln!(f, "{dep} as {}", dep.name)?;
            if f.alternate() {
                if let Some(index) = self.extern_types().get(&dep.name) {
                    writeln!(f, "-- Imports:")?;
                    for (sem_id, name) in index {
                        writeln!(f, "-- {name} := {sem_id:0}")?;
                    }
                    writeln!(f)?;
                }
            }
        }
        if self.dependencies().is_empty() {
            f.write_str("-- no dependencies")?;
        }
        writeln!(f)?;
        writeln!(f)?;
        for (name, ty) in self.types() {
            if f.alternate() {
                writeln!(f, "-- {:0}", ty.sem_id_named(name))?;
            }
            write!(f, "data {name:0$} :: ", f.width().unwrap_or(16))?;
            Display::fmt(ty, f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typelib {} -- {}", self.name, self.id())?;
        writeln!(f)?;
        for dep in &self.dependencies {
            writeln!(f, "{dep} as {}", dep.name)?;
        }
        if self.dependencies.is_empty() {
            f.write_str("-- no dependencies")?;
        }
        writeln!(f)?;
        writeln!(f)?;
        for (name, ty) in &self.types {
            writeln!(f, "data {name:16} :: {ty}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "base64")]
impl fmt::UpperHex for TypeLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use base64::Engine;

        writeln!(f, "-----BEGIN STRICT TYPE LIB-----")?;
        writeln!(f, "Id: {:-}", self.id())?;
        writeln!(f, "Name: {}", self.name)?;
        write!(f, "Dependencies: ")?;
        if self.dependencies.is_empty() {
            writeln!(f, "~")?;
        } else {
            writeln!(f)?;
        }
        let mut iter = self.dependencies.iter();
        while let Some(dep) = iter.next() {
            write!(f, "  {:-}", dep.id)?;
            if iter.len() > 0 {
                writeln!(f, ",")?;
            } else {
                writeln!(f)?;
            }
        }
        writeln!(f)?;

        let data = self.to_strict_serialized::<0xFFFFFF>().expect("in-memory");
        let engine = base64::engine::general_purpose::STANDARD;
        let data = engine.encode(data);
        let mut data = data.as_str();
        while data.len() >= 64 {
            let (line, rest) = data.split_at(64);
            writeln!(f, "{}", line)?;
            data = rest;
        }
        writeln!(f, "{}", data)?;

        writeln!(f, "\n-----END STRICT TYPE LIB-----")?;
        Ok(())
    }
}
