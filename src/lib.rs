use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[source] std::io::Error),
    #[error("error parsing int: {1}")]
    ParseInt(#[source] std::num::ParseIntError, String),
    #[error("error parsing float: {1}")]
    ParseFloat(#[source] std::num::ParseFloatError, String),
    #[error("error parsing molecule: {0}")]
    ParseFile(String),
}

pub struct Molecule {
    pub atomic_numbers: Vec<u8>,
    pub atom_charges: Option<Vec<i8>>,
    pub atom_coordinates: Option<Vec<[f32; 3]>>,
    pub integer_bonds: Option<Bonds>,
    pub dative_bonds: Option<Bonds>,
    pub aromatic_bonds: Option<AromaticBonds>,
}

#[derive(Debug)]
pub struct Bonds {
    pub atoms1: Vec<u32>,
    pub atoms2: Vec<u32>,
    pub orders: Vec<u8>,
}

#[derive(Debug)]
pub struct AromaticBonds {
    pub atoms1: Vec<u32>,
    pub atoms2: Vec<u32>,
}

pub struct ReadMolFileV2000 {
    lines: Lines<BufReader<File>>,
    seek_block_end: bool,
}

pub struct MolFileV2000Data {
    pub title: String,
    pub molecule: Molecule,
    pub properties: Vec<Property>,
}

pub struct Property {
    pub key: String,
    pub value: String,
}

impl Iterator for ReadMolFileV2000 {
    type Item = Result<MolFileV2000Data, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seek_block_end {
            loop {
                if let Ok(line) = self.lines.next()? {
                    if line == "$$$$" {
                        self.seek_block_end = false;
                        break;
                    }
                }
            }
        }
        self.lines.next()?; // drop empty line
        let title = match self.lines.next()? {
            Ok(title) => title,
            Err(err) => {
                self.seek_block_end = true;
                return Some(Err(Error::Io(err)));
            }
        };
        self.lines.next()?; // drop empty line
        let counts = match self.lines.next()? {
            Ok(counts) => counts,
            Err(err) => {
                self.seek_block_end = true;
                return Some(Err(Error::Io(err)));
            }
        };
        let num_atoms = match counts[0..3].trim().parse::<u16>() {
            Ok(num_atoms) => num_atoms,
            Err(err) => {
                self.seek_block_end = true;
                return Some(Err(Error::ParseInt(
                    err,
                    format!(r#""{}" in line "{}""#, &counts[0..3], counts),
                )));
            }
        };
        let num_bonds = match counts[3..6].trim().parse::<u16>() {
            Ok(num_bonds) => num_bonds,
            Err(err) => {
                self.seek_block_end = true;
                return Some(Err(Error::ParseInt(
                    err,
                    format!(r#""{}" in line "{}""#, &counts[3..6], counts),
                )));
            }
        };
        let mut atomic_numbers = Vec::with_capacity(num_atoms as usize);
        let mut atom_coordinates = Vec::with_capacity(num_atoms as usize);
        for _ in 0..num_atoms {
            let line = self.lines.next()?;
            let line = match line {
                Ok(line) => line,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::Io(err)));
                }
            };
            let x = match line[0..10].trim().parse::<f32>() {
                Ok(x) => x,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::ParseFloat(
                        err,
                        format!(r#""{}" in line "{}""#, &line[0..10], line),
                    )));
                }
            };
            let y = match line[10..20].trim().parse::<f32>() {
                Ok(y) => y,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::ParseFloat(
                        err,
                        format!(r#""{}" in line "{}""#, &line[10..20], line),
                    )));
                }
            };
            let z = match line[20..30].trim().parse::<f32>() {
                Ok(z) => z,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::ParseFloat(
                        err,
                        format!(r#""{}" in line "{}""#, &line[20..30], line),
                    )));
                }
            };
            let atomic_number = match atomic_number_from_str(&line[31..34]) {
                Ok(atomic_number) => atomic_number,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(err));
                }
            };
            atomic_numbers.push(atomic_number);
            atom_coordinates.push([x, y, z]);
        }
        let mut atoms1 = Vec::with_capacity(num_bonds as usize);
        let mut atoms2 = Vec::with_capacity(num_bonds as usize);
        let mut orders = Vec::with_capacity(num_bonds as usize);
        for _ in 0..num_bonds {
            let line = self.lines.next()?;
            let line = match line {
                Ok(line) => line,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::Io(err)));
                }
            };
            let atom1 = match line[0..3].trim().parse::<u32>() {
                Ok(atom1) => atom1,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::ParseInt(
                        err,
                        format!(r#""{}" in line "{}""#, &line[0..3], line),
                    )));
                }
            };
            let atom2 = match line[3..6].trim().parse::<u32>() {
                Ok(atom2) => atom2,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::ParseInt(
                        err,
                        format!(r#""{}" in line "{}""#, &line[3..6], line),
                    )));
                }
            };
            let order = match line[6..9].trim().parse::<u8>() {
                Ok(order) => order,
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::ParseInt(
                        err,
                        format!(r#""{}" in line "{}""#, &line[6..9], line),
                    )));
                }
            };
            atoms1.push(atom1);
            atoms2.push(atom2);
            orders.push(order);
        }
        let mut atom_charges = None;
        while let Some(line) = self.lines.next() {
            match line {
                Ok(line) => {
                    if line == "M END" {
                        break;
                    }
                    if &line[3..6] == "CHG" {
                        if atom_charges.is_none() {
                            atom_charges = Some(vec![0; num_atoms as usize]);
                        }
                        let atom = match line[8..11].trim().parse::<u16>() {
                            Ok(atom) => atom,
                            Err(err) => {
                                self.seek_block_end = true;
                                return Some(Err(Error::ParseInt(
                                    err,
                                    format!(r#""{}" in line "{}""#, &line[8..11], line),
                                )));
                            }
                        };
                        let charge = match line[12..15].trim().parse::<i8>() {
                            Ok(charge) => charge,
                            Err(err) => {
                                self.seek_block_end = true;
                                return Some(Err(Error::ParseInt(
                                    err,
                                    format!(r#""{}" in line "{}""#, &line[12..15], line),
                                )));
                            }
                        };
                        let atom_charges = atom_charges.as_mut().unwrap();
                        atom_charges[atom as usize - 1] = charge;
                    }
                }
                Err(err) => {
                    self.seek_block_end = true;
                    return Some(Err(Error::Io(err)));
                }
            }
        }
        Some(Ok(MolFileV2000Data {
            title,
            molecule: Molecule {
                atomic_numbers,
                atom_charges,
                atom_coordinates: Some(atom_coordinates),
                integer_bonds: Some(Bonds {
                    atoms1,
                    atoms2,
                    orders,
                }),
                dative_bonds: None,
                aromatic_bonds: None,
            },
            properties: Vec::new(),
        }))
    }
}

