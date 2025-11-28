use super::*;
use crate::strands::{
    DomainJunction, HelixInterval, read_junctions, sanitize_domains,
};
use regex::Regex;
use std::fmt::Write as _;

#[test]
fn sanitize_with_insertions() {
    let domains = vec![
        Domain::HelixDomain(HelixInterval {
            helix: 0,
            start: 0,
            end: 10,
            forward: true,
            sequence: None,
        }),
        Domain::new_insertion(3),
        Domain::new_insertion(5),
        Domain::HelixDomain(HelixInterval {
            helix: 1,
            start: 0,
            end: 10,
            forward: false,
            sequence: None,
        }),
    ];
    assert_sane_domains_non_cyclic(sanitize_domains(&domains, false).as_slice());
}

#[test]
fn sanitize_domains_scadnano() {
    let input = r##" {
  "version": "0.15.0",
  "grid": "square",
  "helices": [
    {"grid_position": [0, 0]},
    {"grid_position": [0, 1]}
  ],
  "strands": [
    {
      "circular": true,
      "color": "#57bb00",
      "domains": [
        {"helix": 0, "forward": true, "start": 8, "end": 16},
        {"loopout": 5},
        {"helix": 1, "forward": false, "start": 8, "end": 16}
      ]
    }
  ]
      }"##;
    let scadnano_design: ScadnanoDesign =
        serde_json::from_str(input).expect("Failed to parse scadnano input");
    let ensnano_design =
        Design::from_scadnano(&scadnano_design).expect("Could not convert to ensnano");
    assert_eq!(ensnano_design.strands.len(), 1);
    let strand = ensnano_design.strands.values().next().unwrap();
    assert_eq!(strand.domains.len(), 3);
    assert!(strand.is_cyclic);
    let sane_domains = sanitize_domains(strand.domains.as_slice(), true);
    assert_sane_domains_non_cyclic(sane_domains.as_slice());
    assert_sane_domains_cyclic(sane_domains.as_slice());
    let junctions = read_junctions(sane_domains.as_slice(), true);
    assert_eq!(
        junctions,
        vec![
            DomainJunction::Adjacent,
            DomainJunction::UnidentifiedXover,
            DomainJunction::UnidentifiedXover
        ]
    );
}

#[test]
fn scadnano_import_one_loopout() {
    let input = r##" {
  "version": "0.15.0",
  "grid": "square",
  "helices": [
    {"grid_position": [0, 0]},
    {"grid_position": [0, 1]}
  ],
  "strands": [
    {
      "circular": true,
      "color": "#57bb00",
      "domains": [
        {"helix": 0, "forward": true, "start": 8, "end": 16},
        {"loopout": 5},
        {"helix": 1, "forward": false, "start": 8, "end": 16}
      ]
    }
  ]
      }"##;
    let scadnano_design: ScadnanoDesign =
        serde_json::from_str(input).expect("Failed to parse scadnano input");
    let ensnano_design =
        Design::from_scadnano(&scadnano_design).expect("Could not convert to ensnano");
    assert_eq!(ensnano_design.strands.len(), 1);
    let strand = ensnano_design.strands.values().next().unwrap();
    assert_good_strand(strand, "[H0: 8 -> 15] [@5] [H1: 8 <- 15]");
}

fn assert_good_strand<S: std::ops::Deref<Target = str>>(strand: &Strand, objective: S) {
    let re = Regex::new(r"\[[^\]]*\]").unwrap();
    let formatted_strand = strand.formatted_domains();
    let left = re.find_iter(&formatted_strand);
    let right = re.find_iter(&objective);
    for (a, b) in left.zip(right) {
        assert_eq!(a.as_str(), b.as_str());
    }
}

fn assert_sane_domains_non_cyclic(dom: &[Domain]) {
    let mut prev_insertion = false;
    for d in dom {
        if let Domain::Insertion { .. } = d {
            if prev_insertion {
                panic!("Two successive Insertions in {dom:?}");
            } else {
                prev_insertion = true;
            }
        } else {
            prev_insertion = false;
        }
    }
}

fn assert_sane_domains_cyclic(dom: &[Domain]) {
    if dom.len() >= 2
        && let Some(Domain::Insertion { .. }) = dom.first()
        && let Some(Domain::Insertion { .. }) = dom.last()
    {
        panic!("First and last domains are both insertions in cyclic strand")
    }
}

