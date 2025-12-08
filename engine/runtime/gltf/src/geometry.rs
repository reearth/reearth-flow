use reearth_flow_geometry::types::{
    geometry::Geometry3D, multi_polygon::MultiPolygon3D, polygon::Polygon3D,
};

use crate::reader::{read_indices, read_positions, GltfReaderError};

/// Creates a MultiPolygon3D from GLTF primitives
pub fn create_multipolygon_from_primitives(
    primitives: &[gltf::Primitive],
    buffer_data: &[Vec<u8>],
) -> Result<MultiPolygon3D<f64>, GltfReaderError> {
    let mut polygons = Vec::new();

    for primitive in primitives {
        let position_accessor = primitive
            .get(&gltf::Semantic::Positions)
            .ok_or_else(|| GltfReaderError::Accessor("Primitive has no positions".to_string()))?;

        let positions = read_positions(&position_accessor, buffer_data)?;

        if let Some(indices_accessor) = primitive.indices() {
            let indices = read_indices(&indices_accessor, buffer_data)?;

            match primitive.mode() {
                gltf::mesh::Mode::Triangles => {
                    for chunk in indices.chunks(3) {
                        if chunk.len() == 3 {
                            let triangle = vec![
                                positions[chunk[0]],
                                positions[chunk[1]],
                                positions[chunk[2]],
                                positions[chunk[0]], // Close the ring
                            ];
                            polygons.push(Polygon3D::new(triangle.into(), vec![]));
                        }
                    }
                }
                gltf::mesh::Mode::TriangleStrip => {
                    for i in 0..indices.len().saturating_sub(2) {
                        let triangle = if i % 2 == 0 {
                            vec![
                                positions[indices[i]],
                                positions[indices[i + 1]],
                                positions[indices[i + 2]],
                                positions[indices[i]], // Close the ring
                            ]
                        } else {
                            vec![
                                positions[indices[i]],
                                positions[indices[i + 2]],
                                positions[indices[i + 1]],
                                positions[indices[i]], // Close the ring
                            ]
                        };
                        polygons.push(Polygon3D::new(triangle.into(), vec![]));
                    }
                }
                gltf::mesh::Mode::TriangleFan => {
                    for i in 1..indices.len().saturating_sub(1) {
                        let triangle = vec![
                            positions[indices[0]],
                            positions[indices[i]],
                            positions[indices[i + 1]],
                            positions[indices[0]], // Close the ring
                        ];
                        polygons.push(Polygon3D::new(triangle.into(), vec![]));
                    }
                }
                _ => {
                    return Err(GltfReaderError::Parse(format!(
                        "Unsupported primitive mode: {:?}",
                        primitive.mode()
                    )))
                }
            }
        } else {
            // Non-indexed primitives
            match primitive.mode() {
                gltf::mesh::Mode::Triangles => {
                    for chunk in positions.chunks(3) {
                        if chunk.len() == 3 {
                            let triangle = vec![chunk[0], chunk[1], chunk[2], chunk[0]];
                            polygons.push(Polygon3D::new(triangle.into(), vec![]));
                        }
                    }
                }
                _ => {
                    return Err(GltfReaderError::Parse(format!(
                        "Unsupported non-indexed primitive mode: {:?}",
                        primitive.mode()
                    )))
                }
            }
        }
    }

    Ok(MultiPolygon3D::new(polygons))
}

