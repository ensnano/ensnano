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

use super::*;
use crate::controller::{DownloadStappleError, DownloadStappleOk, StaplesDownloader};
use hex;
use rust_xlsxwriter::{Color, Format, Workbook, XlsxError};
use serde::Serialize;
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;

impl StaplesDownloader for DesignReader {
    fn download_staples(&self) -> Result<DownloadStappleOk, DownloadStappleError> {
        let mut warnings = Vec::new();
        if self.presenter.current_design.scaffold_id.is_none() {
            return Err(DownloadStappleError::NoScaffoldSet);
        }
        if self.presenter.current_design.scaffold_sequence.is_none() {
            return Err(DownloadStappleError::ScaffoldSequenceNotSet);
        }

        if let Some(nucl) = self
            .presenter
            .content
            .get_stapple_mismatch(self.presenter.current_design.as_ref())
        {
            warnings.push(warn_all_staples_not_paired(nucl));
        }

        let scaffold_length = self
            .presenter
            .current_design
            .scaffold_id
            .as_ref()
            .and_then(|s_id| {
                self.presenter
                    .current_design
                    .strands
                    .get(s_id)
                    .map(|s| s.length())
            })
            .unwrap();
        let sequence_length = self
            .presenter
            .current_design
            .scaffold_sequence
            .as_ref()
            .map(|s| s.len())
            .unwrap();
        if scaffold_length != sequence_length {
            warnings.push(warn_scaffold_seq_mismatch(scaffold_length, sequence_length));
        }
        Ok(DownloadStappleOk { warnings })
    }

    fn write_staples_xlsx(&self, xlsx_path: &PathBuf) {
        // use simple_excel_writer::{row, Row, Workbook};

        let all_group_names: Vec<String> = self.presenter.get_names_of_all_groups();
        let mut group_map: HashMap<&String, usize> = HashMap::new();
        for (j, name) in all_group_names.iter().enumerate() {
            group_map.insert(name, j);
        }

        let stapples = self
            .presenter
            .content
            .get_staples(&self.presenter.current_design, &self.presenter);

        let mut wb = Workbook::new(); //create(xlsx_path.to_str().unwrap());
        let mut sheets: BTreeMap<usize, Vec<Vec<&str>>> = BTreeMap::new();

        let interval_strs: Vec<_> = stapples
            .iter()
            .map(|stapple| {
                if let Ok(s) = serde_json::to_string(&stapple.intervals.intervals) {
                    s
                } else {
                    String::from("error getting domains")
                }
            })
            .collect();

        let mut first_row_content = vec![
            "Well Position",
            "Name",
            "Sequence",
            "Domains",
            "Length",
            "Domain Length",
            "Color",
            "Groups",
        ];
        first_row_content.extend(all_group_names.iter().map(|s| &**s));

        // Staples are scattered on the sheets according to their plate number
        for (i, stapple) in stapples.iter().enumerate() {
            let sheet = sheets
                .entry(stapple.plate)
                .or_insert_with(|| vec![first_row_content.clone()]);
            let mut group_vec = Vec::from_iter(all_group_names.iter().map(|_| ""));
            for group_name in stapple.group_names.iter() {
                if let Some(index) = group_map.get(&group_name) {
                    group_vec[*index] = &group_name.as_str();
                }
            }
            let mut row: Vec<&str> = vec![
                &stapple.well,
                &stapple.name,
                &stapple.sequence,
                &interval_strs[i],
                &stapple.length_str,
                &stapple.domain_decomposition,
                &stapple.color_str,
                &stapple.group_names_string,
            ];
            row.extend(group_vec.iter());
            sheet.push(row)
        }

        let threshold_black_white_font_color = (5 * 255) / 2;

        // Add one sheet per plate
        for (sheet_id, rows) in sheets.iter() {
            let mut sheet: &mut rust_xlsxwriter::Worksheet = wb
                .add_worksheet()
                .set_name(&format!("Plate {sheet_id}"))
                .expect("Excel error: cannot create worksheet");

            for (i, row) in rows.iter().enumerate() {
                if i == 0 {
                    for (j, data) in row.iter().enumerate() {
                        let bold = Format::new().set_bold();
                        sheet
                            .write_with_format(0, j as u16, data.to_string(), &bold)
                            .expect("error write cell");
                    }
                    continue;
                }
                // write staple
                for (j, data) in row.iter().enumerate() {
                    if j == 4 {
                        // length
                        if let Ok(length) = row[j].parse::<f64>() {
                            sheet
                                .write(i as u32, j as u16, length)
                                .expect("error write cell");
                            continue;
                        }
                    }
                    if j == 6 {
                        // color
                        if let Ok(color) = u32::from_str_radix(row[j], 16) {
                            let (r, g, b) =
                                ((color >> 16) & 0xFF, (color >> 8) & 0xFF, color & 0xFF);
                            let luminance = r + b + 3 * g;
                            // println!("{color:06X} {luminance} {r:0X} {g:0X} {b:0X} {:06X}", (r<<16) + (g<<8) + b);
                            let font_color = if luminance >= threshold_black_white_font_color {
                                Color::Black
                            } else {
                                Color::White
                            };
                            let format = Format::new()
                                .set_background_color(Color::RGB(color))
                                .set_font_color(font_color);
                            sheet
                                .write_with_format(i as u32, j as u16, row[j].to_string(), &format)
                                .expect("error write cell");
                            continue;
                        }
                    }
                    sheet
                        .write(i as u32, j as u16, row[j].to_string())
                        .expect("error write cell");
                }
            }

            sheet.autofit();
        }

        let mut sheet: &mut rust_xlsxwriter::Worksheet = wb
            .add_worksheet()
            .set_name(&format!("All staples"))
            .expect("Excel error: cannot create worksheet");
        let mut write_once = true;
        let mut all_i = 0;
        for (_, rows) in sheets.iter() {
            for (i, row) in rows.iter().enumerate() {
                if i == 0 {
                    if write_once {
                        for (j, data) in row.iter().enumerate() {
                            let bold = Format::new().set_bold();
                            sheet
                                .write_with_format(0, j as u16, data.to_string(), &bold)
                                .expect("error write cell");
                        }
                        write_once = false;
                        all_i += 1;
                    }
                    continue;
                }
                // write staple
                for (j, data) in row.iter().enumerate() {
                    if j == 4 {
                        // length
                        if let Ok(length) = row[j].parse::<f64>() {
                            sheet
                                .write(all_i, j as u16, length)
                                .expect("error write cell");
                            continue;
                        }
                    }
                    if j == 6 {
                        // color
                        if let Ok(color) = u32::from_str_radix(row[j], 16) {
                            let (r, g, b) =
                                ((color >> 16) & 0xFF, (color >> 8) & 0xFF, color & 0xFF);
                            let luminance = r + b + 3 * g;
                            let font_color = if luminance >= threshold_black_white_font_color {
                                Color::Black
                            } else {
                                Color::White
                            };
                            let format = Format::new()
                                .set_background_color(Color::RGB(color))
                                .set_font_color(font_color);
                            sheet
                                .write_with_format(all_i, j as u16, row[j].to_string(), &format)
                                .expect("error write cell");
                            continue;
                        }
                    }
                    sheet
                        .write(all_i as u32, j as u16, row[j].to_string())
                        .expect("error write cell");
                }
                all_i += 1;
            }

            sheet.autofit();
        }

        // close the excel file
        wb.save(xlsx_path).expect("save excel error!");
        // wb.close().expect("close excel error!");
    }

