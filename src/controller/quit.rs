use crate::{
    controller::{
        AutomataState, TransitionMessage, YesNo,
        messages::{
            CADNANO_FILTERS, DESIGN_LOAD_FILTERS, DESIGN_WRITE_FILTERS, NO_FILE_RECEIVED_LOAD,
            NO_FILE_RECEIVED_OXDNA, NO_FILE_RECEIVED_SAVE, OBJECT3D_FILTERS,
            OXDNA_CONFIG_EXTENSION, OXDNA_CONFIG_FILTERS, PDB_FILTERS, SAVE_BEFORE_EXIT,
            SAVE_BEFORE_LOAD, SAVE_BEFORE_NEW, SAVE_BEFORE_RELOAD, SVG_FILTERS, failed_to_save_msg,
        },
        normal_state::NormalState,
    },
    dialog::{self, DialogFilters, PathInput},
    state::MainStateView,
};
use ensnano_exports::ExportType;
use std::path::{Path, PathBuf};

pub(super) struct Quit {
    step: QuitStep,
}

enum QuitStep {
    Init {
        /// None if there is no need to save
        /// Some(Some(path)) if there is a need to save at a known path
        /// Some(None) if there is a need to save at an unknown path
        need_save: Option<Option<PathBuf>>,
    },
    Quitting,
}

#[expect(clippy::self_named_constructors)]
impl Quit {
    fn quitting() -> Self {
        Self {
            step: QuitStep::Quitting,
        }
    }

    pub(super) fn quit(need_save: Option<Option<PathBuf>>) -> Box<Self> {
        Box::new(Self {
            step: QuitStep::Init { need_save },
        })
    }
}

impl AutomataState for Quit {
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn AutomataState> {
        match self.step {
            QuitStep::Init { need_save } => init_quit(need_save),
            QuitStep::Quitting => {
                main_state.exit_control_flow();
                Box::new(NormalState)
            }
        }
    }
}

fn init_quit(need_save: Option<Option<PathBuf>>) -> Box<dyn AutomataState> {
    if let Some(path) = need_save {
        let quitting = Box::new(Quit::quitting());
        Box::new(YesNo::new(
            SAVE_BEFORE_EXIT,
            save_before_quit(path),
            quitting,
        ))
    } else {
        Box::new(Quit::quitting())
    }
}

fn save_before_quit(path: Option<PathBuf>) -> Box<dyn AutomataState> {
    let on_success = Box::new(Quit::quitting());
    let on_error = Box::new(NormalState);
    if let Some(path) = path {
        Box::new(SaveWithPath {
            path,
            on_error,
            on_success,
        })
    } else {
        Box::new(SaveAs::new(on_success, on_error))
    }
}

pub(super) struct Load {
    step: LoadStep,
    load_type: LoadType,
}

#[expect(clippy::self_named_constructors)]
impl Load {
    pub(super) fn known_path(path: PathBuf) -> Self {
        Self {
            step: LoadStep::GotPath(path),
            load_type: LoadType::Design,
        }
    }

    pub(super) fn init_reload(
        need_save: Option<Option<PathBuf>>,
        path_to_load: PathBuf,
    ) -> Box<dyn AutomataState> {
        if let Some(save_path) = need_save {
            let yes = save_before_known_path(save_path, path_to_load.clone());
            let no = Box::new(Self::known_path(path_to_load));
            Box::new(YesNo::new(SAVE_BEFORE_RELOAD, yes, no))
        } else {
            Box::new(Self::known_path(path_to_load))
        }
    }

    fn ask_path(load_type: LoadType) -> Box<Self> {
        Box::new(Self {
            step: LoadStep::AskPath { path_input: None },
            load_type,
        })
    }

    pub(super) fn load(need_save: Option<Option<PathBuf>>, load_type: LoadType) -> Box<Self> {
        Box::new(Self {
            step: LoadStep::Init { need_save },
            load_type,
        })
    }
}

enum LoadStep {
    Init { need_save: Option<Option<PathBuf>> },
    AskPath { path_input: Option<PathInput> },
    GotPath(PathBuf),
}

#[derive(Copy, Clone)]
pub(super) enum LoadType {
    Design,
    Object3D,
    SvgPath,
}

impl AutomataState for Load {
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn AutomataState> {
        match self.step {
            LoadStep::Init { need_save } => init_load(need_save, self.load_type),
            LoadStep::AskPath { path_input } => ask_path(
                path_input,
                main_state.get_current_design_directory(),
                self.load_type,
            ),
            LoadStep::GotPath(path) => match self.load_type {
                LoadType::Design => load_design(path, main_state),
                LoadType::Object3D => load_3d_object(path, main_state),
                LoadType::SvgPath => load_svg(path, main_state),
            },
        }
    }
}