/// Creates a Geometry3D from GLTF primitives
/// Returns Polygon if single polygon, MultiPolygon otherwise
pub fn create_geometry_from_primitives(
    primitives: &[gltf::Primitive],
    buffer_data: &[Vec<u8>],
) -> Result<Geometry3D<f64>, GltfReaderError> {
    let multipolygon = create_multipolygon_from_primitives(primitives, buffer_data)?;
    let polygons = multipolygon.0;

    let geometry = if polygons.len() == 1 {
        Geometry3D::Polygon(polygons.into_iter().next().unwrap())
    } else {
        Geometry3D::MultiPolygon(MultiPolygon3D::new(polygons))
    };

    Ok(geometry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_gltf;
    use bytes::Bytes;

    #[test]
    fn test_parse_rectangle_glb() {
        // Minimal GLB with rectangle (0,0,0)-(1,1,0) as two triangles
        // Vertices: (0,0,0), (1,0,0), (1,1,0), (0,1,0)
        // Triangles: [0,1,2] and [0,2,3]
        let glb_base64 = "Z2xURgIAAAA4AgAA4AEAAEpTT057ImFzc2V0Ijp7InZlcnNpb24iOiIyLjAifSwic2NlbmUiOjAsInNjZW5lcyI6W3sibm9kZXMiOlswXX1dLCJub2RlcyI6W3sibWVzaCI6MH1dLCJtZXNoZXMiOlt7InByaW1pdGl2ZXMiOlt7ImF0dHJpYnV0ZXMiOnsiUE9TSVRJT04iOjB9LCJpbmRpY2VzIjoxLCJtb2RlIjo0fV19XSwiYWNjZXNzb3JzIjpbeyJidWZmZXJWaWV3IjowLCJjb21wb25lbnRUeXBlIjo1MTI2LCJjb3VudCI6NCwidHlwZSI6IlZFQzMiLCJtaW4iOlswLjAsMC4wLDAuMF0sIm1heCI6WzEuMCwxLjAsMC4wXX0seyJidWZmZXJWaWV3IjoxLCJjb21wb25lbnRUeXBlIjo1MTIzLCJjb3VudCI6NiwidHlwZSI6IlNDQUxBUiJ9XSwiYnVmZmVyVmlld3MiOlt7ImJ1ZmZlciI6MCwiYnl0ZU9mZnNldCI6MCwiYnl0ZUxlbmd0aCI6NDh9LHsiYnVmZmVyIjowLCJieXRlT2Zmc2V0Ijo0OCwiYnl0ZUxlbmd0aCI6MTJ9XSwiYnVmZmVycyI6W3siYnl0ZUxlbmd0aCI6NjB9XX0gICA8AAAAQklOAAAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAgD8AAIA/AAAAAAAAAAAAAIA/AAAAAAAAAQACAAAAAgADAA==";

        use base64::Engine;
        let glb_bytes = base64::engine::general_purpose::STANDARD
            .decode(glb_base64)
            .unwrap();
        let content = Bytes::from(glb_bytes);

        let gltf = parse_gltf(&content).unwrap();

        let buffer_data = vec![gltf.blob.as_ref().unwrap().clone()];

        let primitives: Vec<_> = gltf.meshes().next().unwrap().primitives().collect();

        let geometry = create_geometry_from_primitives(&primitives, &buffer_data).unwrap();

        // Should create MultiPolygon with 2 triangles
        match geometry {
            Geometry3D::MultiPolygon(mp) => {
                assert_eq!(mp.0.len(), 2, "Should have 2 triangular polygons");

                // First triangle: (0,0,0) -> (1,0,0) -> (1,1,0) -> (0,0,0)
                let poly1 = &mp.0[0];
                let coords1 = &poly1.exterior().0;
                assert_eq!(
                    coords1.len(),
                    4,
                    "Triangle should have 4 coords (closed ring)"
                );

                assert_eq!(coords1[0].x, 0.0);
                assert_eq!(coords1[0].y, 0.0);
                assert_eq!(coords1[0].z, 0.0);

                assert_eq!(coords1[1].x, 1.0);
                assert_eq!(coords1[1].y, 0.0);
                assert_eq!(coords1[1].z, 0.0);

                assert_eq!(coords1[2].x, 1.0);
                assert_eq!(coords1[2].y, 1.0);
                assert_eq!(coords1[2].z, 0.0);

                // Closed ring - back to first point
                assert_eq!(coords1[3].x, 0.0);
                assert_eq!(coords1[3].y, 0.0);
                assert_eq!(coords1[3].z, 0.0);

                // Second triangle: (0,0,0) -> (1,1,0) -> (0,1,0) -> (0,0,0)
                let poly2 = &mp.0[1];
                let coords2 = &poly2.exterior().0;
                assert_eq!(coords2.len(), 4);

                assert_eq!(coords2[0].x, 0.0);
                assert_eq!(coords2[0].y, 0.0);
                assert_eq!(coords2[0].z, 0.0);

                assert_eq!(coords2[1].x, 1.0);
                assert_eq!(coords2[1].y, 1.0);
                assert_eq!(coords2[1].z, 0.0);

                assert_eq!(coords2[2].x, 0.0);
                assert_eq!(coords2[2].y, 1.0);
                assert_eq!(coords2[2].z, 0.0);

                assert_eq!(coords2[3].x, 0.0);
                assert_eq!(coords2[3].y, 0.0);
                assert_eq!(coords2[3].z, 0.0);
            }
            _ => panic!("Expected MultiPolygon, got {:?}", geometry),
        }
    }
}
