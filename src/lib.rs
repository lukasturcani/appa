use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[source] std::io::Error),
}

pub struct Molecule {
    pub atomic_numbers: Vec<u8>,
    pub atom_charges: Option<Vec<i8>>,
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
}

pub struct MolFileV2000Data {
    title: String,
}

pub struct Property {
    pub key: String,
    pub value: String,
}

impl Iterator for ReadMolFileV2000 {
    type Item = MolFileV2000Data;

    fn next(&mut self) -> Option<Self::Item> {
        self.next(); // drop empty line
        let title = self.next()?;
        self.next(); // drop empty line
        let counts = self.next()?;
        let num_atoms = counts[0..3].parse::<u16>().unwrap();

        todo!()
    }
}

pub fn read_mol_file_v2000(path: &Path) -> Result<ReadMolFileV2000, Error> {
    Ok(ReadMolFileV2000 {
        lines: BufReader::new(File::open(path).map_err(Error::Io)?).lines(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        for (molecule, properties) in
            read_mol_file_v2000(Path::new("/home/lukas/data/ChEBI_complete.sdf")).unwrap()
        {
            todo!()
        }
    }
}
