use iced::widget::image::Handle;

/// Selection modes that can be selected by buttons on the top bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum SelectionMode {
    #[default]
    Nucleotide,
    Strand,
    Helix,
    Design,
}

impl std::fmt::Display for SelectionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Design => "Design",
                Self::Nucleotide => "Nucleotide",
                Self::Strand => "Strand",
                Self::Helix => "Helix",
            }
        )
    }
}

impl SelectionMode {
    pub const ALL: [Self; 4] = [Self::Nucleotide, Self::Design, Self::Strand, Self::Helix];

    pub fn tooltip_description(&self) -> &'static str {
        // TODO: better descriptions
        match self {
            Self::Nucleotide => "Nucleotide",
            Self::Strand => "Strand",
            Self::Helix => "Helix",
            Self::Design => "Design",
        }
    }

    pub fn icon_on(&self) -> Handle {
        let bytes = match self {
            Self::Helix => include_bytes!("../../icons/icons/Helix-on32.png").to_vec(),
            Self::Nucleotide => include_bytes!("../../icons/icons/Nucleotide-on32.png").to_vec(),
            Self::Strand => include_bytes!("../../icons/icons/Strand-on32.png").to_vec(),
            Self::Design => vec![],
        };
        Handle::from_memory(bytes)
    }

    pub fn icon_off(&self) -> Handle {
        let bytes = match self {
            Self::Helix => include_bytes!("../../icons/icons/Helix-off32.png").to_vec(),
            Self::Nucleotide => include_bytes!("../../icons/icons/Nucleotide-off32.png").to_vec(),
            Self::Strand => include_bytes!("../../icons/icons/Strand-off32.png").to_vec(),
            Self::Design => vec![],
        };
        Handle::from_memory(bytes)
    }
}

/// Describe the action currently done by the user when they click left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ActionMode {
    /// User is moving the camera.
    #[default]
    Normal,
    /// User can translate objects and move the camera.
    Translate,
    /// User can rotate objects and move the camera.
    Rotate,
    /// User is creating helices with two strands starting at a given position and with a given
    /// length.
    BuildHelix { position: isize, length: usize },
    /// User can cut strands.
    Cut,
    /// User is drawing a bezier path.
    EditBezierPath,
}

impl std::fmt::Display for ActionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Normal => "Select",
                Self::Translate => "Move",
                Self::Rotate => "Rotate",
                Self::BuildHelix { .. } => "Build",
                Self::Cut => "Cut",
                Self::EditBezierPath => "Edit path",
            }
        )
    }
}

impl ActionMode {
    pub fn is_build(&self) -> bool {
        matches!(self, Self::BuildHelix { .. })
    }

    pub fn tooltip_description(&self) -> &'static str {
        // TODO: better descriptions
        match self {
            Self::Normal => "Normal",
            Self::Translate => "Translate",
            Self::Rotate => "Rotate",
            Self::BuildHelix { .. } => "BuildHelix",
            Self::Cut => "Cut",
            Self::EditBezierPath => "EditBezierPath",
        }
    }

    pub fn icon_on(&self, axis_aligned: bool) -> Handle {
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

    pub fn icon_off(&self, axis_aligned: bool) -> Handle {
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
