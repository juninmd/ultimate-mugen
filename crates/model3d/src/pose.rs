use crate::clip::{Clip, Path};
use crate::math::{mul, Mat4, IDENTITY};
use crate::skeleton_read::Node;

pub fn globals(nodes: &[Node], clip: Option<&Clip>, time: f32) -> Vec<Mat4> {
    let mut local: Vec<Node> = nodes.to_vec();
    if let Some(clip) = clip {
        for track in &clip.tracks {
            let Some(node) = local.get_mut(track.node) else {
                continue;
            };
            node.matrix = None;
            match track.path {
                Path::Translation => {
                    if let Some(v) = track.sample::<3>(time) {
                        node.translation = v
                    }
                }
                Path::Rotation => {
                    if let Some(v) = track.sample::<4>(time) {
                        node.rotation = v
                    }
                }
                Path::Scale => {
                    if let Some(v) = track.sample::<3>(time) {
                        node.scale = v
                    }
                }
            }
        }
    }
    let mut out = vec![IDENTITY; nodes.len()];
    let mut done = vec![false; nodes.len()];
    for index in 0..nodes.len() {
        resolve(index, &local, &mut out, &mut done);
    }
    out
}

fn resolve(index: usize, local: &[Node], out: &mut [Mat4], done: &mut [bool]) {
    if done[index] {
        return;
    }
    done[index] = true;
    let own = local[index].local();
    out[index] = match local[index].parent {
        Some(p) if p != index => {
            resolve(p, local, out, done);
            mul(&out[p], &own)
        }
        _ => own,
    };
}
