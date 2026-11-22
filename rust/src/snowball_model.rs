//! Native glTF skin/linear quaternion tracks for the original Snowball elf.
//! GLTFLoader r110 only binds JOINTS_0/WEIGHTS_0 (extra sets are not used).
use macroquad::prelude::*;
use serde_json::Value;
use std::path::Path;
#[derive(Clone)]
struct Node {
    t: Vec3,
    r: Quat,
    s: Vec3,
    children: Vec<usize>,
}
pub struct ElfModel {
    nodes: Vec<Node>,
    roots: Vec<usize>,
    joints: Vec<usize>,
    inverse: Vec<Mat4>,
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uv: Vec<Vec2>,
    colors: Vec<Vec3>,
    skin: Vec<[usize; 4]>,
    weights: Vec<Vec4>,
    indices: Vec<u16>,
}
fn arr(v: &Value, n: usize, default: f32) -> Vec<f32> {
    (0..n)
        .map(|i| v[i].as_f64().map(|x| x as f32).unwrap_or(default))
        .collect()
}
impl ElfModel {
    pub fn load(root: &Path) -> Result<Self, String> {
        let doc: Value = serde_json::from_slice(
            &std::fs::read(root.join("elf-animated.gltf")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let bytes = std::fs::read(root.join("elf-animated.bin")).map_err(|e| e.to_string())?;
        let read = |idx: usize| -> Vec<f32> {
            let a = &doc["accessors"][idx];
            let v = &doc["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
            let n = match a["type"].as_str().unwrap() {
                "VEC2" => 2,
                "VEC3" => 3,
                "VEC4" => 4,
                "MAT4" => 16,
                _ => 1,
            };
            let kind = a["componentType"].as_u64().unwrap();
            let size = match kind {
                5120 | 5121 => 1,
                5122 | 5123 => 2,
                _ => 4,
            };
            let stride = v["byteStride"].as_u64().unwrap_or((n * size) as u64) as usize;
            let offset = (v["byteOffset"].as_u64().unwrap_or(0)
                + a["byteOffset"].as_u64().unwrap_or(0)) as usize;
            let mut out = vec![];
            for i in 0..a["count"].as_u64().unwrap() as usize {
                for j in 0..n {
                    let p = offset + i * stride + j * size;
                    out.push(match kind {
                        5126 => f32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()),
                        5123 => u16::from_le_bytes(bytes[p..p + 2].try_into().unwrap()) as f32,
                        5121 => bytes[p] as f32,
                        5125 => u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()) as f32,
                        _ => panic!("unsupported glTF accessor"),
                    })
                }
            }
            out
        };
        let nodes = doc["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| Node {
                t: Vec3::from_slice(&arr(&n["translation"], 3, 0.)),
                r: if n["rotation"].is_array() {
                    Quat::from_array(arr(&n["rotation"], 4, 0.).try_into().unwrap())
                } else {
                    Quat::IDENTITY
                },
                s: Vec3::from_slice(&arr(&n["scale"], 3, 1.)),
                children: n["children"]
                    .as_array()
                    .map(|a| a.iter().map(|v| v.as_u64().unwrap() as usize).collect())
                    .unwrap_or_default(),
            })
            .collect();
        let roots = doc["scenes"][doc["scene"].as_u64().unwrap_or(0) as usize]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as usize)
            .collect();
        let skin = &doc["skins"][0];
        let joints = skin["joints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as usize)
            .collect();
        let inverse = read(skin["inverseBindMatrices"].as_u64().unwrap() as usize)
            .chunks_exact(16)
            .map(Mat4::from_cols_slice)
            .collect();
        let prim = &doc["meshes"][0]["primitives"][0];
        let attr = &prim["attributes"];
        let get = |key: &str| read(attr[key].as_u64().unwrap() as usize);
        let positions = get("POSITION")
            .chunks_exact(3)
            .map(Vec3::from_slice)
            .collect();
        let normals = get("NORMAL")
            .chunks_exact(3)
            .map(Vec3::from_slice)
            .collect();
        let uv = get("TEXCOORD_0")
            .chunks_exact(2)
            .map(Vec2::from_slice)
            .collect();
        let colors = get("COLOR_0")
            .chunks_exact(4)
            .map(|c| vec3(c[0], c[1], c[2]))
            .collect();
        let weights = get("WEIGHTS_0")
            .chunks_exact(4)
            .map(|p| {
                let v = Vec4::from_slice(p);
                v / v.abs().element_sum()
            })
            .collect();
        let indices = read(prim["indices"].as_u64().unwrap() as usize)
            .iter()
            .map(|&v| v as u16)
            .collect();
        let skin = get("JOINTS_0")
            .chunks_exact(4)
            .map(|p| [p[0] as usize, p[1] as usize, p[2] as usize, p[3] as usize])
            .collect();
        Ok(Self {
            nodes,
            roots,
            joints,
            inverse,
            positions,
            normals,
            uv,
            colors,
            skin,
            weights,
            indices,
        })
    }
    #[cfg(test)]
    pub fn mesh_pose(&self, pose: &[santa_tracker::snowball_animation::Transform]) -> Mesh {
        self.mesh_pose_scaled(pose, 1.)
    }
    pub fn mesh_pose_scaled(
        &self,
        pose: &[santa_tracker::snowball_animation::Transform],
        root_scale: f32,
    ) -> Mesh {
        assert_eq!(pose.len(), self.nodes.len());
        let mut nodes = self.nodes.clone();
        for (n, p) in nodes.iter_mut().zip(pose) {
            n.t = Vec3::from_array(p.t.map(|v| v as f32));
            n.r = Quat::from_array(p.r.map(|v| v as f32));
            n.s = Vec3::from_array(p.s.map(|v| v as f32));
        }
        for &root in &self.roots {
            nodes[root].s *= root_scale;
        }
        self.mesh_nodes(&nodes)
    }
    fn mesh_nodes(&self, nodes: &[Node]) -> Mesh {
        let mut world = vec![Mat4::IDENTITY; nodes.len()];
        fn visit(i: usize, parent: Mat4, n: &[Node], world: &mut [Mat4]) {
            let local = Mat4::from_scale_rotation_translation(n[i].s, n[i].r, n[i].t);
            world[i] = parent * local;
            for &j in &n[i].children {
                visit(j, world[i], n, world)
            }
        }
        for &i in &self.roots {
            visit(i, Mat4::IDENTITY, &nodes, &mut world)
        }
        let matrices: Vec<_> = self
            .joints
            .iter()
            .zip(&self.inverse)
            .map(|(&j, inv)| world[j] * (*inv))
            .collect();
        let vertices = self
            .positions
            .iter()
            .enumerate()
            .map(|(i, &p)| {
                let s = self.skin[i];
                let w = self.weights[i];
                let m = matrices[s[0]] * w.x
                    + matrices[s[1]] * w.y
                    + matrices[s[2]] * w.z
                    + matrices[s[3]] * w.w;
                let mut v = Vertex::new2(
                    m.transform_point3(p),
                    self.uv[i],
                    Color::new(self.colors[i].x, self.colors[i].y, self.colors[i].z, 1.),
                );
                v.normal = m.transform_vector3(self.normals[i]).extend(0.);
                v
            })
            .collect();
        Mesh {
            vertices,
            indices: self.indices.clone(),
            texture: None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_scaled_vertices_and_normals_match_original_skeleton() {
        let model = ElfModel::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../static/scenes/snowball/models"),
        )
        .unwrap();
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-animation-reference.json"))
                .unwrap();
        for case in data["cases"].as_array().unwrap() {
            let mut animator = santa_tracker::snowball_animation::Animator::new();
            for (step, row) in case["states"].as_array().unwrap().iter().enumerate() {
                let op = &row["op"];
                let now = op["at"].as_f64().unwrap();
                match op["op"].as_str().unwrap() {
                    "play" => {
                        animator.play(op["name"].as_str().unwrap(), now);
                    }
                    "mix" => animator.mix_once(op["name"].as_str().unwrap()),
                    "fall" => animator.fall(op["n"].as_u64().unwrap() as usize, now),
                    "update" => animator.update(now),
                    _ => unreachable!(),
                }
                let mesh = model.mesh_pose_scaled(&animator.pose, 5.);
                let scaled = &row["scaled"];
                for (j, index) in scaled["indices"].as_array().unwrap().iter().enumerate() {
                    let v = &mesh.vertices[index.as_u64().unwrap() as usize];
                    let normal = v.normal.truncate().normalize();
                    for (axis, (position, normal)) in v
                        .position
                        .to_array()
                        .into_iter()
                        .zip(normal.to_array())
                        .enumerate()
                    {
                        let source = scaled["positions"][j * 3 + axis].as_f64().unwrap() as f32;
                        assert!(
                            (position - source).abs() < 0.0002,
                            "{} step {step} vertex {index} axis {axis}: {position} != {source}",
                            case["name"]
                        );
                        let source = scaled["normals"][j * 3 + axis].as_f64().unwrap() as f32;
                        assert!(
                            (normal - source).abs() < 0.00003,
                            "{} step {step} normal {index} axis {axis}: {normal} != {source}",
                            case["name"]
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn skinned_vertices_match_original_three_animation_mixer() {
        let model = ElfModel::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../static/scenes/snowball/models"),
        )
        .unwrap();
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-skin-reference.json")).unwrap();
        for row in data.as_array().unwrap() {
            let name = row["name"].as_str().unwrap();
            let time = row["time"].as_f64().unwrap();
            let mut animator = santa_tracker::snowball_animation::Animator::new();
            animator.play(name, 0.);
            animator.update(time * 1000.);
            let mesh = model.mesh_pose(&animator.pose);
            for (i, v) in mesh.vertices.iter().enumerate() {
                for (j, a) in [v.position.x, v.position.y, v.position.z]
                    .into_iter()
                    .enumerate()
                {
                    let b = row["positions"][i * 3 + j].as_f64().unwrap() as f32;
                    assert!(
                        (a - b).abs() < 0.00003,
                        "{name} {time} vertex {i} axis {j}: {a} != {b}"
                    );
                }
            }
        }
    }
}
