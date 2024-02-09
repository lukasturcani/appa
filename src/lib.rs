use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[source] std::io::Error),
    #[error("error parsing int")]
    ParseInt(#[source] std::num::ParseIntError),
    #[error("error parsing float")]
    ParseFloat(#[source] std::num::ParseFloatError),
}

pub struct Molecule {
    pub atomic_numbers: Vec<u8>,
    pub atom_charges: Option<Vec<i8>>,
    pub atom_coordinates: Option<Vec<[f32; 3]>>,
    pub integer_bonds: Option<Bonds>,
    pub dative_bonds: Option<Bonds>,
    pub aromatic_bonds: Option<AromaticBonds>,
}

pub struct Bonds {
    pub atoms1: Vec<u32>,
    pub atoms2: Vec<u32>,
    pub order: Vec<u8>,
}

pub struct AromaticBonds {
    pub atoms1: Vec<u32>,
    pub atoms2: Vec<u32>,
}

pub struct ReadMolFileV2000 {
    lines: Lines<BufReader<File>>,
    seek_block_end: bool,
}

pub struct MolFileV2000Data {
    title: String,
    molecule: Molecule,
    properties: Vec<Property>,
}

pub struct Property {
    pub key: String,
    pub value: String,
}

impl Iterator for ReadMolFileV2000 {
    type Item = Result<MolFileV2000Data, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seek_block_end {
            todo!()
        }
        self.next()?; // drop empty line
        let title = match self.lines.next()? {
            Ok(title) => title,
            Err(err) => return Some(Err(Error::Io(err))),
        };
        self.next()?; // drop empty line
        let counts = match self.lines.next()? {
            Ok(counts) => counts,
            Err(err) => return Some(Err(Error::Io(err))),
        };
        let num_atoms = match counts[0..3].parse::<u16>() {
            Ok(num_atoms) => num_atoms,
            Err(err) => return Some(Err(Error::ParseInt(err))),
        };
        let num_bonds = match counts[3..6].parse::<u16>() {
            Ok(num_bonds) => num_bonds,
            Err(err) => return Some(Err(Error::ParseInt(err))),
        };
        let mut atomic_numbers = Vec::with_capacity(num_atoms as usize);
        let mut atom_charges = Vec::with_capacity(num_atoms as usize);
        let mut atom_coordinates = Vec::with_capacity(num_atoms as usize);
        for _ in 0..num_atoms {
            let line = self.lines.next()?;
            let line = match line {
                Ok(line) => line,
                Err(err) => return Some(Err(Error::Io(err))),
            };
            let x = match line[0..10].parse::<f32>() {
                Ok(x) => x,
                Err(err) => return Some(Err(Error::ParseFloat(err))),
            };
            let y = match line[10..20].parse::<f32>() {
                Ok(y) => y,
                Err(err) => return Some(Err(Error::ParseFloat(err))),
            };
            let z = match line[20..30].parse::<f32>() {
                Ok(z) => z,
                Err(err) => return Some(Err(Error::ParseFloat(err))),
            };
            // let atomic_number = atomic_number_from_str(line[31..34]);
            let atom_charge = match line[36..39].parse::<i8>() {
                Ok(atom_charge) => atom_charge,
                Err(err) => return Some(Err(Error::ParseInt(err))),
            };
            // atomic_numbers.push(atomic_number);
            atom_charges.push(atom_charge);
            atom_coordinates.push([x, y, z]);
        }
        Some(Ok(MolFileV2000Data {
            title,
            molecule: Molecule {
                atomic_numbers,
                atom_charges: Some(atom_charges),
                atom_coordinates: Some(atom_coordinates),
                integer_bonds: None,
                dative_bonds: None,
                aromatic_bonds: None,
            },
            properties: Vec::new(),
        }))
    }
}

pub fn read_mol_file_v2000(path: &Path) -> Result<ReadMolFileV2000, Error> {
    Ok(ReadMolFileV2000 {
        lines: BufReader::new(File::open(path).map_err(Error::Io)?).lines(),
        seek_block_end: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        for data in read_mol_file_v2000(Path::new("/home/lukas/data/ChEBI_complete.sdf"))
            .unwrap()
            .filter_map(Result::ok)
        {
            todo!()
        }
    }
}
