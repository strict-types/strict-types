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

use std::path::Path;
use std::str::FromStr;
use std::{env, io};

use amplify::num::u24;
use encoding::{StrictEncode, StrictWriter};

use crate::TypeLib;

#[derive(Clone, Eq, PartialEq, Debug, Display, Error)]
#[display("unknown name for the file format '{0}'")]
pub struct UnknownFormat(String);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
pub enum StlFormat {
    #[display("sty")]
    Source,
    #[display("stl")]
    Binary,
    #[display("asc.stl")]
    Armored,
}

impl FromStr for StlFormat {
    type Err = UnknownFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stl" => Ok(StlFormat::Binary),
            "asc" | "asc.stl" => Ok(StlFormat::Armored),
            "sty" => Ok(StlFormat::Source),
            invalid => Err(UnknownFormat(invalid.to_owned())),
        }
    }
}

impl TypeLib {
    pub fn serialize(
        &self,
        format: StlFormat,
        dir: Option<impl AsRef<Path>>,
        lib_name: &'static str,
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
                filename.push(format!("{lib_name}@{ver}.{format}"));
                Box::new(fs::File::create(filename)?) as Box<dyn io::Write>
            }
        };

        match format {
            StlFormat::Binary => {
                self.strict_encode(StrictWriter::with(u24::MAX.into_usize(), file))?;
            }
            StlFormat::Armored => {
                writeln!(file, "{self:X}")?;
            }
            StlFormat::Source => {
                writeln!(
                    file,
                    "{{-\n  Id: {id:+}\n  Name: {lib_name}\n  Version: {ver}{}\n-}}\n",
                    header.unwrap_or_default()
                )?;
                writeln!(file, "{self}")?;
            }
        }

        Ok(())
    }
}

pub fn parse_args() -> (StlFormat, Option<String>) {
    let args: Vec<String> = env::args().collect();
    let ext = args.get(1).map(String::as_str).map(|s| s.trim_start_matches("--")).unwrap_or("sty");
    let format = StlFormat::from_str(ext).expect("unrecognized file format argument");
    let dir = match args.len() {
        1 => None,
        2 | 3 => Some(args.get(2).cloned().unwrap_or_else(|| s!("stl"))),
        _ => panic!("invalid argument count"),
    };
    (format, dir)
}