fn init_load(path_to_save: Option<Option<PathBuf>>, load_type: LoadType) -> Box<dyn AutomataState> {
    if let Some(path_to_save) = path_to_save {
        let yes = save_before_load(path_to_save, load_type);
        let no = Load::ask_path(load_type);
        Box::new(YesNo::new(SAVE_BEFORE_LOAD, yes, no))
    } else {
        Load::ask_path(load_type)
    }
}

fn save_before_load(path_to_save: Option<PathBuf>, load_type: LoadType) -> Box<dyn AutomataState> {
    let on_success = Load::ask_path(load_type);
    let on_error = Box::new(NormalState);
    if let Some(path) = path_to_save {
        Box::new(SaveWithPath {
            path,
            on_error,
            on_success,
        })
    } else {
        Box::new(SaveAs::new(on_success, on_error))
    }
}

fn save_before_known_path(
    path_to_save: Option<PathBuf>,
    path_to_load: PathBuf,
) -> Box<dyn AutomataState> {
    let on_success = Box::new(Load::known_path(path_to_load));
    let on_error = Box::new(NormalState);
    if let Some(path) = path_to_save {
        Box::new(SaveWithPath {
            path,
            on_error,
            on_success,
        })
    } else {
        Box::new(SaveAs::new(on_success, on_error))
    }
}

fn ask_path<P: AsRef<Path>>(
    path_input: Option<PathInput>,
    starting_directory: Option<P>,
    load_type: LoadType,
) -> Box<dyn AutomataState> {
    if let Some(path_input) = path_input {
        if let Some(result) = path_input.get() {
            if let Some(path) = result {
                Box::new(Load {
                    step: LoadStep::GotPath(path),
                    load_type,
                })
            } else {
                TransitionMessage::new(
                    NO_FILE_RECEIVED_LOAD,
                    rfd::MessageLevel::Error,
                    Box::new(NormalState),
                )
            }
        } else {
            Box::new(Load {
                step: LoadStep::AskPath {
                    path_input: Some(path_input),
                },
                load_type,
            })
        }
    } else {
        let filters = match load_type {
            LoadType::Object3D => OBJECT3D_FILTERS,
            LoadType::Design => DESIGN_LOAD_FILTERS,
            LoadType::SvgPath => SVG_FILTERS,
        };
        let path_input = dialog::load(starting_directory, filters);
        Box::new(Load {
            step: LoadStep::AskPath {
                path_input: Some(path_input),
            },
            load_type,
        })
    }
}

fn load_design(path: PathBuf, state: &mut MainStateView) -> Box<dyn AutomataState> {
    if let Err(err) = state.load_design(path) {
        TransitionMessage::new(
            format!("Error when loading design:\n{err}"),
            rfd::MessageLevel::Error,
            Box::new(NormalState),
        )
    } else {
        Box::new(NormalState)
    }
}

fn load_3d_object(path: PathBuf, state: &mut MainStateView) -> Box<dyn AutomataState> {
    state.load_3d_object(path);
    Box::new(NormalState)
}

fn load_svg(path: PathBuf, state: &mut MainStateView) -> Box<dyn AutomataState> {
    state.load_svg(path);
    Box::new(NormalState)
}

pub(super) struct NewDesign {
    step: NewStep,
}

enum NewStep {
    Init { need_save: Option<Option<PathBuf>> },
    MakeNewDesign,
}

impl NewDesign {
    pub(super) fn init(need_save: Option<Option<PathBuf>>) -> Self {
        Self {
            step: NewStep::Init { need_save },
        }
    }

    fn make_new_design() -> Box<dyn AutomataState> {
        Box::new(Self {
            step: NewStep::MakeNewDesign,
        })
    }
}

impl AutomataState for NewDesign {
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn AutomataState> {
        match self.step {
            NewStep::Init { need_save } => {
                if let Some(path) = need_save {
                    init_new_design(path)
                } else {
                    new_design(main_state)
                }
            }
            NewStep::MakeNewDesign => new_design(main_state),
        }
    }
}

fn init_new_design(path_to_save: Option<PathBuf>) -> Box<dyn AutomataState> {
    let yes = save_before_new(path_to_save);
    let no = NewDesign::make_new_design();
    Box::new(YesNo::new(SAVE_BEFORE_NEW, yes, no))
}

fn new_design(main_state: &mut MainStateView) -> Box<dyn AutomataState> {
    main_state.new_design();
    Box::new(NormalState)
}

fn save_before_new(path_to_save: Option<PathBuf>) -> Box<dyn AutomataState> {
    let on_success = NewDesign::make_new_design();
    let on_error = Box::new(NormalState);
    if let Some(path) = path_to_save {
        Box::new(SaveWithPath {
            path,
            on_error,
            on_success,
        })
    } else {
        Box::new(SaveAs::new(on_success, on_error))
    }
}