    fn write_intervals(&self, origami_path: &PathBuf) {
        let stapples = self
            .presenter
            .content
            .get_staples(&self.presenter.current_design, &self.presenter);
        let origami = Origami {
            scaffold_sequence: self
                .presenter
                .current_design
                .scaffold_sequence
                .clone()
                .unwrap_or("NO SEQUENCE".to_string()),
            intervals: stapples
                .iter()
                .map(|s| (s.intervals.staple_id, s.intervals.intervals.clone()))
                .collect(),
        };
        let mut origamis = Origamis(BTreeMap::new());
        origamis.0.insert(1, origami);
        if let Ok(json_content) = serde_json::to_string_pretty(&origamis) {
            if let Ok(mut f) = std::fs::File::create(origami_path) {
                if let Err(e) = f.write_all(json_content.as_bytes()) {
                    log::error!("Could not write to file {}", e);
                }
            } else {
                log::error!("Could not open file");
            }
        } else {
            log::error!("Serialization error");
        }
    }

    fn default_shift(&self) -> Option<usize> {
        self.presenter.current_design.scaffold_shift
    }
}

fn warn_all_staples_not_paired(first_unpaired: Nucl) -> String {
    format!(
        "All staptes are not paired. First unpaired nucleotide: {}",
        first_unpaired
    )
}

fn warn_scaffold_seq_mismatch(scaffold_length: usize, sequence_length: usize) -> String {
    format!(
        "The lengh of the scaffold is not equal to the length of the sequence.\n
        length of the scaffold: {}\n
        length of the sequence: {}",
        scaffold_length, sequence_length
    )
}

use ensnano_design::grid::HelixGridPosition;
use ensnano_interactor::DesignReader as MainReader;

impl MainReader for DesignReader {
    fn get_xover_id(&self, pair: &(Nucl, Nucl)) -> Option<usize> {
        self.presenter.junctions_ids.get_id(pair)
    }

    fn get_xover_with_id(&self, id: usize) -> Option<(Nucl, Nucl)> {
        self.presenter.junctions_ids.get_element(id)
    }

    fn get_grid_position_of_helix(&self, h_id: usize) -> Option<HelixGridPosition> {
        self.presenter
            .current_design
            .helices
            .get(&h_id)
            .and_then(|h| h.grid_position)
    }

    fn get_strand_with_id(&self, id: usize) -> Option<&ensnano_design::Strand> {
        self.presenter.current_design.strands.get(&id)
    }

    fn get_helix_grid(&self, h_id: usize) -> Option<GridId> {
        self.presenter
            .current_design
            .helices
            .get(&h_id)
            .and_then(|h| h.grid_position.map(|pos| pos.grid))
    }

    fn get_domain_ends(&self, s_id: usize) -> Option<Vec<Nucl>> {
        self.presenter
            .current_design
            .strands
            .get(&s_id)
            .map(|s| s.domain_ends())
    }
}

use std::collections::BTreeMap;
#[derive(Serialize)]
struct Origamis(BTreeMap<usize, Origami>);

#[derive(Serialize)]
struct Origami {
    scaffold_sequence: String,
    intervals: BTreeMap<usize, Vec<(isize, isize)>>,
}
