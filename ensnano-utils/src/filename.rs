/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use chrono::Utc;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const DEFAULT_FILE_PREFIX: &str = "export";
pub const DEFAULT_FILE_EXTENSION: &str = "none";

pub fn derive_path_with_prefix_and_time_stamp_and_suffix(
    from_path: Option<Arc<Path>>,
    prefix: Option<&str>,
    suffix: Option<&str>,
    extension: Option<&str>,
) -> PathBuf {
    let time_stamp = Utc::now().format("%Y_%m_%d-%H_%M_%S-%6f").to_string();
    let prefix = prefix.unwrap_or(DEFAULT_FILE_PREFIX);
    let suffix = if let Some(suf) = suffix {
        format!("-{suf}")
    } else {
        "".to_string()
    };
    let extension = extension.unwrap_or(DEFAULT_FILE_EXTENSION);
    match from_path {
        Some(path) => {
            let file_stem = if let Some(stem) = path.file_stem() {
                stem.to_str().unwrap().to_owned() + &"-"
            } else {
                "".to_string()
            };
            let file_name = format!("{file_stem}{prefix}-{time_stamp}{suffix}.{extension}");
            return PathBuf::from(path.with_file_name(file_name));
        }
        None => {
            let file_name = format!("{prefix}-{time_stamp}{suffix}.{extension}");
            return PathBuf::from(file_name);
        }
    }
}
