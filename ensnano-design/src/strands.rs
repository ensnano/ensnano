use crate::{
    domains::{Domain, helix_interval::HelixInterval, sanitize_domains},
    helices::Helices,
    nucl::{Nucl, VirtualNucl},
    utils::is_false,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, fmt::Write as _};

/// A collection of strands, that maps strand identifier to strands.
///
/// It contains all the information about the "topology of the design".  Information about
/// cross-over or helix interval are obtained via this structure
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Strands(pub(super) BTreeMap<usize, Strand>);

impl Strands {
    pub fn get_xovers(&self) -> Vec<(Nucl, Nucl)> {
        let mut ret = vec![];
        for s in self.0.values() {
            for x in s.xovers() {
                ret.push(x);
            }
        }
        ret
    }

    pub fn get_intervals(&self) -> BTreeMap<usize, (isize, isize)> {
        let mut ret = BTreeMap::new();
        for s in self.0.values() {
            for d in &s.domains {
                if let Domain::HelixDomain(dom) = d {
                    let left = dom.start;
                    let right = dom.end - 1;
                    let interval = ret.entry(dom.helix).or_insert((left, right));
                    interval.0 = interval.0.min(left);
                    interval.1 = interval.1.max(right);
                }
            }
        }
        ret
    }

    pub fn get_strand_nucl(&self, nucl: &Nucl) -> Option<usize> {
        for (s_id, s) in &self.0 {
            if s.has_nucl(nucl) {
                return Some(*s_id);
            }
        }
        None
    }

    pub fn remove_empty_domains(&mut self) {
        for s in self.0.values_mut() {
            s.remove_empty_domains();
        }
    }

    pub fn has_at_least_on_strand_with_insertions(&self) -> bool {
        self.0.values().any(Strand::has_insertions)
    }

    /// Return the strand end status of nucl
    pub fn is_strand_end(&self, nucl: &Nucl) -> Extremity {
        for s in self.0.values() {
            if !s.is_cyclic && s.get_5prime() == Some(*nucl) {
                return Extremity::Prime5;
            } else if !s.is_cyclic && s.get_3prime() == Some(*nucl) {
                return Extremity::Prime3;
            }
        }
        Extremity::No
    }

    pub fn is_domain_end(&self, nucl: &Nucl) -> Extremity {
        for strand in self.0.values() {
            let mut prev_helix = None;
            for domain in &strand.domains {
                if domain.prime5_end() == Some(*nucl) && prev_helix != domain.half_helix() {
                    return Extremity::Prime5;
                } else if domain.prime3_end() == Some(*nucl) {
                    return Extremity::Prime3;
                } else if domain.has_nucl(nucl).is_some() {
                    return Extremity::No;
                }
                prev_helix = domain.half_helix();
            }
        }
        Extremity::No
    }

    pub fn is_true_xover_end(&self, nucl: &Nucl) -> bool {
        self.is_domain_end(nucl).to_opt().is_some() && self.is_strand_end(nucl).to_opt().is_none()
    }

