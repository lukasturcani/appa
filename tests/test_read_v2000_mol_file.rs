use std::path::PathBuf;

#[test]
fn read_file_1() -> Result<(), anyhow::Error> {
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?)
        .join("resources/test/v2000_mol_files/1.mol");
    let data = appa::read_v2000_mol_file(&path)?.next().unwrap()?;
    assert_eq!(data.title, "     RDKit          2D");
    assert_eq!(data.molecule.atomic_numbers, vec![6, 6, 7]);
    assert_eq!(data.molecule.atom_charges, None);
    assert_eq!(
        data.molecule.atom_coordinates,
        Some(vec![
            [0.0, 0.0, 0.0],
            [1.299, 0.75, 0.0],
            [2.5981, 0.0, 0.0]
        ])
    );
    let integer_bonds = data.molecule.integer_bonds.unwrap();
    assert_eq!(integer_bonds.atoms1, vec![1, 2]);
    assert_eq!(integer_bonds.atoms2, vec![2, 3]);
    assert_eq!(integer_bonds.orders, vec![1, 1]);
    assert!(data.molecule.dative_bonds.is_none());
    assert!(data.molecule.aromatic_bonds.is_none());
    Ok(())
}

#[test]
fn read_file_2() -> Result<(), anyhow::Error> {
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?)
        .join("resources/test/v2000_mol_files/2.mol");
    let data = appa::read_v2000_mol_file(&path)?.next().unwrap()?;
    assert_eq!(data.title, "     RDKit          2D");
    assert_eq!(data.molecule.atomic_numbers, vec![6, 6, 7]);
    assert_eq!(data.molecule.atom_charges, Some(vec![0, 2, 0]));
    assert_eq!(
        data.molecule.atom_coordinates,
        Some(vec![
            [0.0, 0.0, 0.0],
            [1.299, 0.75, 0.0],
            [2.5981, 1.5, 0.0]
        ])
    );
    let integer_bonds = data.molecule.integer_bonds.unwrap();
    assert_eq!(integer_bonds.atoms1, vec![1, 2]);
    assert_eq!(integer_bonds.atoms2, vec![2, 3]);
    assert_eq!(integer_bonds.orders, vec![1, 1]);
    assert!(data.molecule.dative_bonds.is_none());
    assert!(data.molecule.aromatic_bonds.is_none());
    Ok(())
}