#[test]
fn correct_sanitization() {
    let mut strand = strand_with_insertion();
    let sane_domains = sanitize_domains(&strand.domains, false);
    strand.domains = sane_domains;
    assert_good_strand(&strand, formatted_sane_strand_with_insertion());
}

#[test]
fn correct_sanitization_cyclic() {
    let mut strand = strand_with_insertion();
    strand.is_cyclic = true;
    let sane_domains = sanitize_domains(&strand.domains, true);
    assert_eq!(
        sane_domains.iter().map(Domain::length).collect::<Vec<_>>(),
        vec![4, 8, 4, 5, 8]
    );
}

#[test]
fn correct_sanitization_cyclic_pathological() {
    let mut strand = strand_with_insertion();
    strand.is_cyclic = true;
    let add_prime5 = 123;
    strand.domains.insert(0, Domain::new_insertion(add_prime5));
    let add_prime3 = 9874;
    strand.domains.push(Domain::new_insertion(add_prime3));
    let sane_domains = sanitize_domains(&strand.domains, true);
    assert_eq!(
        sane_domains.iter().map(Domain::length).collect::<Vec<_>>(),
        vec![4, 8, 4, 5, 8, add_prime5 + add_prime3]
    );
    strand.domains = sane_domains;
    let mut expected = String::from(formatted_sane_strand_with_insertion());
    write!(&mut expected, "[@{}]", add_prime5 + add_prime3).unwrap();
    assert_good_strand(&strand, expected);
}

#[test]
fn correct_sanitization_cyclic_insertion_5prime() {
    let mut strand = strand_with_insertion();
    strand.is_cyclic = true;
    let insertion_prime5 = 17;
    strand
        .domains
        .insert(0, Domain::new_insertion(insertion_prime5));
    let sane_domains = sanitize_domains(&strand.domains, true);
    assert_eq!(
        sane_domains.iter().map(Domain::length).collect::<Vec<_>>(),
        vec![4, 8, 4, 5, 8, insertion_prime5]
    );
}

#[test]
fn correct_sanitization_cyclic_insertion_3prime() {
    let mut strand = strand_with_insertion();
    strand.is_cyclic = true;
    let insertion_prime3 = 17;
    strand.domains.push(Domain::new_insertion(insertion_prime3));
    let sane_domains = sanitize_domains(&strand.domains, true);
    assert_eq!(
        sane_domains.iter().map(Domain::length).collect::<Vec<_>>(),
        vec![4, 8, 4, 5, 8, insertion_prime3]
    );
}

#[test]
fn correct_sanitization_cyclic_insertion_3prime_5prime() {
    let mut strand = strand_with_insertion();
    strand.is_cyclic = true;
    let insertion_prime5 = 12;
    let insertion_prime3 = 17;
    strand
        .domains
        .insert(0, Domain::new_insertion(insertion_prime5));
    strand.domains.push(Domain::new_insertion(insertion_prime3));
    let sane_domains = sanitize_domains(&strand.domains, true);
    assert_eq!(
        sane_domains.iter().map(Domain::length).collect::<Vec<_>>(),
        vec![4, 8, 4, 5, 8, insertion_prime3 + insertion_prime5]
    );
}

#[test]
fn correct_junction_insertions() {
    let strand = strand_with_insertion();
    assert_eq!(strand.domains.len(), 6);
    let sane_domains = sanitize_domains(&strand.domains, false);
    assert_sane_domains_non_cyclic(&sane_domains);
    let junctions = read_junctions(sane_domains.as_slice(), false);
    assert_eq!(
        junctions,
        vec![
            DomainJunction::Adjacent,
            DomainJunction::Adjacent,
            DomainJunction::Adjacent,
            DomainJunction::UnidentifiedXover,
            DomainJunction::Prime3
        ]
    );
}

#[test]
fn correct_insertion_points() {
    let mut strand = strand_with_insertion();
    let sane_domains = sanitize_domains(strand.domains.as_slice(), false);
    strand.domains = sane_domains;
    let insertion_points = strand.insertion_points();
    assert_eq!(
        insertion_points,
        vec![
            (
                Some(Nucl {
                    helix: 1,
                    position: 3,
                    forward: true
                }),
                Some(Nucl {
                    helix: 1,
                    position: 4,
                    forward: true
                })
            ),
            (
                Some(Nucl {
                    helix: 1,
                    position: 7,
                    forward: true
                }),
                Some(Nucl {
                    helix: 2,
                    position: 7,
                    forward: false
                })
            ),
        ]
    );
}