    /// Return true if at least one strand goes through helix h_id
    pub fn uses_helix(&self, h_id: usize) -> bool {
        for s in self.0.values() {
            for d in &s.domains {
                if let Domain::HelixDomain(interval) = d
                    && interval.helix == h_id
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_used_bounds_for_helix(
        &self,
        h_id: usize,
        helices: &Helices,
    ) -> Option<(isize, isize)> {
        let mut min = None;
        let mut max = None;

        for s in self.0.values() {
            for d in &s.domains {
                if let Domain::HelixDomain(i) = d {
                    for nucl in [i.prime5(), i.prime3()] {
                        let (helix, pos) =
                            if let Some(nucl) = Nucl::map_to_virtual_nucl(nucl, helices) {
                                (nucl.0.helix, nucl.0.position)
                            } else {
                                (nucl.helix, nucl.position)
                            };
                        if helix == h_id {
                            min = Some(min.map_or(pos, |m: isize| m.min(pos)));
                            max = Some(max.map_or(pos, |m: isize| m.max(pos)));
                        }
                    }
                }
            }
        }

        min.zip(max)
    }

    // Collection methods
    //============================================================================================
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, id: &usize) -> Option<&Strand> {
        self.0.get(id)
    }

    pub fn get_mut(&mut self, id: &usize) -> Option<&mut Strand> {
        self.0.get_mut(id)
    }

    pub fn insert(&mut self, key: usize, strand: Strand) -> Option<Strand> {
        self.0.insert(key, strand)
    }

    pub fn remove(&mut self, key: &usize) -> Option<Strand> {
        self.0.remove(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &usize> {
        self.0.keys()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&usize, &mut Strand)> {
        self.0.iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&usize, &Strand)> {
        self.0.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &Strand> {
        self.0.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Strand> {
        self.0.values_mut()
    }

    pub fn push(&mut self, strand: Strand) {
        let id = self.0.keys().max().map_or(0, |m| m + 1);
        self.0.insert(id, strand);
    }
    //============================================================================================
}

/// A link between a 5' and a 3' domain.
///
/// For any non cyclic strand, the last domain junction must be DomainJunction::Prime3. For a cyclic
/// strand it must be the link that would be appropriate between the first and the last domain.
///
/// An Insertion is considered to be adjacent to its 5' neighbor. The link between an Insertion
/// and it's 3' neighbor is the link that would exist between it's 5' and 3' neighbor if there
/// were no insertion.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum DomainJunction {
    /// A cross-over that has not yet been given an identifier. These should exist only in
    /// transitory states.
    UnidentifiedXover,
    /// A cross-over with an identifier.
    IdentifiedXover(usize),
    /// A link between two neighboring domains
    Adjacent,
    /// Indicate that the previous domain is the end of the strand.
    Prime3,
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

/// A DNA strand. Strands are represented as sequences of `Domains`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Strand {
    /// The (ordered) vector of domains, where each domain is a
    /// directed interval of a helix.
    pub domains: Vec<Domain>,
    /// The junctions between the consecutive domains of the strand.
    /// This field is optional and will be filled automatically when absent.
    #[serde(default)]
    pub junctions: Vec<DomainJunction>,
    /// The sequence of this strand, if any. If the sequence is longer
    /// than specified by the domains, a prefix is assumed. Can be
    /// skipped in the serialization.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sequence: Option<Cow<'static, str>>,
    /// Is this sequence cyclic? Can be skipped (and defaults to
    /// `false`) in the serialization.
    #[serde(skip_serializing_if = "is_false", default, alias = "cyclic")]
    pub is_cyclic: bool,
    /// Color of this strand. If skipped, a default color will be
    /// chosen automatically.
    #[serde(default)]
    pub color: u32,
    /// A name of the strand, used for strand export. If the name is `None`, the exported strand
    /// will be given a name corresponding to the position of its 5' nucleotide
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<Cow<'static, str>>,
}

impl Strand {
    pub fn init(helix: usize, position: isize, forward: bool, color: u32) -> Self {
        let domains = vec![Domain::HelixDomain(HelixInterval {
            sequence: None,
            start: position,
            end: position + 1,
            helix,
            forward,
        })];
        let sane_domains = sanitize_domains(&domains, false);
        let junctions = read_junctions(&sane_domains, false);
        Self {
            domains: sane_domains,
            sequence: None,
            is_cyclic: false,
            junctions,
            color,
            ..Default::default()
        }
    }

    pub fn get_5prime(&self) -> Option<Nucl> {
        for domain in &self.domains {
            match domain {
                Domain::Insertion { .. } => (),
                Domain::HelixDomain(h) => {
                    let position = if h.forward { h.start } else { h.end - 1 };
                    return Some(Nucl {
                        helix: h.helix,
                        position,
                        forward: h.forward,
                    });
                }
            }
        }
        None
    }

    pub fn get_3prime(&self) -> Option<Nucl> {
        for domain in self.domains.iter().rev() {
            match domain {
                Domain::Insertion { .. } => (),
                Domain::HelixDomain(h) => {
                    let position = if h.forward { h.end - 1 } else { h.start };
                    return Some(Nucl {
                        helix: h.helix,
                        position,
                        forward: h.forward,
                    });
                }
            }
        }
        None
    }

    pub fn length(&self) -> usize {
        self.domains.iter().map(Domain::length).sum()
    }

    /// Merge all consecutive domains that are on the same helix
    pub fn merge_consecutive_domains(&mut self) {
        let mut to_merge = vec![];
        for n in 0..self.domains.len() - 1 {
            let dom1 = &self.domains[n];
            let dom2 = &self.domains[n + 1];
            if dom1.can_merge(dom2) {
                to_merge.push(n);
            }
        }
        while let Some(n) = to_merge.pop() {
            let dom2 = self.domains[n + 1].clone();
            self.domains[n].merge(&dom2);
            self.domains.remove(n + 1);
            self.junctions.remove(n);
        }
    }

    pub fn xovers(&self) -> Vec<(Nucl, Nucl)> {
        let mut ret = vec![];
        for n in 0..self.domains.len() - 1 {
            let dom1 = &self.domains[n];
            let dom2 = &self.domains[n + 1];
            match (dom1, dom2) {
                (Domain::HelixDomain(int1), Domain::HelixDomain(int2))
                    if int1.helix != int2.helix =>
                {
                    ret.push((dom1.prime3_end().unwrap(), dom2.prime5_end().unwrap()));
                }
                _ => (),
            }
        }
        if self.is_cyclic && self.domains.len() > 1 {
            let dom1 = &self.domains[self.domains.len() - 1];
            let dom2 = &self.domains[0];
            match (dom1, dom2) {
                (Domain::HelixDomain(int1), Domain::HelixDomain(int2))
                    if int1.helix != int2.helix =>
                {
                    ret.push((dom1.prime3_end().unwrap(), dom2.prime5_end().unwrap()));
                }
                _ => (),
            }
        }
        ret
    }

    pub fn intersect_domains(&self, domains: &[Domain]) -> bool {
        for d in &self.domains {
            for other in domains {
                if d.intersect(other) {
                    return true;
                }
            }
        }
        false
    }

    pub fn has_nucl(&self, nucl: &Nucl) -> bool {
        for d in &self.domains {
            if d.has_nucl(nucl).is_some() {
                return true;
            }
        }
        false
    }

    pub fn find_nucl(&self, nucl: &Nucl) -> Option<usize> {
        let mut ret = 0;
        for d in &self.domains {
            if let Some(n) = d.has_nucl(nucl) {
                return Some(ret + n);
            }
            ret += d.length();
        }
        None
    }

    pub fn find_virtual_nucl(&self, nucl: &VirtualNucl, helices: &Helices) -> Option<usize> {
        let mut ret = 0;
        for d in &self.domains {
            if let Some(n) = d.has_virtual_nucl(nucl, helices) {
                return Some(ret + n);
            }
            ret += d.length();
        }
        None
    }

    pub fn get_insertions(&self) -> Vec<Nucl> {
        let mut last_nucl = None;
        let mut ret = Vec::with_capacity(self.domains.len());
        for d in &self.domains {
            match d {
                Domain::Insertion { nb_nucl, .. } if *nb_nucl > 0 => {
                    if let Some(nucl) = last_nucl {
                        ret.push(nucl);
                    } else if let Some(nucl) = self.get_5prime() {
                        ret.push(nucl);
                    }
                }
                Domain::Insertion { .. } => (),
                Domain::HelixDomain(_) => {
                    last_nucl = d.prime3_end();
                }
            }
        }
        ret
    }

    pub(super) fn remove_empty_domains(&mut self) {
        self.domains.retain(|d| {
            if d.length() > 0 {
                true
            } else {
                println!("Warning, removing empty domain {d:?}");
                false
            }
        });
    }

    pub fn get_nth_nucl(&self, n: usize) -> Option<Nucl> {
        let mut seen = 0;
        for d in &self.domains {
            if seen + d.length() > n {
                return if let Domain::HelixDomain(d) = d {
                    let position = d.iter().nth(n - seen);
                    position.map(|position| Nucl {
                        position,
                        helix: d.helix,
                        forward: d.forward,
                    })
                } else {
                    None
                };
            }
            seen += d.length();
        }
        None
    }

    pub fn insertion_points(&self) -> Vec<(Option<Nucl>, Option<Nucl>)> {
        let mut ret = Vec::new();
        let mut prev_prime3 = if self.is_cyclic {
            self.domains.last().and_then(Domain::prime3_end)
        } else {
            None
        };
        for (d1, d2) in self.domains.iter().zip(self.domains.iter().skip(1)) {
            if let Domain::Insertion { .. } = d1 {
                ret.push((prev_prime3, d2.prime5_end()));
            } else {
                prev_prime3 = d1.prime3_end();
            }
        }
        if let Some(Domain::Insertion { .. }) = self.domains.last() {
            if self.is_cyclic {
                ret.push((
                    prev_prime3,
                    self.domains.first().and_then(Domain::prime5_end),
                ));
            } else {
                ret.push((prev_prime3, None));
            }
        }
        ret
    }

    pub fn has_insertions(&self) -> bool {
        self.domains
            .iter()
            .any(|d| matches!(d, Domain::Insertion { .. }))
    }

    pub fn add_insertion_at_nucl(&mut self, nucl: &Nucl, insertion_size: usize) {
        if let Some((d_id, n)) = self.locate_nucl(nucl) {
            self.add_insertion_at_dom_position(d_id, n, insertion_size);
        } else {
            log::warn!("Could not add insertion");
        }
    }

    fn locate_nucl(&self, nucl: &Nucl) -> Option<(usize, usize)> {
        for (d_id, d) in self.domains.iter().enumerate() {
            if let Some(n) = d.has_nucl(nucl) {
                return Some((d_id, n));
            }
        }
        None
    }

    pub fn locate_virtual_nucl(
        &self,
        nucl: &VirtualNucl,
        helices: &Helices,
    ) -> Option<PositionOnStrand> {
        let mut len = 0;
        for (mut d_id, d) in self.domains.iter().enumerate() {
            if let Some(n) = d.has_virtual_nucl(nucl, helices) {
                if self.is_cyclic
                    && d_id == self.domains.len() - 1
                    && d.prime3_end().map(|n| n.prime3()) == self.domains[0].prime5_end()
                {
                    d_id = 0;
                }
                return Some(PositionOnStrand {
                    domain_id: d_id,
                    pos_on_domain: n,
                    pos_on_strand: n + len,
                });
            }
            len += d.length();
        }
        None
    }

    fn add_insertion_at_dom_position(&mut self, d_id: usize, pos: usize, insertion_size: usize) {
        if let Some((prime5, prime3)) = self.domains[d_id].split(pos) {
            self.domains[d_id] = prime3;
            if pos == 0 {
                self.domains
                    .insert(d_id, Domain::new_prime5_insertion(insertion_size));
            } else {
                self.domains
                    .insert(d_id, Domain::new_insertion(insertion_size));
                self.domains.insert(d_id, prime5);
            }
        } else {
            log::warn!("Could not split");
        }
    }

    pub fn set_name<S: Into<Cow<'static, str>>>(&mut self, name: S) {
        self.name = Some(name.into());
    }

    pub fn domain_ends(&self) -> Vec<Nucl> {
        self.domains
            .iter()
            .filter_map(|d| Some([d.prime5_end()?, d.prime3_end()?]))
            .flatten()
            .collect()
    }

    pub fn domain_lengths(&self) -> Vec<usize> {
        let mut previous_domain: Option<Domain> = None;
        let mut lengths: Vec<usize> = vec![];
        for d in &self.domains {
            if previous_domain.filter(|prev| prev.is_neighbor(d)).is_some() {
                *lengths.last_mut().unwrap() += d.length();
            } else {
                lengths.push(d.length());
            }
            previous_domain = Some(d.clone());
        }
        if lengths.len() > 1
            && self
                .domains
                .first()
                .zip(self.domains.last())
                .filter(|(d1, d2)| d1.is_neighbor(d2))
                .is_some()
        {
            lengths[0] += lengths.pop().unwrap();
        }
        lengths
    }

    pub fn formatted_domains(&self) -> String {
        let mut ret = String::new();
        for d in &self.domains {
            writeln!(&mut ret, "{d}").unwrap_or_default();
        }
        if self.is_cyclic {
            writeln!(&mut ret, "[cycle]").unwrap_or_default();
        }
        ret
    }

    pub fn formatted_anonymous_junctions(&self) -> String {
        let mut ret = String::new();
        for j in &self.junctions {
            let _ = write!(ret, "{} ", j.anonymous_fmt());
        }
        ret
    }
}

/// Add the correct junction between current and next to junctions.
/// Assumes and preserve the following invariant
/// Invariant read_junctions::PrevDomain: One of the following is true
/// * the strand is not cyclic
/// * the strand is cyclic and its first domain is NOT and insertion.
/// * previous domain points to some Domain::HelixDomain.
///
/// Moreover at the end of each iteration of the loop, previous_domain points to some
/// Domain::HelixDomain. The loop is responsible for preserving the invariant. The invariant is
/// true at initialization if SaneDomains is true.
fn add_junction<'b, 'a: 'b>(
    junctions: &'b mut Vec<DomainJunction>,
    current: &'a Domain,
    next: &'a Domain,
    previous_domain: &'b mut &'a Domain,
    cyclic: bool,
    i: usize,
) {
    match next {
        Domain::Insertion { .. } => {
            junctions.push(DomainJunction::Adjacent);
            if let Domain::HelixDomain(_) = current {
                *previous_domain = current;
            } else {
                panic!("Invariant violated: SaneDomains");
            }
        }
        Domain::HelixDomain(prime3) => {
            match current {
                Domain::Insertion { .. } => {
                    if i == 0 && !cyclic {
                        // The first domain IS an insertion
                        junctions.push(DomainJunction::Adjacent);
                    } else {
                        // previous domain MUST point to some Domain::HelixDomain.
                        if let Domain::HelixDomain(prime5) = *previous_domain {
                            junctions.push(junction(prime5, prime3));
                        } else if i == 0 {
                            panic!("Invariant violated: SaneDomains");
                        } else {
                            panic!("Invariant violated: read_junctions::PrevDomain");
                        }
                    }
                }
                Domain::HelixDomain(prime5) => {
                    junctions.push(junction(prime5, prime3));
                    *previous_domain = current;
                }
            }
        }
    }
}

/// Infer junctions from a succession of domains.
pub fn read_junctions(domains: &[Domain], cyclic: bool) -> Vec<DomainJunction> {
    if domains.is_empty() {
        return vec![];
    }

    let mut ret = Vec::with_capacity(domains.len());
    let mut previous_domain = &domains[domains.len() - 1];

    for i in 0..(domains.len() - 1) {
        let current = &domains[i];
        let next = &domains[i + 1];
        add_junction(&mut ret, current, next, &mut previous_domain, cyclic, i);
    }

    if cyclic {
        let last = &domains[domains.len() - 1];
        let first = &domains[0];
        add_junction(
            &mut ret,
            last,
            first,
            &mut previous_domain,
            cyclic,
            domains.len() - 1,
        );
    } else {
        ret.push(DomainJunction::Prime3);
    }

    ret
}

/// Return the appropriate junction between two HelixInterval
fn junction(prime5: &HelixInterval, prime3: &HelixInterval) -> DomainJunction {
    let prime5_nucl = prime5.prime3();
    let prime3_nucl = prime3.prime5();

    if prime3_nucl == prime5_nucl.prime3() {
        DomainJunction::Adjacent
    } else {
        DomainJunction::UnidentifiedXover
    }
}

/// The return type for methods that ask if a nucleotide is the end of a domain/strand/xover
#[derive(Debug, Clone, Copy)]
pub enum Extremity {
    No,
    Prime3,
    Prime5,
}

impl Extremity {
    pub fn is_3prime(&self) -> bool {
        matches!(self, Self::Prime3)
    }

    pub fn is_5prime(&self) -> bool {
        matches!(self, Self::Prime5)
    }

    pub fn is_end(&self) -> bool {
        !matches!(self, Self::No)
    }

    // TODO: remove and use Extremity directly
    pub fn to_opt(self) -> Option<bool> {
        match self {
            Self::No => None,
            Self::Prime3 => Some(true),
            Self::Prime5 => Some(false),
        }
    }
}

/// The index of a nucleotide on a Strand
pub struct PositionOnStrand {
    pub domain_id: usize,
    pub pos_on_domain: usize,
    pub pos_on_strand: usize,
}
