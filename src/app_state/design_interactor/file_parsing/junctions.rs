use ensnano_design::{
    domains::{Domain, helix_interval::HelixInterval, sanitize_domains},
    nucl::Nucl,
    strands::{DomainJunction, Strand},
};
use ensnano_utils::id_generator::IdGenerator;

pub(crate) trait StrandJunction {
    /// Read the junctions for self when loading the design.
    /// If `identified` is true (i.e. during the first pass), read the IdentifiedXover
    /// and insert them in the xover_ids.
    /// If `identified` is false (i.e. during the second pass), read the unidentified Xover and
    /// provide them with identifier
    ///
    /// Assumes that self.junctions is either empty or a Vec with the following properties.
    /// * Its length is equal to self.domains.length
    /// * All the junctions are appropriate.
    fn read_junctions(&mut self, xover_ids: &mut IdGenerator<(Nucl, Nucl)>, identified: bool);
}

impl StrandJunction for Strand {
    fn read_junctions(&mut self, xover_ids: &mut IdGenerator<(Nucl, Nucl)>, identified: bool) {
        //TODO check validity of self.junctions
        if self.junctions.is_empty() {
            let sane_domains = sanitize_domains(&self.domains, self.is_cyclic);
            self.domains = sane_domains;
            let junctions = read_junctions(&self.domains, self.is_cyclic);
            self.junctions = junctions;
        }
        if self.domains.is_empty() {
            return;
        }
        let mut previous_domain = self.domains.last().unwrap();
        for i in 0..(self.domains.len()) {
            let current = &self.domains[i];
            let next = if i == self.domains.len() - 1 {
                if self.is_cyclic {
                    &self.domains[0]
                } else {
                    break;
                }
            } else {
                &self.domains[i + 1]
            };
            match &mut self.junctions[i] {
                s @ DomainJunction::UnidentifiedXover => {
                    if !identified {
                        if let (Domain::HelixDomain(d1), Domain::HelixDomain(d2)) = (current, next)
                        {
                            let prime5 = d1.prime3();
                            let prime3 = d2.prime5();
                            let id = xover_ids.insert((prime5, prime3));
                            *s = DomainJunction::IdentifiedXover(id);
                        } else if let (Domain::HelixDomain(d1), Domain::HelixDomain(d2)) =
                            (previous_domain, next)
                        {
                            let prime5 = d1.prime3();
                            let prime3 = d2.prime5();
                            let id = xover_ids.insert((prime5, prime3));
                            *s = DomainJunction::IdentifiedXover(id);
                        } else if let Domain::Insertion { .. } = next {
                            panic!("UnidentifiedXover before an insertion");
                        } else if let Domain::Insertion { .. } = previous_domain {
                            panic!("Invariant violated: SaneDomains");
                        } else {
                            unreachable!("Non-exhaustive match");
                        }
                    }
                }
                DomainJunction::IdentifiedXover(id) => {
                    if identified {
                        if let (Domain::HelixDomain(d1), Domain::HelixDomain(d2)) = (current, next)
                        {
                            let prime5 = d1.prime3();
                            let prime3 = d2.prime5();
                            xover_ids.insert_at((prime5, prime3), *id);
                        } else if let (Domain::HelixDomain(d1), Domain::HelixDomain(d2)) =
                            (previous_domain, next)
                        {
                            let prime5 = d1.prime3();
                            let prime3 = d2.prime5();
                            xover_ids.insert_at((prime5, prime3), *id);
                        } else if let Domain::Insertion { .. } = next {
                            panic!("IdentifiedXover before an insertion");
                        } else if let Domain::Insertion { .. } = previous_domain {
                            panic!("Invariant violated: SaneDomains");
                        } else {
                            unreachable!("Non-exhaustive match");
                        }
                    }
                }
                _ => (),
            }
            if let Domain::HelixDomain(_) = current {
                previous_domain = current;
            }
        }
    }
}

/// Return the appropriate junction between two HelixInterval
pub(super) fn junction(prime5: &HelixInterval, prime3: &HelixInterval) -> DomainJunction {
    let prime5_nucl = prime5.prime3();
    let prime3_nucl = prime3.prime5();

    if prime3_nucl == prime5_nucl.prime3() {
        DomainJunction::Adjacent
    } else {
        DomainJunction::UnidentifiedXover
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
pub(super) fn read_junctions(domains: &[Domain], cyclic: bool) -> Vec<DomainJunction> {
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
