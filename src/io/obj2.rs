use std::path::{Path, PathBuf};

use crate::{
    Color, Geometry, Indices, LightingModel, Node, PbrMaterial, Positions, Result, Scene,
    Texture2D, TriMesh, Vec2, Vec3,
};

use super::RawAssets;

pub fn deserialize_obj(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let (models, materials_data) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        },
    )?;

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
            material_index: model.mesh.material_id,
            ..Default::default()
        });
    }

    let mut load_tex = move |texture_path: String| -> Option<Texture2D> {
        if !texture_path.is_empty() {
            let mut tex_path = path.parent().unwrap();

            let tex_path_part: Vec<&str> = texture_path.split(&['/', '\\']).collect();
            for tp in tex_path_part.iter() {
                if !tp.is_empty() {
                    tex_path.join(tp);
                }
            }

            println!("{:?}", tex_path.clone());
            if let Ok(tex) = raw_assets.deserialize(tex_path) {
                println!("done");
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
            let color = if m.diffuse[0] != m.diffuse[1] || m.diffuse[1] != m.diffuse[2] {
                m.diffuse
            } else if m.specular[0] != m.specular[1] || m.specular[1] != m.specular[2] {
                m.specular
            } else if m.ambient[0] != m.ambient[1] || m.ambient[1] != m.ambient[2] {
                m.ambient
            } else {
                m.diffuse
            };

            materials.push(PbrMaterial {
                name: m.name.clone(),
                albedo: Color::from_rgb_slice(&[color[0], color[1], color[2]]),
                albedo_texture: load_tex(m.diffuse_texture.clone()),
                metallic: (m.specular[0] + m.specular[1] + m.specular[2]) / 3.0,
                roughness: m.shininess,
                // metallic_roughness_texture: load_tex(m.specular_texture.clone()),
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