pub fn read_v2000_mol_file(path: &Path) -> Result<ReadMolFileV2000, Error> {
    Ok(ReadMolFileV2000 {
        lines: BufReader::new(File::open(path).map_err(Error::Io)?).lines(),
        seek_block_end: false,
    })
}

fn atomic_number_from_str(s: &str) -> Result<u8, Error> {
    match s.trim() {
        "H" => Ok(1),
        "He" => Ok(2),
        "Li" => Ok(3),
        "Be" => Ok(4),
        "B" => Ok(5),
        "C" => Ok(6),
        "N" => Ok(7),
        "O" => Ok(8),
        "F" => Ok(9),
        "Ne" => Ok(10),
        "Na" => Ok(11),
        "Mg" => Ok(12),
        "Al" => Ok(13),
        "Si" => Ok(14),
        "P" => Ok(15),
        "S" => Ok(16),
        "Cl" => Ok(17),
        "Ar" => Ok(18),
        "K" => Ok(19),
        "Ca" => Ok(20),
        "Sc" => Ok(21),
        "Ti" => Ok(22),
        "V" => Ok(23),
        "Cr" => Ok(24),
        "Mn" => Ok(25),
        "Fe" => Ok(26),
        "Co" => Ok(27),
        "Ni" => Ok(28),
        "Cu" => Ok(29),
        "Zn" => Ok(30),
        "Ga" => Ok(31),
        "Ge" => Ok(32),
        "As" => Ok(33),
        "Se" => Ok(34),
        "Br" => Ok(35),
        "Kr" => Ok(36),
        "Rb" => Ok(37),
        "Sr" => Ok(38),
        "Y" => Ok(39),
        "Zr" => Ok(40),
        "Nb" => Ok(41),
        "Mo" => Ok(42),
        "Tc" => Ok(43),
        "Ru" => Ok(44),
        "Rh" => Ok(45),
        "Pd" => Ok(46),
        "Ag" => Ok(47),
        "Cd" => Ok(48),
        "In" => Ok(49),
        "Sn" => Ok(50),
        "Sb" => Ok(51),
        "Te" => Ok(52),
        "I" => Ok(53),
        "Xe" => Ok(54),
        "Cs" => Ok(55),
        "Ba" => Ok(56),
        "La" => Ok(57),
        "Ce" => Ok(58),
        "Pr" => Ok(59),
        "Nd" => Ok(60),
        "Pm" => Ok(61),
        "Sm" => Ok(62),
        "Eu" => Ok(63),
        "Gd" => Ok(64),
        "Tb" => Ok(65),
        "Dy" => Ok(66),
        "Ho" => Ok(67),
        "Er" => Ok(68),
        "Tm" => Ok(69),
        "Yb" => Ok(70),
        "Lu" => Ok(71),
        "Hf" => Ok(72),
        "Ta" => Ok(73),
        "W" => Ok(74),
        "Re" => Ok(75),
        "Os" => Ok(76),
        "Ir" => Ok(77),
        "Pt" => Ok(78),
        "Au" => Ok(79),
        "Hg" => Ok(80),
        "Tl" => Ok(81),
        "Pb" => Ok(82),
        "Bi" => Ok(83),
        "Po" => Ok(84),
        "At" => Ok(85),
        "Rn" => Ok(86),
        "Fr" => Ok(87),
        "Ra" => Ok(88),
        "Ac" => Ok(89),
        "Th" => Ok(90),
        "Pa" => Ok(91),
        "U" => Ok(92),
        "Np" => Ok(93),
        "Pu" => Ok(94),
        "Am" => Ok(95),
        "Cm" => Ok(96),
        "Bk" => Ok(97),
        "Cf" => Ok(98),
        "Es" => Ok(99),
        "Fm" => Ok(100),
        "Md" => Ok(101),
        "No" => Ok(102),
        "Lr" => Ok(103),
        "Rf" => Ok(104),
        "Db" => Ok(105),
        "Sg" => Ok(106),
        "Bh" => Ok(107),
        "Hs" => Ok(108),
        "Mt" => Ok(109),
        "Ds" => Ok(110),
        "Rg" => Ok(111),
        "Cn" => Ok(112),
        "Nh" => Ok(113),
        "Fl" => Ok(114),
        "Mc" => Ok(115),
        "Lv" => Ok(116),
        "Ts" => Ok(117),
        "Og" => Ok(118),
        _ => Err(Error::ParseFile(format!("unknown atomic number: {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        return;
        for data in read_v2000_mol_file(Path::new("/home/lukas/data/ChEBI_complete.sdf"))
            .unwrap()
            .filter_map(Result::ok)
        {
            todo!()
        }
    }
}
