use crate::{
    app_state::{
        address_pointer::AddressPointer,
        design_interactor::{DesignInteractor, presenter::Presenter},
    },
    controller::LoadDesignError,
};
use ensnano_design::{
    Design,
    cadnano::CadnanoDesign,
    codenano::CodenanoDesign,
    ensnano_version,
    id_generator::IdGenerator,
    nucl::Nucl,
    scadnano::{ScadnanoDesign, ScadnanoImportError},
};
use ensnano_utils::app_state_parameters::suggestion_parameters::SuggestionParameters;
use std::path::{Path, PathBuf};
use version_compare::Cmp;

impl DesignInteractor {
    /// Create a new data by reading a file. At the moment, the supported format are
    /// * codenano
    /// * icednano
    pub(crate) fn new_with_path(json_path: &PathBuf) -> Result<Self, LoadDesignError> {
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
            let codenano_design: Result<CodenanoDesign, _> = serde_json::from_str(&json_str);
            let scadnano_design: Result<ScadnanoDesign, _> = serde_json::from_str(&json_str);

            if let Ok(design) = scadnano_design {
                Design::from_scadnano(&design).map_err(LoadDesignError::ScadnanoImportError)
            } else if let Ok(design) = codenano_design {
                log::error!("{:?}", scadnano_design.err());
                log::info!("ok codenano");
                Ok(Design::from_codenano(&design))
            } else if let Ok(design) = CadnanoDesign::from_file(path) {
                log::info!("ok cadnano");
                Ok(Design::from_cadnano(&design))
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
