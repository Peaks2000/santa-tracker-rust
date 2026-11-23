//! THREE r110 transparent ordering; bounded production use for iceberg shells.
use crate::snowball::V;
use crate::snowball_removal::{Ice, Removal};
use std::{cmp::Ordering, f64::consts::PI};

#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub group_order: i32,
    pub render_order: i32,
    pub depth: f64,
    pub id: usize,
}
pub fn compare(a: &Key, b: &Key) -> Ordering {
    if a.group_order != b.group_order {
        a.group_order.cmp(&b.group_order)
    } else if a.render_order != b.render_order {
        a.render_order.cmp(&b.render_order)
    } else if a.depth != b.depth {
        // JS sort treats the source comparator's NaN subtraction as zero,
        // without falling through to the object-ID tie break.
        b.depth.partial_cmp(&a.depth).unwrap_or(Ordering::Equal)
    } else {
        a.id.cmp(&b.id)
    }
}

/// Object-origin clip depth for the original local camera and gimbal, in f64.
/// Source sorting uses JS matrix doubles, not GPU/f32 vertex positions. The
/// camera has fixed X=-PI, Z=0 rotation after Game.setLevel's lookAt/z reset.
pub fn iceberg_depth(body: &Ice, camera: V) -> f64 {
    let half = (4. * PI / 5.) / 2.;
    let x = half.sin();
    let w = half.cos();
    let x2 = x + x;
    let c = 1. - x * x2;
    let s = w * x2;
    let world_y = 0. * body.pos.x + c * body.pos.y + (-s) * body.z + 0.;
    let world_z = 0. * body.pos.x + s * body.pos.y + c * body.z + 0.;
    let camera_s = -(-PI / 2.).sin() * ((-PI / 2.).cos() * 2.);
    let p = 1. / (100000. - 1.);
    let zz = -2. * p;
    let zy = zz * camera_s;
    let zt = zz * (-camera_s * camera.y - 3200.) - (100000. + 1.) * p;
    0. * body.pos.x + zy * world_y + (-zz) * world_z + zt
}

pub fn iceberg_slots(removal: &Removal, camera: V) -> Vec<usize> {
    let mut slots = removal.layer.clone();
    slots.sort_by(|&a, &b| {
        compare(
            &Key {
                group_order: 0,
                render_order: 1,
                depth: iceberg_depth(&removal.bodies[a], camera),
                id: a,
            },
            &Key {
                group_order: 0,
                render_order: 1,
                depth: iceberg_depth(&removal.bodies[b], camera),
                id: b,
            },
        )
    });
    // At fixed source cellSize=64, each graphic is created once per body, so
    // slot creation order is equivalent to ascending source graphic Object3D ID.
    // A radius change recreates source graphics and needs separate ID tracking.
    slots
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    fn fixture() -> Value {
        serde_json::from_str(include_str!("../data/snowball-render-order-reference.json")).unwrap()
    }
    fn key(v: &Value) -> Key {
        Key {
            group_order: v["groupOrder"].as_i64().unwrap() as i32,
            render_order: v["renderOrder"].as_i64().unwrap() as i32,
            depth: v["z"].as_f64().unwrap_or(f64::NAN),
            id: v["id"].as_u64().unwrap() as usize,
        }
    }
    #[test]
    fn transparent_comparator_matches_original_helper_and_render_list() {
        let data = fixture();
        for pair in data["pairs"].as_array().unwrap() {
            let result = match compare(&key(&pair["a"]), &key(&pair["b"])) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            };
            assert_eq!(result, pair["result"].as_i64().unwrap());
        }
        for row in data["rows"].as_array().unwrap() {
            let mut records = row["records"]
                .as_array()
                .unwrap()
                .iter()
                .collect::<Vec<_>>();
            records.sort_by(|a, b| compare(&key(a), &key(b)));
            let names: Vec<_> = records
                .iter()
                .map(|r| r["name"].as_str().unwrap())
                .collect();
            assert_eq!(
                serde_json::json!(names),
                row["order"],
                "{}",
                row["input"]["name"]
            );
            // Source renderOrder=1 shells form the tail even when marker depth
            // is nearer; this is the production pass placement being preserved.
            assert!(names[..names.len() - 6]
                .iter()
                .all(|n| !n.starts_with("ice")));
            assert!(names[names.len() - 6..]
                .iter()
                .all(|n| n.starts_with("ice")));
        }
    }
    #[test]
    fn fixed_radius_iceberg_depths_and_pool_ties_match_original_scene() {
        let data = fixture();
        for row in data["rows"].as_array().unwrap() {
            let input = &row["input"];
            let bodies = input["positions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Ice {
                    pos: V::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap()),
                    z: v[2].as_f64().unwrap(),
                    radius: Some(32.),
                    ..Ice::default()
                })
                .collect();
            let removal = Removal {
                bodies,
                layer: input["layer"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|i| i.as_u64().unwrap() as usize)
                    .collect(),
                ..Removal::default()
            };
            let camera = V::new(
                input["camera"][0].as_f64().unwrap(),
                input["camera"][1].as_f64().unwrap(),
            );
            for (slot, body) in removal.bodies.iter().enumerate() {
                let name = format!("ice{slot}");
                let record = row["records"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["name"] == name)
                    .unwrap();
                let depth = iceberg_depth(body, camera);
                assert!(
                    (depth - record["z"].as_f64().unwrap()).abs() < 2e-15,
                    "{} slot{slot}: {depth} != {}",
                    input["name"],
                    record["z"]
                );
            }
            let ids = row["graphicIds"].as_array().unwrap();
            assert!(ids
                .windows(2)
                .all(|p| p[0].as_u64().unwrap() < p[1].as_u64().unwrap()));
            assert_eq!(
                serde_json::json!(iceberg_slots(&removal, camera)),
                row["iceOrder"],
                "{}",
                input["name"]
            );
        }
    }
}
