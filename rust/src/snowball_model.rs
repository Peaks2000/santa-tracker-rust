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
struct Track {
    node: usize,
    path: String,
    times: Vec<f32>,
    values: Vec<f32>,
    stride: usize,
}
pub struct Clip {
    pub name: String,
    pub duration: f32,
    tracks: Vec<Track>,
}
pub struct ElfModel {
    nodes: Vec<Node>,
    roots: Vec<usize>,
    joints: Vec<usize>,
    inverse: Vec<Mat4>,
    pub clips: Vec<Clip>,
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
        let clips = doc["animations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| {
                let tracks: Vec<_> = a["channels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| {
                        let s = &a["samplers"][c["sampler"].as_u64().unwrap() as usize];
                        assert_eq!(s["interpolation"], "LINEAR");
                        let path = c["target"]["path"].as_str().unwrap().to_owned();
                        Track {
                            node: c["target"]["node"].as_u64().unwrap() as usize,
                            stride: if path == "rotation" { 4 } else { 3 },
                            path,
                            times: read(s["input"].as_u64().unwrap() as usize),
                            values: read(s["output"].as_u64().unwrap() as usize),
                        }
                    })
                    .collect();
                let duration = tracks
                    .iter()
                    .filter_map(|t| t.times.last())
                    .copied()
                    .fold(0f32, f32::max);
                Clip {
                    name: a["name"].as_str().unwrap_or("").into(),
                    duration,
                    tracks,
                }
            })
            .collect();
        Ok(Self {
            nodes,
            roots,
            joints,
            inverse,
            clips,
            positions,
            normals,
            uv,
            colors,
            skin,
            weights,
            indices,
        })
    }
    fn pose(&self, name: &str, elapsed: f32, once: bool) -> Vec<Node> {
        let mut nodes = self.nodes.clone();
        if let Some(c) = self.clips.iter().find(|c| c.name == name) {
            let t = if c.duration <= 0. {
                0.
            } else if once {
                elapsed.min(c.duration)
            } else {
                elapsed % c.duration
            };
            for track in &c.tracks {
                let hi = track.times.partition_point(|v| *v <= t);
                let a = hi.saturating_sub(1);
                let b = hi.min(track.times.len() - 1);
                let f = if a == b {
                    0.
                } else {
                    (t - track.times[a]) / (track.times[b] - track.times[a])
                };
                let va = &track.values[a * track.stride..(a + 1) * track.stride];
                let vb = &track.values[b * track.stride..(b + 1) * track.stride];
                match track.path.as_str() {
                    "translation" => {
                        nodes[track.node].t = Vec3::from_slice(va).lerp(Vec3::from_slice(vb), f)
                    }
                    "scale" => {
                        nodes[track.node].s = Vec3::from_slice(va).lerp(Vec3::from_slice(vb), f)
                    }
                    "rotation" => {
                        nodes[track.node].r = Quat::from_array(va.try_into().unwrap())
                            .slerp(Quat::from_array(vb.try_into().unwrap()), f)
                    }
                    _ => {}
                }
            }
        }
        nodes
    }
    pub fn mesh(&self, name: &str, elapsed: f32, throwing: Option<f32>) -> Mesh {
        let mut nodes = self.pose(
            name,
            elapsed,
            name.contains("fall_down") && !name.ends_with("idle"),
        );
        if let Some(t) = throwing {
            let duration = self
                .clips
                .iter()
                .find(|c| c.name == "elf_rig_throw")
                .unwrap()
                .duration;
            let clock = t * 1.75;
            if clock < duration + 0.25 {
                let overlay = self.pose("elf_rig_throw", clock, true);
                let w = if clock <= duration {
                    1.
                } else {
                    1. - (clock - duration) / 0.25
                };
                for (a, b) in nodes.iter_mut().zip(overlay) {
                    a.t = a.t.lerp(b.t, w);
                    a.s = a.s.lerp(b.s, w);
                    a.r = a.r.slerp(b.r, w)
                }
            }
        }
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
    fn skinned_vertices_match_original_three_animation_mixer() {
        let model = ElfModel::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../static/scenes/snowball/models"),
        )
        .unwrap();
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-skin-reference.json")).unwrap();
        for row in data.as_array().unwrap() {
            let name = row["name"].as_str().unwrap();
            let time = row["time"].as_f64().unwrap() as f32; // Source harness calls Model.play (looped), including fall clips.
            let mesh = model.mesh(name, time, None);
            let duration = model
                .clips
                .iter()
                .find(|c| c.name == name)
                .unwrap()
                .duration;
            let elapsed = if duration > 0. { time % duration } else { 0. };
            let mesh = if name.contains("fall_down") && time >= duration {
                model.mesh(name, elapsed, None)
            } else {
                mesh
            };
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
