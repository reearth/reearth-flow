//! A minimal software depth-buffer renderer: camera + triangle soup in, a
//! grayscale depth `Canvas` out. Deliberately has no knowledge of glTF, 3D
//! Tiles, or any other source format — that's the job of whatever builds the
//! `[Vec3; 3]` triangle list and calls `input::recenter_and_cast` to get there
//! (see `tileset_mesh.rs`). Keeping this module format-agnostic makes it
//! testable on its own and reusable if a second triangle source shows up.

pub mod camera;
pub mod input;
pub mod rasterizer;
