//! A minimal software depth-buffer renderer: camera + triangle soup in, a
//! grayscale depth `Canvas` out. Has no knowledge of glTF, 3D Tiles, or any
//! other source format.

pub mod camera;
pub mod input;
pub mod rasterizer;
