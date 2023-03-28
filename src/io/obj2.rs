use std::{error::Error, path::PathBuf};

use obj::{load_obj, Obj};

use crate::{Geometry, Indices, Node, Positions, Result, Scene, TriMesh, Vec3};

use super::RawAssets;

pub fn deserialize_obj(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let obj_bytes = raw_assets.remove(path)?;
    let obj_data: Obj = load_obj(&obj_bytes[..])?;

    let materials = Vec::new();

    let mut nodes = Vec::new();

    let mut positions: Vec<Vec3> = vec![];
    let mut normals: Vec<Vec3> = vec![];

    for vertex in obj_data.vertices.iter() {
        positions.push(vertex.position.into());
        normals.push(vertex.normal.into());
    }

    nodes.push(Node {
        name: obj_data.name.unwrap(),
        geometry: Some(Geometry::Triangles(TriMesh {
            positions: Positions::F32(positions),
            normals: Some(normals),
            indices: Indices::U16(obj_data.indices),
            ..Default::default()
        })),
        ..Default::default()
    });

    Ok(Scene {
        name: path.to_str().unwrap_or("default").to_owned(),
        children: nodes,
        materials,
    })
}
