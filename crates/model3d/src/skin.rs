use crate::math::{mul, transform_point, Mat4};
use crate::mesh_read::Primitive;
use crate::skeleton_read::Skin;

pub fn joint_matrices(skin: &Skin, globals: &[Mat4]) -> Vec<Mat4> {
    skin.joints
        .iter()
        .zip(&skin.inverse_bind)
        .map(|(&j, ibm)| match globals.get(j) {
            Some(g) => mul(g, ibm),
            None => *ibm,
        })
        .collect()
}

pub fn skin_vertices(prim: &Primitive, joints: &[Mat4], out: &mut Vec<[f32; 3]>) {
    out.clear();
    for (i, &p) in prim.positions.iter().enumerate() {
        let (Some(js), Some(ws)) = (prim.joints.get(i), prim.weights.get(i)) else {
            out.push(p);
            continue;
        };
        let mut acc = [0.0f32; 3];
        let mut total = 0.0;
        for lane in 0..4 {
            let w = ws[lane];
            let Some(m) = joints.get(js[lane] as usize) else {
                continue;
            };
            if w <= 0.0 {
                continue;
            }
            let q = transform_point(m, p);
            for k in 0..3 {
                acc[k] += q[k] * w;
            }
            total += w;
        }
        out.push(if total > 0.0 {
            acc.map(|v| v / total)
        } else {
            p
        });
    }
}
