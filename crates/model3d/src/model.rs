use crate::accessor::Source;
use crate::clip::{read_clips, Clip};
use crate::error::ModelError;
use crate::glb;
use crate::image_read::{read_images, Image};
use crate::mesh_read::{read_primitives, Primitive};
use crate::pose::globals;
use crate::skeleton_read::{read_nodes, read_skins, Node, Skin};
use crate::skin::{joint_matrices, skin_vertices};
use crate::state_map::{clip_time, ClipBinding, StateMap};

pub struct Model {
    pub(crate) nodes: Vec<Node>,
    pub(crate) skins: Vec<Skin>,
    pub(crate) primitives: Vec<Primitive>,
    pub clips: Vec<Clip>,
    pub images: Vec<Option<Image>>,
}

#[derive(Default)]
pub struct Posed {
    pub positions: Vec<Vec<[f32; 3]>>,
}

impl Model {
    pub fn from_glb(bytes: &[u8]) -> Result<Model, ModelError> {
        let parsed = glb::parse(bytes)?;
        let src = Source {
            json: &parsed.json,
            bin: &parsed.bin,
        };
        let primitives = read_primitives(&src)?;
        if primitives.is_empty() {
            return Err(ModelError::Missing("mesh primitives"));
        }
        Ok(Model {
            nodes: read_nodes(&parsed.json),
            skins: read_skins(&src)?,
            primitives,
            clips: read_clips(&src)?,
            images: read_images(&src),
        })
    }

    pub fn clip_index(&self, name: &str) -> Option<usize> {
        self.clips.iter().position(|c| c.name == name)
    }

    pub fn pose(&self, clip: Option<usize>, time: f32, out: &mut Posed) {
        let clip = clip.and_then(|c| self.clips.get(c));
        let world = globals(&self.nodes, clip, time);
        let joints: Vec<_> = self
            .skins
            .iter()
            .map(|s| joint_matrices(s, &world))
            .collect();
        out.positions.resize_with(self.primitives.len(), Vec::new);
        for (prim, slot) in self.primitives.iter().zip(out.positions.iter_mut()) {
            let skin = self
                .nodes
                .get(prim.node)
                .and_then(|n| n.skin)
                .and_then(|s| joints.get(s));
            match skin {
                Some(j) => skin_vertices(prim, j, slot),
                None => {
                    let m = world[prim.node];
                    slot.clear();
                    slot.extend(
                        prim.positions
                            .iter()
                            .map(|&p| crate::math::transform_point(&m, p)),
                    );
                }
            }
        }
    }

    pub fn pose_binding(&self, binding: &ClipBinding, ticks: i32, out: &mut Posed) {
        let clip = self.clip_index(&binding.clip);
        let duration = clip.map(|c| self.clips[c].duration).unwrap_or(0.0);
        self.pose(clip, clip_time(binding, ticks, duration), out);
    }

    pub fn pose_for_state(&self, map: &StateMap, stateno: i32, state_ticks: i32, out: &mut Posed) {
        self.pose_binding(map.resolve(stateno), state_ticks, out);
    }
}
