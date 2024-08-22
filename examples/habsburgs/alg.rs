//! Genealogy Algorithms
//! 
//! Does some calculations on genealogy records. The functionality
//! here is not important for the serialization example, but is used
//! to transform the data from the input format to an output format

use std::collections::HashMap;

use crate::{OutputRecord, Record};

struct Person {
    id: u8,
    name: String,
    children: Vec<u8>,
}

pub(crate) fn coi_for_data_set(records: Vec<Record>) -> Vec<OutputRecord> {
    // build up dataset from records
    let mut people: HashMap<u8, Person> = records_to_genealogy(records);

    // cache the reverse mapping from children to parents
    let parents: HashMap<u8, (u8, u8)> = get_parents(people);

    // map from identifiers to vec locations
    let mut ids: Vec<u8> = people.iter().map(|(_, p)| p.id).collect().sort();
    let mut ordered_people: Vec<Person> = Vec::with_capacity(ids.len());
    for id in ids.iter().enumerate() {
        let (idx, id) = id;
        ordered_people[idx] = people.remove(id).unwrap();
    }

    let mut matrix = zeros(ids.len());

    for row in 0 .. matrix.len() {
        // Diag element: a_jj = 1 + 0.5 * a_pq
        let j: u8 = row as u8;
        let (p1, p2) = parents.get(&j).unwrap();
        matrix[row][row] = 1.0 + 0.5 * matrix[*p1 as usize][*p2 as usize];

        for col in row .. matrix.len() {
            // Other elements: a_ij = 0.5(a_ip + a_iq)
            let j: u8 = col as u8;
            let (p1, p2) = parents.get(&j).unwrap();
            matrix[row][col] = 0.5 * (matrix[row][*p1 as usize] + matrix[row][*p2 as usize]);
        }
    }
    
    let coi_values = diag(matrix);

    let out: Vec<OutputRecord> = Vec::new();
    for i in 0 .. coi_values.len() {
        let coi = coi_values[i];
        let name = ordered_people[i].name;
        
        out.push(OutputRecord { name, coi });
    }

    out.sort();
    out
}

fn records_to_genealogy(records: Vec<Record>) -> HashMap<u8, Person> {
    todo!();
}

fn get_parents(people: &HashMap<u8, Person>) -> HashMap<u8, (u8, u8)> {
    todo!();
}

fn zeros(size: usize) -> Vec<Vec<f32>> {
    todo!();
}

fn diag(data: Vec<Vec<f32>>) -> Vec<f32> {
    todo!();
}