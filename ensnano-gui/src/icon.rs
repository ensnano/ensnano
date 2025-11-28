use ensnano_interactor::selection::{ActionMode, SelectionMode};
use iced::widget::image::Handle;

pub trait HasIcon {
    fn icon_on(&self) -> Handle;
    fn icon_off(&self) -> Handle;
}

impl HasIcon for SelectionMode {
    fn icon_on(&self) -> Handle {
        let bytes = match self {
            Self::Helix => include_bytes!("../../icons/icons/Helix-on32.png").to_vec(),
            Self::Nucleotide => include_bytes!("../../icons/icons/Nucleotide-on32.png").to_vec(),
            Self::Strand => include_bytes!("../../icons/icons/Strand-on32.png").to_vec(),
            Self::Design => vec![],
        };
        Handle::from_memory(bytes)
    }

    fn icon_off(&self) -> Handle {
        let bytes = match self {
            Self::Helix => include_bytes!("../../icons/icons/Helix-off32.png").to_vec(),
            Self::Nucleotide => include_bytes!("../../icons/icons/Nucleotide-off32.png").to_vec(),
            Self::Strand => include_bytes!("../../icons/icons/Strand-off32.png").to_vec(),
            Self::Design => vec![],
        };
        Handle::from_memory(bytes)
    }
}

pub trait HasIconDependentOnAxis {
    fn icon_on(&self, axis_aligned: bool) -> Handle;
    fn icon_off(&self, axis_aligned: bool) -> Handle;
}

impl HasIconDependentOnAxis for ActionMode {
    fn icon_on(&self, axis_aligned: bool) -> Handle {
        let bytes = match self {
            Self::BuildHelix { .. } => {
                include_bytes!("../../icons/icons/NewHelix-on32.png").to_vec()
            }
            Self::Normal => include_bytes!("../../icons/icons/Select-on32.png").to_vec(),
            Self::Translate => {
                if axis_aligned {
                    include_bytes!("../../icons/icons/Move-on32.png").to_vec()
                } else {
                    include_bytes!("../../icons/icons/Move-on-in32.png").to_vec()
                }
            }
            Self::Rotate => {
                if axis_aligned {
                    include_bytes!("../../icons/icons/Rotate-on32.png").to_vec()
                } else {
                    include_bytes!("../../icons/icons/Rotate-on-in32.png").to_vec()
                }
            }
            _ => vec![],
        };
        Handle::from_memory(bytes)
    }

    fn icon_off(&self, axis_aligned: bool) -> Handle {
        let bytes = match self {
            Self::BuildHelix { .. } => {
                include_bytes!("../../icons/icons/NewHelix-off32.png").to_vec()
            }
            Self::Normal => include_bytes!("../../icons/icons/Select-off32.png").to_vec(),
            Self::Translate => {
                if axis_aligned {
                    include_bytes!("../../icons/icons/Move-off32.png").to_vec()
                } else {
                    include_bytes!("../../icons/icons/Move-off-in32.png").to_vec()
                }
            }
            Self::Rotate => {
                if axis_aligned {
                    include_bytes!("../../icons/icons/Rotate-off32.png").to_vec()
                } else {
                    include_bytes!("../../icons/icons/Rotate-off-in32.png").to_vec()
                }
            }
            _ => vec![],
        };
        Handle::from_memory(bytes)
    }
}
