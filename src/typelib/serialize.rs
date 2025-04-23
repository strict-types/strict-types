// Strict encoding schema library, implementing validation and parsing of strict encoded data
// against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2019-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
// Written in 2024-2025 by Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2025 Laboratories for Ubiquitous Deterministic Computing (UBIDECO),
//                         Institute for Distributed and Cognitive Systems (InDCS), Switzerland.
// Copyright (C) 2022-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use std::fmt::{Display, Formatter};
use std::path::Path;
use std::{fmt, io};

use amplify::confinement::U24 as U24MAX;
use baid64::DisplayBaid64;
use encoding::{StreamWriter, StrictDeserialize, StrictEncode, StrictSerialize, StrictWriter};

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
                self.strict_encode(StrictWriter::with(StreamWriter::new::<U24MAX>(file)))?;
            }
            #[cfg(feature = "armor")]
            StlFormat::Armored => {
                use armor::AsciiArmor;
                writeln!(file, "{}", self.to_ascii_armored_string())?;
            }
            StlFormat::Source => {
                writeln!(
                    file,
                    "{{-\n  Id: {id:+}\n  Name: {}\n  Version: {ver}{}\n-}}\n",
                    self.name,
                    header.unwrap_or_default()
                )?;
                writeln!(file, "{}", self.to_symbolic().expect("invalid library data"))?;
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
        writeln!(file, "{self}")?;

        Ok(())
    }
}

impl Display for SymbolicLib {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "@context")?;
        writeln!(f, "typelib {}", self.name())?;
        writeln!(f)?;
        for dep in self.dependencies() {
            writeln!(f, "import {dep}")?;
            if let Some(index) = self.extern_types().get(&dep.name) {
                for (sem_id, name) in index {
                    writeln!(f, "  use {name}#{}", sem_id.to_baid64_mnemonic())?;
                }
                writeln!(f)?;
            }
        }
        if self.dependencies().is_empty() {
            f.write_str("-- no dependencies\n")?;
        }
        writeln!(f)?;
        let width = f.width().unwrap_or(17);
        for (name, ty) in self.types() {
            if !f.alternate() {
                let mnemo = ty.sem_id_named(name).to_baid64_mnemonic();
                writeln!(f, "@mnemonic({mnemo})")?;
            }
            write!(f, "data {name:0$} : ", width)?;
            Display::fmt(ty, f)?;
            writeln!(f)?;
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
            writeln!(f, "import {dep}")?;
        }
        if self.dependencies.is_empty() {
            f.write_str("-- no dependencies")?;
        }
        writeln!(f)?;
        writeln!(f)?;
        let width = f.width().unwrap_or(17);
        for (name, ty) in &self.types {
            writeln!(f, "data {name:0$} : {ty}\n", width)?;
        }
        Ok(())
    }
}

#[cfg(feature = "armor")]
impl armor::StrictArmor for TypeLib {
    type Id = crate::TypeLibId;
    const PLATE_TITLE: &'static str = "STRICT TYPE LIB";

    fn armor_id(&self) -> Self::Id { self.id() }

    fn armor_headers(&self) -> Vec<armor::ArmorHeader> {
        use armor::ArmorHeader;

        use crate::Dependency;

        let mut headers = vec![ArmorHeader::new("Name", self.name.to_string())];
        if !self.dependencies.is_empty() {
            headers.push(ArmorHeader::with(
                "Dependencies",
                self.dependencies.iter().map(Dependency::to_string),
            ));
        }
        headers
    }
}
