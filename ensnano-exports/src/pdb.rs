/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Export to pdb file format. The method used here is an adpatation from the one used in
//! [tacOxDNA](https://github.com/lorenzo-rovigatti/tacoxDNA)

use super::ultraviolet;
use ultraviolet::{Rotor3, Vec3};

struct PdbNucleotide {
    chain_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct PdbAtom {
    serial_number: usize,
    name: String,
    residue_name: String,
    chain_id: char,
    residue_idx: usize,
    position: Vec3,
}

const OCCUPENCY: f32 = 1.0;
const TEMPERATURE_FACTOR: f32 = 1.0;

impl PdbAtom {
    fn pdb_repr(&self) -> Result<String, std::fmt::Error> {
        // https://www.cgl.ucsf.edu/chimera/docs/UsersGuide/tutorials/framepdbintro.html
        use std::fmt::Write;
        let mut ret = String::with_capacity(80);
        write!(&mut ret, "ATOM")?; // 1-4
        ret.push_str("  "); // 5-6
        write!(&mut ret, "{:>5}", self.serial_number)?; // 7-11
        ret.push_str(" "); //12
        if self.name.len() < 4 {
            // we assume that all atoms that we manipulate have a one letter symbol which is
            // conveniently the case for all atoms of nucleic acids
            write!(&mut ret, " {:<3}", self.name)?; //13-16
        } else {
            write!(&mut ret, "{:<4}", self.name)?; //13-16
        }
        ret.push_str(" "); // 17
        write!(&mut ret, "{:>3}", self.residue_name)?; // 18-20
        write!(&mut ret, " {}", self.chain_id)?; //21-22
        write!(&mut ret, "{:>4}", self.residue_idx)?; // 23-26
        ret.push_str(&vec![" "; 4].join("")); // 27-30
        write!(&mut ret, "{:>8.3}", self.position.x)?; // 31-38
        write!(&mut ret, "{:>8.3}", self.position.y)?; // 39-46
        write!(&mut ret, "{:>8.3}", self.position.z)?; // 47-54
        write!(&mut ret, "{:>6.2}", OCCUPENCY)?; // 55-60
        write!(&mut ret, "{:>6.2}", TEMPERATURE_FACTOR)?; // 61-66
        ret.push_str(&vec![" "; 14].join("")); // 67-80
        Ok(ret)
    }

    fn parse_line<S: AsRef<str>>(input: &S) -> Result<Self, PdbAtomParseError> {
        let input: &str = input.as_ref();
        if !input.is_ascii() {
            return Err(PdbAtomParseError::InputIsNotAscii);
        }

        if input.len() < 66 {
            return Err(PdbAtomParseError::InputTooShort);
        }

        if &input[0..4] != "ATOM" {
            return Err(PdbAtomParseError::NotAnAtom);
        }

        let serial_number = input[6..11]
            .trim()
            .parse::<usize>()
            .map_err(|_| PdbAtomParseError::InvalidSerialNumber)?;
        let name = input[12..16].trim().to_string();
        let residue_name = input[17..20].trim().to_string();
        let chain_id: char = input
            .chars()
            .nth(21)
            .ok_or(PdbAtomParseError::InputTooShort)?;
        let residue_idx = input[22..26]
            .trim()
            .parse::<usize>()
            .map_err(|_| PdbAtomParseError::InvalidResidueSequenceNumber)?;

        let position_x = input[30..38]
            .trim()
            .parse::<f32>()
            .map_err(|_| PdbAtomParseError::InvalidCoordinateX)?;
        let position_y = input[38..46]
            .trim()
            .parse::<f32>()
            .map_err(|_| PdbAtomParseError::InvalidCoordinateY)?;
        let position_z = input[46..54]
            .trim()
            .parse::<f32>()
            .map_err(|_| PdbAtomParseError::InvalidCoordinateZ)?;

        Ok(Self {
            serial_number,
            name,
            residue_idx,
            chain_id,
            residue_name,
            position: Vec3::new(position_x, position_y, position_z),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum PdbAtomParseError {
    InputIsNotAscii,
    InputTooShort,
    NotAnAtom,
    InvalidSerialNumber,
    InvalidResidueSequenceNumber,
    InvalidCoordinateX,
    InvalidCoordinateY,
    InvalidCoordinateZ,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pdb_repr() {
        let expected =
            "ATOM      1  N9  DG5 A   1      55.550  70.279 208.461  1.00  1.00              ";

        let atom = PdbAtom {
            serial_number: 1,
            name: String::from("N9"),
            residue_name: String::from("DG5"),
            chain_id: 'A',
            position: Vec3::new(55.550, 70.279, 208.461),
            residue_idx: 1,
        };
        assert_eq!(atom.pdb_repr().unwrap(), expected);
    }

    #[test]
    fn parse_atom() {
        let atom = PdbAtom {
            serial_number: 1,
            name: String::from("N9"),
            residue_name: String::from("DG5"),
            chain_id: 'A',
            position: Vec3::new(55.550, 70.279, 208.461),
            residue_idx: 1,
        };
        let input =
            "ATOM      1  N9  DG5 A   1      55.550  70.279 208.461  1.00  1.00              ";

        let parsed_atom = PdbAtom::parse_line(&input).unwrap();
        assert_eq!(parsed_atom, atom);
    }
}