pub(super) struct SaveAs {
    file_getter: Option<PathInput>,
    on_success: Box<dyn AutomataState>,
    on_error: Box<dyn AutomataState>,
}

impl SaveAs {
    pub(super) fn new(
        on_success: Box<dyn AutomataState>,
        on_error: Box<dyn AutomataState>,
    ) -> Self {
        Self {
            file_getter: None,
            on_success,
            on_error,
        }
    }
}

impl AutomataState for SaveAs {
    fn make_progress(
        mut self: Box<Self>,
        main_state: &mut MainStateView,
    ) -> Box<dyn AutomataState> {
        if let Some(getter) = &self.file_getter {
            if let Some(path_opt) = getter.get() {
                if let Some(path) = &path_opt {
                    if let Err(err) = main_state.save_design(path) {
                        TransitionMessage::new(
                            format!("Failed to save: {:?}", err.0),
                            rfd::MessageLevel::Error,
                            self.on_error,
                        )
                    } else {
                        TransitionMessage::new(
                            "Saved successfully".to_owned(),
                            rfd::MessageLevel::Info,
                            self.on_success,
                        )
                    }
                } else {
                    TransitionMessage::new(
                        NO_FILE_RECEIVED_SAVE,
                        rfd::MessageLevel::Error,
                        Box::new(NormalState),
                    )
                }
            } else {
                self
            }
        } else {
            let getter = dialog::get_file_to_write(
                DESIGN_WRITE_FILTERS,
                main_state.get_current_design_directory(),
                main_state.get_current_file_name(),
            );
            self.file_getter = Some(getter);
            self
        }
    }
}

pub(super) struct SaveWithPath {
    pub path: PathBuf,
    pub on_error: Box<dyn AutomataState>,
    pub on_success: Box<dyn AutomataState>,
}

impl AutomataState for SaveWithPath {
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn AutomataState> {
        if let Err(err) = main_state.save_design(&self.path) {
            TransitionMessage::new(
                format!("Failed to save: {:?}", err.0),
                rfd::MessageLevel::Error,
                self.on_error,
            )
        } else {
            TransitionMessage::new(
                "Saved successfully".to_owned(),
                rfd::MessageLevel::Info,
                self.on_success,
            )
        }
    }
}

pub(super) struct Exporting {
    file_getter: Option<PathInput>,
    on_success: Box<dyn AutomataState>,
    on_error: Box<dyn AutomataState>,
    export_type: ExportType,
}

impl Exporting {
    pub(super) fn new(
        on_success: Box<dyn AutomataState>,
        on_error: Box<dyn AutomataState>,
        export_type: ExportType,
    ) -> Self {
        Self {
            file_getter: None,
            on_success,
            on_error,
            export_type,
        }
    }
}

impl AutomataState for Exporting {
    fn make_progress(
        mut self: Box<Self>,
        main_state: &mut MainStateView,
    ) -> Box<dyn AutomataState> {
        if let Some(getter) = &self.file_getter {
            if let Some(path_opt) = getter.get() {
                if let Some(path) = &path_opt {
                    match main_state.export(path, self.export_type) {
                        Err(err) => TransitionMessage::new(
                            failed_to_save_msg(&err),
                            rfd::MessageLevel::Error,
                            self.on_error,
                        ),
                        Ok(success) => TransitionMessage::new(
                            success.message(),
                            rfd::MessageLevel::Info,
                            self.on_success,
                        ),
                    }
                } else {
                    TransitionMessage::new(
                        NO_FILE_RECEIVED_OXDNA,
                        rfd::MessageLevel::Error,
                        self.on_error,
                    )
                }
            } else {
                self
            }
        } else {
            let candidate_name = main_state.get_current_file_name().map(|p| {
                let mut ret = p.to_owned();
                ret.set_extension(export_extension(self.export_type.clone()));
                ret
            });
            let getter = dialog::get_file_to_write(
                export_filters(self.export_type.clone()),
                main_state.get_current_design_directory(),
                candidate_name,
            );
            self.file_getter = Some(getter);
            self
        }
    }
}

fn export_extension(export_type: ExportType) -> &'static str {
    match export_type {
        ExportType::Oxdna => OXDNA_CONFIG_EXTENSION,
        ExportType::Pdb => "pdb",
        ExportType::Cadnano => "json",
    }
}

fn export_filters(export_type: ExportType) -> DialogFilters {
    match export_type {
        ExportType::Oxdna => OXDNA_CONFIG_FILTERS,
        ExportType::Pdb => PDB_FILTERS,
        ExportType::Cadnano => CADNANO_FILTERS,
    }
}
