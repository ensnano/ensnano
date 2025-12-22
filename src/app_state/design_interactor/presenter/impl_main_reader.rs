use crate::app_state::design_interactor::DesignInteractor;
use crate::controller::download_staples::{
    DownloadStapleError, DownloadStapleOk, StaplesDownloader,
};
use ensnano_design::{
    grid::{GridId, HelixGridPosition},
    helices::HelixCollection as _,
    nucl::Nucl,
    strands::Strand,
};
use ensnano_utils::selection::InteractorDesignReaderExt as MainReader;
use itertools::Itertools as _;
use rust_xlsxwriter::{Color, Format, Workbook};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap},
    io::Write as _,
    path::Path,
};

impl StaplesDownloader for DesignInteractor {
    fn download_staples(&self) -> Result<DownloadStapleOk, DownloadStapleError> {
        let mut warnings = Vec::new();
        if self.presenter.current_design.scaffold_id.is_none() {
            return Err(DownloadStapleError::NoScaffoldSet);
        }
        if self.presenter.current_design.scaffold_sequence.is_none() {
            return Err(DownloadStapleError::ScaffoldSequenceNotSet);
        }

        if let Some(nucl) = self
            .presenter
            .content
            .get_staple_mismatch(self.presenter.current_design.as_ref())
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
                    .map(Strand::length)
            })
            .unwrap();
        let sequence_length = self
            .presenter
            .current_design
            .scaffold_sequence
            .as_ref()
            .map(String::len)
            .unwrap();
        if scaffold_length != sequence_length {
            warnings.push(warn_scaffold_seq_mismatch(scaffold_length, sequence_length));
        }
        Ok(DownloadStapleOk { warnings })
    }

    fn write_staples_xlsx(&self, xlsx_path: &Path) {
        // use simple_excel_writer::{row, Row, Workbook};

        let all_group_names: Vec<String> = self.presenter.get_names_of_all_groups();
        let mut group_map: HashMap<&String, usize> = HashMap::new();
        for (j, name) in all_group_names.iter().enumerate() {
            group_map.insert(name, j);
        }

        let staples = self
            .presenter
            .content
            .get_staples(&self.presenter.current_design, &self.presenter);

        let mut wb = Workbook::new(); //create(xlsx_path.to_str().unwrap());
        let mut sheets: BTreeMap<usize, Vec<Vec<&str>>> = BTreeMap::new();

        let interval_strs: Vec<_> = staples
            .iter()
            .map(|staple| {
                if let Ok(s) = serde_json::to_string(&staple.intervals.intervals) {
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
        for (i, staple) in staples.iter().enumerate() {
            let sheet = sheets
                .entry(staple.plate)
                .or_insert_with(|| vec![first_row_content.clone()]);
            let mut group_vec = all_group_names.iter().map(|_| "").collect_vec();
            for group_name in &staple.group_names {
                if let Some(index) = group_map.get(&group_name) {
                    group_vec[*index] = group_name.as_str();
                }
            }
            let mut row: Vec<&str> = vec![
                &staple.well,
                &staple.name,
                &staple.sequence,
                &interval_strs[i],
                &staple.length_str,
                &staple.domain_decomposition,
                &staple.color_str,
                &staple.group_names_string,
            ];
            row.extend(group_vec.iter());
            sheet.push(row);
        }

        let threshold_black_white_font_color = (5 * 255) / 2;

        // Add one sheet per plate
        for (sheet_id, rows) in &sheets {
            let sheet: &mut rust_xlsxwriter::Worksheet = wb
                .add_worksheet()
                .set_name(format!("Plate {sheet_id}"))
                .expect("Excel error: cannot create worksheet");

            for (i, row) in rows.iter().enumerate() {
                if i == 0 {
                    for (j, &data) in row.iter().enumerate() {
                        let bold = Format::new().set_bold();
                        sheet
                            .write_with_format(0, j as u16, data.to_owned(), &bold)
                            .expect("error write cell");
                    }
                    continue;
                }
                // write staple
                for (j, _) in row.iter().enumerate() {
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
                            let font_color = if luminance >= threshold_black_white_font_color {
                                Color::Black
                            } else {
                                Color::White
                            };
                            let format = Format::new()
                                .set_background_color(Color::RGB(color))
                                .set_font_color(font_color);
                            sheet
                                .write_with_format(i as u32, j as u16, row[j].to_owned(), &format)
                                .expect("error write cell");
                            continue;
                        }
                    }
                    sheet
                        .write(i as u32, j as u16, row[j].to_owned())
                        .expect("error write cell");
                }
            }

            sheet.autofit();
        }

        let sheet: &mut rust_xlsxwriter::Worksheet = wb
            .add_worksheet()
            .set_name("All staples".to_owned())
            .expect("Excel error: cannot create worksheet");
        let mut write_once = true;
        let mut all_i = 0;
        for rows in sheets.values() {
            for (i, row) in rows.iter().enumerate() {
                if i == 0 {
                    if write_once {
                        for (j, &data) in row.iter().enumerate() {
                            let bold = Format::new().set_bold();
                            sheet
                                .write_with_format(0, j as u16, data.to_owned(), &bold)
                                .expect("error write cell");
                        }
                        write_once = false;
                        all_i += 1;
                    }
                    continue;
                }
                // write staple
                for (j, _) in row.iter().enumerate() {
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
                                .write_with_format(all_i, j as u16, row[j].to_owned(), &format)
                                .expect("error write cell");
                            continue;
                        }
                    }
                    sheet
                        .write(all_i, j as u16, row[j].to_owned())
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

    fn write_intervals(&self, origami_path: &Path) {
        let staples = self
            .presenter
            .content
            .get_staples(&self.presenter.current_design, &self.presenter);
        let origami = Origami {
            scaffold_sequence: self
                .presenter
                .current_design
                .scaffold_sequence
                .clone()
                .unwrap_or_else(|| "NO SEQUENCE".to_owned()),
            intervals: staples
                .iter()
                .map(|s| (s.intervals.staple_id, s.intervals.intervals.clone()))
                .collect(),
        };
        let mut origamis = Origamis(BTreeMap::new());
        origamis.0.insert(1, origami);
        if let Ok(json_content) = serde_json::to_string_pretty(&origamis) {
            if let Ok(mut f) = std::fs::File::create(origami_path) {
                if let Err(e) = f.write_all(json_content.as_bytes()) {
                    log::error!("Could not write to file {e}");
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
    format!("All staples are not paired. First unpaired nucleotide: {first_unpaired}")
}

fn warn_scaffold_seq_mismatch(scaffold_length: usize, sequence_length: usize) -> String {
    format!(
        "The length of the scaffold is not equal to the length of the sequence.\n
        length of the scaffold: {scaffold_length}\n
        length of the sequence: {sequence_length}",
    )
}

impl MainReader for DesignInteractor {
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

    fn get_strand_with_id(&self, id: usize) -> Option<&Strand> {
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
            .map(Strand::domain_ends)
    }
}

#[derive(Serialize)]
struct Origamis(BTreeMap<usize, Origami>);

#[derive(Serialize)]
struct Origami {
    scaffold_sequence: String,
    intervals: BTreeMap<usize, Vec<(isize, isize)>>,
}