#[test]
fn correct_insertion_prime5() {
    let mut strand = strand_with_insertion();
    strand.domains.insert(0, Domain::new_insertion(4));
    let sane_domains = sanitize_domains(strand.domains.as_slice(), false);
    strand.domains = sane_domains;
    let insertion_points = strand.insertion_points();
    assert_eq!(
        insertion_points,
        vec![
            (
                None,
                Some(Nucl {
                    helix: 1,
                    position: 0,
                    forward: true
                })
            ),
            (
                Some(Nucl {
                    helix: 1,
                    position: 3,
                    forward: true
                }),
                Some(Nucl {
                    helix: 1,
                    position: 4,
                    forward: true
                })
            ),
            (
                Some(Nucl {
                    helix: 1,
                    position: 7,
                    forward: true
                }),
                Some(Nucl {
                    helix: 2,
                    position: 7,
                    forward: false
                })
            ),
        ]
    );
}

#[test]
fn correct_insertion_prime3() {
    let mut strand = strand_with_insertion();
    strand.domains.push(Domain::new_insertion(4));
    let sane_domains = sanitize_domains(strand.domains.as_slice(), false);
    strand.domains = sane_domains;
    let insertion_points = strand.insertion_points();
    assert_eq!(
        insertion_points,
        vec![
            (
                Some(Nucl {
                    helix: 1,
                    position: 3,
                    forward: true
                }),
                Some(Nucl {
                    helix: 1,
                    position: 4,
                    forward: true
                })
            ),
            (
                Some(Nucl {
                    helix: 1,
                    position: 7,
                    forward: true
                }),
                Some(Nucl {
                    helix: 2,
                    position: 7,
                    forward: false
                })
            ),
            (
                Some(Nucl {
                    helix: 2,
                    position: 0,
                    forward: false
                }),
                None
            )
        ]
    );
}

#[test]
fn correct_insertion_cyclic_prime5() {
    let mut strand = strand_with_insertion();
    strand.domains.insert(0, Domain::new_insertion(4));
    let sane_domains = sanitize_domains(strand.domains.as_slice(), true);
    strand.domains = sane_domains;
    strand.is_cyclic = true;
    let insertion_points = strand.insertion_points();
    assert_eq!(
        insertion_points,
        vec![
            (
                Some(Nucl {
                    helix: 1,
                    position: 3,
                    forward: true
                }),
                Some(Nucl {
                    helix: 1,
                    position: 4,
                    forward: true
                })
            ),
            (
                Some(Nucl {
                    helix: 1,
                    position: 7,
                    forward: true
                }),
                Some(Nucl {
                    helix: 2,
                    position: 7,
                    forward: false
                })
            ),
            (
                Some(Nucl {
                    helix: 2,
                    position: 0,
                    forward: false
                }),
                Some(Nucl {
                    helix: 1,
                    position: 0,
                    forward: true
                })
            ),
        ]
    );
}

#[test]
fn correct_insertion_cyclic_prime3() {
    let mut strand = strand_with_insertion();
    strand.domains.push(Domain::new_insertion(4));
    let sane_domains = sanitize_domains(strand.domains.as_slice(), true);
    strand.domains = sane_domains;
    strand.is_cyclic = true;
    let insertion_points = strand.insertion_points();
    assert_eq!(
        insertion_points,
        vec![
            (
                Some(Nucl {
                    helix: 1,
                    position: 3,
                    forward: true
                }),
                Some(Nucl {
                    helix: 1,
                    position: 4,
                    forward: true
                })
            ),
            (
                Some(Nucl {
                    helix: 1,
                    position: 7,
                    forward: true
                }),
                Some(Nucl {
                    helix: 2,
                    position: 7,
                    forward: false
                })
            ),
            (
                Some(Nucl {
                    helix: 2,
                    position: 0,
                    forward: false
                }),
                Some(Nucl {
                    helix: 1,
                    position: 0,
                    forward: true
                })
            )
        ]
    );
}

