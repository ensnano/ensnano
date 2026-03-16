pub enum ClipboardContent {
    Empty,
    Xovers(usize),
    Strands(usize),
    Grids(usize),
    Helices(usize),
}

impl std::fmt::Display for ClipboardContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Xovers(n) => write!(f, "{n} {}", if *n < 2 { "xover" } else { "xovers" }),
            Self::Strands(n) => write!(f, "{n} {}", if *n < 2 { "strand" } else { "strands" }),
            Self::Grids(n) => write!(f, "{n} {}", if *n < 2 { "grid" } else { "grids" }),
            Self::Helices(n) => write!(f, "{n} {}", if *n < 2 { "helix" } else { "helices" }),
        }
    }
}
