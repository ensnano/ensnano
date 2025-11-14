use super::*;
use std::fmt::{self, Write};

impl Strand {
    pub fn formatted_domains(&self) -> String {
        let mut ret = String::new();
        for d in self.domains.iter() {
            writeln!(&mut ret, "{d}").unwrap_or_default();
        }
        if self.is_cyclic {
            writeln!(&mut ret, "[cycle]").unwrap_or_default();
        }
        ret
    }

    pub fn formatted_anonymous_junctions(&self) -> String {
        let mut ret = String::new();
        for j in self.junctions.iter() {
            ret.push_str(&format!("{} ", j.anonymous_fmt()));
        }
        ret
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Insertion { nb_nucl, .. } => write!(f, "[@{nb_nucl}]"),
            Self::HelixDomain(dom) => write!(f, "{dom}"),
        }
    }
}

impl DomainJunction {
    fn anonymous_fmt(&self) -> String {
        match self {
            Self::Prime3 => String::from("[3']"),
            Self::Adjacent => String::from("[->]"),
            Self::UnidentifiedXover | Self::IdentifiedXover(_) => String::from("[x]"),
        }
    }
}

impl fmt::Debug for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for HelixInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.forward {
            write!(f, "[H{}: {} -> {}]", self.helix, self.start, self.end - 1)
        } else {
            write!(f, "[H{}: {} <- {}]", self.helix, self.start, self.end - 1)
        }
    }
}

impl fmt::Debug for HelixInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}
