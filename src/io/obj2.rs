use std::path::{Path, PathBuf};

use crate::{
    Color, Geometry, Indices, LightingModel, Node, PbrMaterial, Positions, Result, Scene,
    Texture2D, TriMesh, Vec2, Vec3,
};

use super::RawAssets;

pub fn deserialize_obj(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let (models, materials_data) = tobj::load_obj(path, &tobj::LoadOptions::default())?;

    let mut nodes = Vec::new();

    for model in models.iter() {
        let mut positions: Vec<Vec3> = vec![];
        let mut normals: Vec<Vec3> = vec![];

        let mut indices = model.mesh.indices.clone();

        for idx in indices.iter() {
            let i = *idx as usize;
            positions.push(Vec3::new(
                model.mesh.positions[3 * i],
                model.mesh.positions[3 * i + 1],
                model.mesh.positions[3 * i + 2],
            ));
            normals.push(
                if !model.mesh.normals.is_empty() && model.mesh.normals.len() > 3 * i {
                    Vec3::new(
                        model.mesh.normals[3 * i],
                        model.mesh.normals[3 * i + 1],
                        model.mesh.normals[3 * i + 2],
                    )
                } else {
                    Vec3::new(0.0, 0.0, 0.0)
                },
            );
        }

        let uvs: Vec<Vec2> = model
            .mesh
            .texcoords
            .chunks_exact(2)
            .map(|chunk| Vec2::new(chunk[0], chunk[1]))
            .collect();

        nodes.push(Node {
            name: model.name.clone(),
            geometry: Some(Geometry::Triangles(TriMesh {
                positions: Positions::F32(positions),
                normals: Some(normals),
                indices: Indices::U32(indices),
                uvs: Some(uvs),
                ..Default::default()
            })),
            material_index: model.mesh.material_id,
            ..Default::default()
        });
    }

    let mut load_tex = move |texture_path: String| -> Option<Texture2D> {
        if !texture_path.is_empty() {
            if let Ok(tex) = raw_assets.deserialize(path.parent().unwrap_or(Path::new(""))) {
                Some(tex)
            } else {
                None
            }
        } else {
            None
        }
    };

    let mut materials = Vec::new();
    if let Ok(mats) = materials_data {
        for m in mats.iter() {
            materials.push(PbrMaterial {
                name: m.name.clone(),
                albedo: Color::from_rgb_slice(&m.diffuse),
                albedo_texture: load_tex(m.diffuse_texture.clone()),
                metallic: (m.specular[0] + m.specular[1] + m.specular[2]) / 3.0,
                roughness: m.shininess,
                normal_texture: load_tex(m.normal_texture.clone()),
                lighting_model: LightingModel::Blinn,
                ..Default::default()
            });
        }
    }

    Ok(Scene {
        name: path.to_str().unwrap_or("default").to_owned(),
        children: nodes,
        materials,
    })
}