#[test]
fn correct_insertion_cyclic_prime5_prime3() {
    let mut strand = strand_with_insertion();
    strand.domains.insert(0, Domain::new_insertion(3));
    strand.domains.push(Domain::new_insertion(4));
    let sane_domains = sanitize_domains(strand.domains.as_slice(), true);
    strand.domains = sane_domains;
    strand.is_cyclic = true;
    let insertion_points = strand.insertion_points();
    assert_eq!(
        insertion_points,
        vec![
            (
                Some(Nucl {
                    helix: 1,
                    position: 3,
                    forward: true
                }),
                Some(Nucl {
                    helix: 1,
                    position: 4,
                    forward: true
                })
            ),
            (
                Some(Nucl {
                    helix: 1,
                    position: 7,
                    forward: true
                }),
                Some(Nucl {
                    helix: 2,
                    position: 7,
                    forward: false
                })
            ),
            (
                Some(Nucl {
                    helix: 2,
                    position: 0,
                    forward: false
                }),
                Some(Nucl {
                    helix: 1,
                    position: 0,
                    forward: true
                })
            ),
        ]
    );
}

#[test]
fn correct_junction_cyclic_pathological() {
    let mut strand = strand_with_insertion();
    strand.is_cyclic = true;
    let insertion_prime5 = 12;
    let insertion_prime3 = 17;
    strand
        .domains
        .insert(0, Domain::new_insertion(insertion_prime5));
    strand.domains.push(Domain::new_insertion(insertion_prime3));
    let sane_domains = sanitize_domains(&strand.domains, true);
    let junctions = read_junctions(sane_domains.as_slice(), strand.is_cyclic);
    assert_eq!(
        junctions,
        vec![
            DomainJunction::Adjacent,
            DomainJunction::Adjacent,
            DomainJunction::Adjacent,
            DomainJunction::UnidentifiedXover,
            DomainJunction::Adjacent,
            DomainJunction::UnidentifiedXover
        ]
    );
}

#[test]
fn correct_insertion_left_to_right() {
    let mut strand = strand_with_insertion();
    let domains = sanitize_domains(&strand.domains, strand.is_cyclic);
    strand.domains = domains;
    let first_insertion = 1552;
    strand.add_insertion_at_nucl(
        &Nucl {
            helix: 1,
            position: 6,
            forward: true,
        },
        first_insertion,
    );

    let second_insertion = 172634;
    strand.add_insertion_at_nucl(
        &Nucl {
            helix: 2,
            position: 5,
            forward: false,
        },
        second_insertion,
    );

    let objective = format!(
        "[H1: 0 -> 3] [@8] [H1: 4 -> 6] [@{first_insertion}] [H1: 7 -> 7] [@5] [H2: 5 <- 7] [@{second_insertion}] [H2: 0 <- 4]",
    );
    assert_good_strand(&strand, objective);
}

#[test]
fn correct_insertion_right_to_left() {
    let mut strand = strand_with_insertion();
    let domains = sanitize_domains(&strand.domains, strand.is_cyclic);
    strand.domains = domains;
    let first_insertion = 1552;

    let second_insertion = 172634;
    strand.add_insertion_at_nucl(
        &Nucl {
            helix: 2,
            position: 5,
            forward: false,
        },
        second_insertion,
    );
    strand.add_insertion_at_nucl(
        &Nucl {
            helix: 1,
            position: 6,
            forward: true,
        },
        first_insertion,
    );

    let objective = format!(
        "[H1: 0 -> 3] [@8] [H1: 4 -> 6] [@{first_insertion}] [H1: 7 -> 7] [@5] [H2: 5 <- 7] [@{second_insertion}] [H2: 0 <- 4]",
    );
    assert_good_strand(&strand, objective);
}

/// A strand whose initial topology is [H1: 0 -> 3] [@5] [@3] [H1: 4 -> 7] [@5] [H2: 0 <- 7]
fn strand_with_insertion() -> Strand {
    let strand_str = include_str!("./strand_with_insertion.json");
    let strand: Strand = serde_json::from_str(strand_str).expect("Could not parse strand");
    strand
}

fn formatted_strand_with_insertion() -> &'static str {
    "[H1: 0 -> 3] [@5] [@3] [H1: 4 -> 7] [@5] [H2: 0 <- 7]"
}

fn formatted_sane_strand_with_insertion() -> &'static str {
    "[H1: 0 -> 3] [@8] [H1: 4 -> 7] [@5] [H2: 0 <- 7]"
}

#[test]
fn check_formatted_strand_with_insertion() {
    let strand = strand_with_insertion();
    assert_good_strand(&strand, formatted_strand_with_insertion());
}
