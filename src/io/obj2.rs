use std::{
    collections::HashSet,
    os::raw,
    path::{Path, PathBuf},
};

use tobj::LoadOptions;

use crate::{
    Color, Geometry, Indices, LightingModel, Node, PbrMaterial, Positions, Result, Scene,
    Texture2D, TriMesh, Vec2, Vec3,
};

use super::RawAssets;

pub fn deserialize_obj(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    raw_assets.remove(path)?;
    let (models, materials_data) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        },
    )?;

    let p = path.parent().unwrap_or(Path::new(""));

    let mut insert_tex = move |texture_path: String| {
        if texture_path.is_empty() {
            return;
        }

        let tex_path_part: Vec<&str> = texture_path
            .split(['/', '\\'])
            .filter(|p| !p.is_empty())
            .collect();

        let tex_path = p.join(&PathBuf::from_iter(tex_path_part.iter()));

        let tex_bytes = std::fs::read(tex_path).unwrap();
        raw_assets.insert(texture_path.clone(), tex_bytes);
    };

    let mut load_tex = move |texture_path: String| -> Option<Texture2D> {
        let tex_de = raw_assets.deserialize(texture_path);
        if let Ok(tex) = tex_de {
            Some(tex)
        } else {
            None
        }
    };

    let mut materials = Vec::new();
    if let Ok(mats) = materials_data {
        for m in mats.iter() {
            if !m.diffuse_texture.is_empty() {
                insert_tex(m.diffuse_texture);
            }

            if !m.ambient_texture.is_empty() {
                insert_tex(m.ambient_texture);
            }

            if !m.normal_texture.is_empty() {
                insert_tex(m.normal_texture);
            }

            if !m.specular_texture.is_empty() {
                insert_tex(m.specular_texture);
            }

            if !m.dissolve_texture.is_empty() {
                insert_tex(m.dissolve_texture);
            }

            if !m.shininess_texture.is_empty() {
                insert_tex(m.shininess_texture);
            }

            materials.push(PbrMaterial {
                name: m.name.clone(),
                albedo: Color::from_rgba_slice(&[
                    m.diffuse[0],
                    m.diffuse[1],
                    m.diffuse[2],
                    m.dissolve,
                ]),
                albedo_texture: load_tex(m.diffuse_texture.clone()),
                metallic: (m.specular[0] + m.specular[1] + m.specular[2]) / 3.0,
                roughness: m.shininess,
                metallic_roughness_texture: load_tex(m.specular_texture.clone()),
                normal_texture: load_tex(m.normal_texture.clone()),
                lighting_model: LightingModel::Blinn,
                ..Default::default()
            });
        }
    }

    let mut nodes = Vec::new();

    for model in models.iter() {
        let positions: Vec<Vec3> = model
            .mesh
            .positions
            .chunks_exact(3)
            .map(|chunk| Vec3::new(chunk[0], chunk[1], chunk[2]))
            .collect();

        let normals: Vec<Vec3> = model
            .mesh
            .normals
            .chunks_exact(3)
            .map(|chunk| Vec3::new(chunk[0], chunk[1], chunk[2]))
            .collect();

        let indices: Vec<u32> = model.mesh.indices.clone();

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
            material_index: materials.iter().position(|m| m.name == model.name.clone()),
            ..Default::default()
        });
    }

    Ok(Scene {
        name: path.to_str().unwrap_or("default").to_owned(),
        children: nodes,
        materials,
    })
}
