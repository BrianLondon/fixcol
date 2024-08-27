//! Genealogy Algorithms
//! 
//! Does some calculations on genealogy records. The functionality
//! here is not important for the serialization example, but is used
//! to transform the data from the input format to an output format

use std::collections::HashMap;

use crate::{OutputRecord, Record, RelationType};

#[derive(Debug, Clone)]
struct Person {
    id: u8,
    name: String,
    children: Vec<u8>,
}

pub(crate) fn coi_for_data_set(records: Vec<Record>) -> Vec<OutputRecord> {
    // build up dataset from records
    let mut people: HashMap<u8, Person> = records_to_genealogy(records);

    // cache the reverse mapping from children to parents
    let parents: HashMap<u8, (u8, u8)> = get_parents(&people);

    // map from identifiers to vec locations
    let mut ids: Vec<u8> = people.iter().map(|(_, p)| p.id).collect();
    ids.sort();
    let ids = ids;

    let idx_from_id: HashMap<&u8, usize> =
        HashMap::from_iter(ids.iter().enumerate().map(|(a, b)| (b, a)));

    let parents: Vec<_> = ids
        .iter()
        .map(|id| {
            parents.get(id).map(|(a, b)| {
                (*idx_from_id.get(a).unwrap() as usize, *idx_from_id.get(b).unwrap() as usize)
            })
        })
        .collect();

    let mut ordered_people: Vec<Person> = Vec::with_capacity(ids.len());
    for id in &ids {
        ordered_people.push(people.remove(id).unwrap());
    }

    let mut matrix = zeros(ids.len());

    for row in 0 .. matrix.len() {
        // Diag element: a_jj = 1 + 0.5 * a_pq
        if let Some((p1, p2)) = parents[row] {
            matrix[row][row] = 1.0 + 0.5 * matrix[p1][p2]
        } else {
            matrix[row][row] = 1.0;
        }

        for col in row + 1 .. matrix.len() {
            // Other elements: a_ij = 0.5(a_ip + a_iq)
            if let Some((p1, p2)) = parents[col] {
                let f = 0.5 * (matrix[row][p1] + matrix[row][p2]);
                matrix[row][col] = f;
                matrix[col][row] = f;
            } 
        }
    }
    
    let coi_values = diag_minus_one(matrix);

    let mut out: Vec<OutputRecord> = Vec::new();
    for i in 0 .. coi_values.len() {
        let coi = coi_values[i];
        let name = ordered_people[i].name.clone();
        
        out.push(OutputRecord { name, coi });
    }

    out.sort_by(|a, b| {
        b.coi.partial_cmp(&a.coi)
            .unwrap_or(a.name.cmp(&b.name))
            .then(a.name.cmp(&b.name))
    });
    out
}

fn records_to_genealogy(records: Vec<Record>) -> HashMap<u8, Person> {
    let mut people: HashMap<u8, Person> = HashMap::new();

    // Note we assume relations always come after the referenced person records
    for record in records {
        match record {
            Record::Person { id, name, regnal_number, birth: _, death: _ } => {
                let person = Person {
                    name: cat_name(&name, &regnal_number),
                    id: id,
                    children: Vec::new(),
                };
                people.insert(id, person);
            },
            Record::Relation { rel_type, from, to } => {
                if rel_type == RelationType::ParentChild {
                    people.get_mut(&from).unwrap().children.push(to);
                }  
            },
        }
    }

    people
}

fn get_parents(people: &HashMap<u8, Person>) -> HashMap<u8, (u8, u8)> {
    let mut map: HashMap<u8, (u8, Option<u8>)> = HashMap::new();

    for (_, parent) in people {
        for child in &parent.children {
            map.entry(*child).and_modify(|r| r.1 = Some(parent.id)).or_insert((parent.id, None));
        }
    }

    map.into_iter().map(|(k, v)| (k, (v.0, v.1.unwrap()))).collect()
}

fn zeros(size: usize) -> Vec<Vec<f32>> {
    let mut matrix = Vec::with_capacity(size);
    for _ in 0 .. size {
        let mut row = Vec::new();
        for _ in 0 .. size {
            row.push(0.0);
        }
        matrix.push(row);
    }
    matrix
}

fn diag_minus_one(data: Vec<Vec<f32>>) -> Vec<f32> {
    data.iter().enumerate().map(|(r, row)| row[r] - 1.0).collect()
}

// concatenates two strings inserting a space if the second is not empty
fn cat_name(a: &String, b: &String) -> String {
    if b.is_empty() {
        a.to_owned()
    } else {
        format!("{} {}", a, b)
    }
}
