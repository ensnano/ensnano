mod cadnano;
pub mod junctions;

use crate::app_state::{
    address_pointer::AddressPointer,
    design_interactor::{DesignInteractor, presenter::Presenter},
};
use ensnano_design::{Design, Nucl, codenano, ensnano_version, scadnano};
use ensnano_interactor::app_state_parameters::suggestion_parameters::SuggestionParameters;
use crate::ensnano_utils::id_generator::IdGenerator;
use crate::{
    app_state::design_interactor::file_parsing::junctions::StrandJunction as _,
    controller::LoadDesignError,
};
use cadnano::{Cadnano, FromCadnano as _};
use scadnano::ScadnanoImportError;
use std::path::{Path, PathBuf};
use version_compare::Cmp;

impl DesignInteractor {
    /// Create a new data by reading a file. At the moment, the supported format are
    /// * codenano
    /// * icednano
    pub fn new_with_path(json_path: &PathBuf) -> Result<Self, LoadDesignError> {
        let mut xover_ids: IdGenerator<(Nucl, Nucl)> = Default::default();
        let mut design = read_file(json_path)?;
        println!("Design read");
        design.strands.remove_empty_domains();

        for s in design.strands.values_mut() {
            s.read_junctions(&mut xover_ids, true);
        }
        for s in design.strands.values_mut() {
            s.read_junctions(&mut xover_ids, false);
        }
        //let file_name = real_name(json_path);
        let suggestion_parameters = SuggestionParameters::default();
        let (presenter, design_ptr) =
            Presenter::from_new_design(design, &xover_ids, suggestion_parameters);
        let ret = Self {
            design: design_ptr,
            presenter: AddressPointer::new(presenter),
            ..Default::default()
        };
        Ok(ret)
    }
}

/// Create a design by parsing a file
#[expect(clippy::panic_in_result_fn)] // FIXME
fn read_file<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<Design, LoadDesignError> {
    let json_str =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("File not found {path:?}"));

    let design: Result<Design, _> = serde_json::from_str(&json_str);
    // First try to read icednano format
    match design {
        Ok(mut design) => {
            design.update_version();
            log::info!("ok icednano");
            let required_version = design.ensnano_version.clone();
            let current_version = ensnano_version();
            match version_compare::compare(&required_version, &current_version) {
                Ok(Cmp::Lt | Cmp::Eq) => Ok(design),
                _ => Err(LoadDesignError::IncompatibleVersion {
                    current: current_version,
                    required: required_version,
                }),
            }
        }
        Err(e) => {
            // If the file is not in icednano format, try the other supported format
            let cdn_design: Result<codenano::Design<(), ()>, _> = serde_json::from_str(&json_str);

            let scadnano_design: Result<scadnano::ScadnanoDesign, _> =
                serde_json::from_str(&json_str);

            // Try codenano format
            if let Ok(scadnano) = scadnano_design {
                Design::from_scadnano(&scadnano).map_err(LoadDesignError::ScadnanoImportError)
            } else if let Ok(design) = cdn_design {
                log::error!("{:?}", scadnano_design.err());
                log::info!("ok codenano");
                Ok(Design::from_codenano(&design))
            } else if let Ok(cadnano) = Cadnano::from_file(path) {
                log::info!("ok cadnano");
                Ok(Design::from_cadnano(cadnano))
            } else {
                log::error!("{e:?}");
                // The file is not in any supported format
                //message("Unrecognized file format".into(), rfd::MessageLevel::Error);
                Err(LoadDesignError::JsonError(e))
            }
        }
    }
}

impl From<ScadnanoImportError> for LoadDesignError {
    fn from(error: ScadnanoImportError) -> Self {
        Self::ScadnanoImportError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ensnano_design::helices::HelixCollection as _;

    fn one_helix_path() -> PathBuf {
        let mut ret = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        ret.push("tests");
        ret.push("one_helix.json");
        ret
    }

    #[test]
    fn parse_one_helix() {
        let path = one_helix_path();
        let interactor = DesignInteractor::new_with_path(&path).ok().unwrap();
        let design = interactor.design.as_ref();
        assert_eq!(design.helices.len(), 1);
    }
}
