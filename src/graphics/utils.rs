extern crate gl;
extern crate regex;

use self::regex::Regex;
use gl::types::{ GLfloat, GLuint };
use std::io::{ BufferedReader, File };

pub fn import_from_obj(filepath: &str) -> (Vec<GLfloat>, Vec<GLfloat>, Vec<GLuint>) {
    let comments_regex = Regex::new(r"\A\s*#(?s:.*)\z").ok().unwrap();
    let vertex_regex = Regex::new(r"\A\s*v\s+(\+?-?\d+\.\d+)\s+(\+?-?\d+\.\d+)\s+(\+?-?\d+\.\d+)\s*\z").ok().unwrap();
    let vertex_normal_regex = Regex::new(r"\A\s*vn\s+(\+?-?\d+\.\d+)\s+(\+?-?\d+\.\d+)\s+(\+?-?\d+\.\d+)\s*\z").ok().unwrap();
    let face_regex = Regex::new(r"\A\s*f\s+(\d+)/(\d+)?/(\d+)\s+(\d+)/(\d+)?/(\d+)\s+(\d+)/(\d+)?/(\d+)\s*\z").ok().unwrap();

    let mut file = BufferedReader::new(File::open(&Path::new(filepath)));
    let mut vertices: Vec<[GLfloat; 3]> = Vec::new();
    let mut normals: Vec<[GLfloat; 3]> = Vec::new();
    let mut indices: Vec<GLuint> = Vec::new();
    let mut normal_indices: Vec<GLuint> = Vec::new();

    for (line_num, line) in file.lines().enumerate() {
        let contents = line.unwrap();

        if comments_regex.is_match(contents.as_slice()) {
            continue;
        }

        match vertex_regex.captures(contents.as_slice()) {
            Some(capture) => {
                let vertex: Vec<GLfloat> = capture.iter().skip(1).map(|s| s.parse::<f32>().unwrap()).take(3).collect();
                vertices.push([vertex[0], vertex[1], vertex[2]]);
                continue;
            }

            None => { /* do nothing */ }
        }

        match vertex_normal_regex.captures(contents.as_slice()) {
            Some(capture) => {
                let normal: Vec<GLfloat> = capture.iter().skip(1).map(|s| s.parse::<f32>().unwrap()).take(3).collect();
                normals.push([normal[0], normal[1], normal[2]]);
                continue;
            }

            None => { /* do nothing */ }
        }

        match face_regex.captures(contents.as_slice()) {
            Some(capture) => {
                indices.push(capture.at(1).unwrap().parse::<u32>().unwrap() - 1);
                indices.push(capture.at(4).unwrap().parse::<u32>().unwrap() - 1);
                indices.push(capture.at(7).unwrap().parse::<u32>().unwrap() - 1);

                normal_indices.push(capture.at(3).unwrap().parse::<u32>().unwrap() - 1);
                normal_indices.push(capture.at(6).unwrap().parse::<u32>().unwrap() - 1);
                normal_indices.push(capture.at(9).unwrap().parse::<u32>().unwrap() - 1);
                continue;
            }

            None => { /* do nothing */ }
        }

        println!("[IGNORED] {:>20}:{:<4} {:?}", filepath, line_num, contents);
    }

    let (new_vertices, new_normals, new_indices) = unify_indexes(&indices, &vertices, &normal_indices, &normals);

    let flattened_vertices: Vec<GLfloat> = new_vertices.iter().flat_map(|a| a.iter().map(|a| *a)).collect();
    let flattened_normals: Vec<GLfloat> = new_normals.iter().flat_map(|a| a.iter().map(|a| *a)).collect();

    return (flattened_vertices, flattened_normals, new_indices);
}


fn unify_indexes<T: Clone>(indices_0: &Vec<u32>, values_0: &Vec<T>, indices_1: &Vec<u32>, values_1: &Vec<T>) -> (Vec<T>, Vec<T>, Vec<u32>) {
    let indices: Vec<(u32, u32)> = indices_0.iter().zip(indices_1.iter()).map(|(a, b)| (*a, *b)).collect();

    let mut available_index = 0u32;
    let mut consumed: Vec<(u32, u32)> = Vec::new();
    let mut new_indices: Vec<u32> = Vec::new();
    let mut new_values_0: Vec<T> = Vec::new();
    let mut new_values_1: Vec<T> = Vec::new();

    for (index, &index_pair) in indices.iter().enumerate() {
        let pos = indices.iter().position(|a| index_pair == *a).unwrap();

        if pos == index {
            // first occurrence
            consumed.push(index_pair);
            new_indices.push(available_index);
            new_values_0.push(values_0[index_pair.0 as usize].clone());
            new_values_1.push(values_1[index_pair.1 as usize].clone());
            available_index = available_index + 1;
        } else {
            // repeated occurrence
            new_indices.push(consumed.iter().position(|a| index_pair == *a).unwrap() as u32);
        }
    }

    return (new_values_0, new_values_1, new_indices);
}

#[test]
fn unify_indexes_test() {
    let v1: Vec<i32> = vec!(1, 2, 3, 4, 5, 6);
    let v2: Vec<i32> = vec!(-1, -2, -3, -4, -5);
    let i1 = vec!(
        0, 1, 2,
        3, 4, 5,
        0, 1, 2,
        5, 4, 3,
        3, 4, 5,
    );
    let i2 = vec!(
        3, 4, 1,
        0, 1, 2,
        4, 1, 3,
        2, 1, 0,
        2, 0, 1,
    );
    let (v3, v4, i3) = unify_indexes(&i1, &v1, &i2, &v2);

    // length must be preserved
    assert_eq!(i3.len(), 15);

    // repeated elements
    assert_eq!(i3[3], i3[11]);
    assert_eq!(i3[4], i3[10]);
    assert_eq!(i3[5], i3[9]);

    // unique pairs
    assert!(i3[0] != i3[6]);
    assert!(i3[1] != i3[7]);
    assert!(i3[2] != i3[8]);

    // resulting vectors have duplicates
    assert_eq!(v3.len(), 12);
    assert_eq!(v3.len(), v4.len());

    // unique values
    let mut v3_copy = v3.clone();
    v3_copy.sort();
    v3_copy.dedup();
    assert_eq!(v3_copy.len(), 6);

    // unique values
    let mut v4_copy = v4.clone();
    v4_copy.sort();
    v4_copy.dedup();
    assert_eq!(v4_copy.len(), 5);

    // unique indexes
    let mut i4 = i3.clone();
    i4.sort();
    i4.dedup();
    assert_eq!(i4.len(), 12);

    // indexes generated should be a sequence
    for i in range(0us, 11us) {
        assert_eq!(i4[i], i as u32);
    }
}
