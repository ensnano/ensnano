use crate::dialog::{DialogFilter, DialogFilters};
use ensnano_utils::consts::{ENS_BACKUP_EXTENSION, ENS_EXTENSION, ORIGAMI_EXTENSION};
use std::path::Path;

pub(super) const NO_FILE_RECEIVED_LOAD: &str = "Open canceled";
pub(super) const NO_FILE_RECEIVED_SAVE: &str = "Save canceled";
pub(super) const NO_FILE_RECEIVED_OXDNA: &str = "OxDNA export canceled";
pub(super) const NO_FILE_RECEIVED_SCAFFOLD: &str = "Scaffold setting canceled";
pub(super) const NO_FILE_RECEIVED_STAPLE: &str = "Staple export canceled";

pub(super) fn failed_to_save_msg<D: std::fmt::Debug>(reason: &D) -> String {
    format!("Failed to save {reason:?}")
}

pub(super) const NO_SCAFFOLD_SET: &str = "No scaffold set. \n
                    Chose a strand and set it as the scaffold by checking the scaffold checkbox\
                    in the status bar";

pub(super) const NO_SCAFFOLD_SEQUENCE_SET: &str = "No sequence uploaded for scaffold. \n
                Upload a sequence for the scaffold by pressing the \"Load scaffold\" button";

pub(super) fn successful_staples_export_msg<P: AsRef<Path>>(file: P) -> String {
    format!(
        "Successfully wrote staples in {}",
        file.as_ref().to_string_lossy()
    )
}

pub(super) const OXDNA_EXPORT_FAILED: &str = "OxDNA export failed";
pub(super) const SAVE_DESIGN_FAILED: &str = "Could not save design";
pub(super) const SAVE_BEFORE_EXIT: &str = "Do you want to save your design before exiting?";
pub(super) const SAVE_BEFORE_LOAD: &str =
    "Do you want to save your design before loading an other one?";
pub(super) const SAVE_BEFORE_RELOAD: &str =
    "Do you want to save your changes in an other file before reloading?";
pub(super) const SAVE_BEFORE_NEW: &str =
    "Do you want to save your design before starting a new one?";

pub(super) const CHANGING_DNA_PARAMETERS_WARNING: &str =
    "Are you sure that you want to change DNA parameters?";

pub(super) const OXDNA_CONFIG_EXTENSION: &str = "oxdna";

pub(super) fn optimize_scaffold_position_msg(default_position: usize) -> String {
    format!("Optimize the scaffold position ?\n
              If you chose \"Yes\", ENSnano will position the scaffold in a way that minimizes the \
              number of anti-pattern (G^4, C^4 (A|T)^7) in the staples sequence. If you chose \"No\", \
              the scaffold sequence will begin at position {default_position}")
}

pub(super) fn invalid_sequence_file(first_invalid_char_position: usize) -> String {
    format!(
        "This text file does not contain a valid DNA sequence.\n
             First invalid char at position {first_invalid_char_position}"
    )
}

pub(super) const CADNANO_FILTERS: DialogFilters = &[DialogFilter::new("Cadnano files", &["json"])];
pub(super) const DESIGN_LOAD_FILTERS: DialogFilters = &[
    DialogFilter::new(
        "All supported files",
        &[ENS_EXTENSION, ENS_BACKUP_EXTENSION, "json", "sc"],
    ),
    DialogFilter::new("ENSnano files", &[ENS_EXTENSION, ENS_BACKUP_EXTENSION]),
    DialogFilter::new("json files", &["json"]),
    DialogFilter::new("scadnano files", &["sc"]),
];
pub(super) const DESIGN_WRITE_FILTERS: DialogFilters =
    &[DialogFilter::new("ENSnano files", &[ENS_EXTENSION])];
pub(super) const OBJECT3D_FILTERS: DialogFilters = &[
    DialogFilter::new("All supported files", &["gltf", "stl"]),
    DialogFilter::new("Stl files", &["stl"]),
    DialogFilter::new("Gltf files", &["gltf"]),
];
pub(super) const ORIGAMI_FILTERS: DialogFilters =
    &[DialogFilter::new("Origami files", &[ORIGAMI_EXTENSION])];
pub(super) const OXDNA_CONFIG_FILTERS: DialogFilters = &[DialogFilter::new(
    "Oxdna config files",
    &[OXDNA_CONFIG_EXTENSION],
)];
pub(super) const PDB_FILTERS: DialogFilters = &[DialogFilter::new("Pdb files", &["pdb"])];
pub(super) const SEQUENCE_FILTERS: DialogFilters = &[DialogFilter::new("Text files", &["txt"])];
pub(super) const STAPLES_FILTERS: DialogFilters = &[DialogFilter::new("Excel files", &["xlsx"])];
pub(super) const SVG_FILTERS: DialogFilters = &[DialogFilter::new("Svg files", &["svg"])];

pub(super) const SET_DESIGN_DIRECTORY_FIRST: &str =
    "It is not possible to import 3D objects in an unnamed design.
Please save your design first to give it a name";
