use std::collections::HashMap;

use flatgeom::{LineString2, MultiLineString2, MultiPoint2, MultiPolygon2, Polygon2};
use reearth_flow_types::{Attributes, Feature, GeometryType, GeometryValue};
use serde::{Deserialize, Serialize};
use tinymvt::webmercator::lnglat_to_web_mercator;

use super::tiling::TileContent;

pub(super) type TileKey = (u8, u32, u32);

/// Slicing resolution, as a power of two (4096).
const MAX_DETAIL: u32 = 12;
const BUFFER_PIXELS: u32 = 5;

/// A feature's geometry in normalized Web Mercator, as stored until slicing.
#[derive(Serialize, Deserialize)]
pub(super) enum Leaf {
    Polygon(Polygon2<'static>),
    LineString(LineString2<'static>),
    Point([f64; 2]),
}

/// A feature's part within one tile, in tile-local coordinates.
#[derive(Serialize, Deserialize)]
pub(super) enum SlicedGeom {
    Polygons(MultiPolygon2<'static>),
    LineStrings(MultiLineString2<'static>),
    Points(MultiPoint2<'static>),
}

pub(super) struct SlicedFeature {
    pub(super) typename: String,
    pub(super) geom: SlicedGeom,
    pub(super) properties: Attributes,
}

/// `feature`'s geometry projected to Web Mercator, with its lng/lat extent.
pub(super) fn extract(
    feature: &Feature,
) -> Result<(TileContent, Vec<Leaf>), crate::errors::SinkError> {
    let mut polygons: Vec<Polygon2<'static>> = Vec::new();
    let mut line_strings: Vec<LineString2<'static>> = Vec::new();
    let mut points = Vec::new();

    match &feature.geometry.value {
        GeometryValue::CityGmlGeometry(city_geometry) => {
            for entry in &city_geometry.gml_geometries {
                match entry.ty {
                    GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                        polygons.extend(entry.polygons.iter().map(|p| p.clone().into()));
                    }
                    GeometryType::Curve => {
                        line_strings.extend(entry.line_strings.iter().map(|l| l.clone().into()));
                    }
                    GeometryType::Point => unimplemented!(),
                }
            }
        }
        GeometryValue::FlowGeometry2D(flow_geometry) => {
            use reearth_flow_geometry::types::geometry::Geometry;

            match flow_geometry {
                Geometry::Polygon(poly) => polygons.push(poly.clone().into()),
                Geometry::MultiPolygon(multi_poly) => {
                    polygons.extend(multi_poly.iter().map(|p| p.clone().into()));
                }
                Geometry::LineString(line_string) => line_strings.push(line_string.clone().into()),
                Geometry::MultiLineString(multi_line_string) => {
                    line_strings.extend(multi_line_string.iter().map(|l| l.clone().into()));
                }
                Geometry::Point(point) => points.push([point.x(), point.y()]),
                Geometry::MultiPoint(multi_point) => {
                    points.extend(multi_point.0.iter().map(|p| [p.x(), p.y()]));
                }
                _ => {
                    // Other geometry types not supported for MVT
                }
            }
        }
        _ => {
            return Err(crate::errors::SinkError::MvtWriter(
                "Unsupported geometry type for MVT".to_string(),
            ));
        }
    }

    let mut content = TileContent::default();
    let mut leaves = Vec::new();
    for mut poly in polygons {
        for &[lng, lat] in poly.raw_coords() {
            content.extend(lng, lat);
        }
        poly.transform_inplace(|&[lng, lat]| {
            let (mx, my) = lnglat_to_web_mercator(lng, lat);
            [mx, my]
        });
        if poly.exterior().is_cw() {
            leaves.push(Leaf::Polygon(poly));
        }
    }
    for mut line_string in line_strings {
        for &[lng, lat] in line_string.raw_coords() {
            content.extend(lng, lat);
        }
        line_string.transform_inplace(|&[lng, lat]| {
            let (mx, my) = lnglat_to_web_mercator(lng, lat);
            [mx, my]
        });
        leaves.push(Leaf::LineString(line_string));
    }
    for [lng, lat] in points {
        content.extend(lng, lat);
        let (mx, my) = lnglat_to_web_mercator(lng, lat);
        leaves.push(Leaf::Point([mx, my]));
    }
    Ok((content, leaves))
}

/// Slice `leaves` into every tile they reach over `min_z..=max_z`, one part
/// per tile and geometry kind.
pub(super) fn slice(leaves: &[Leaf], min_z: u8, max_z: u8) -> Vec<(TileKey, SlicedGeom)> {
    let mut tiled_mpolys = HashMap::new();
    let mut tiled_line_strings = HashMap::new();
    let mut tiled_points = HashMap::new();

    let extent = 1 << MAX_DETAIL;
    let buffer = extent * BUFFER_PIXELS / 256;

    for leaf in leaves {
        match leaf {
            Leaf::Polygon(poly) => {
                let area = poly.area();
                for zoom in min_z..=max_z {
                    // Skip if the polygon is smaller than 4 square subpixels
                    //
                    // TODO: emulate the 'tiny-polygon-reduction' of tippecanoe
                    if area * (4u64.pow(zoom as u32 + MAX_DETAIL) as f64) < 4.0 {
                        continue;
                    }
                    slice_polygon(zoom, extent, buffer, poly, &mut tiled_mpolys);
                }
            }
            Leaf::LineString(line_string) => {
                for zoom in min_z..=max_z {
                    slice_line_string(zoom, extent, buffer, line_string, &mut tiled_line_strings);
                }
            }
            &Leaf::Point([mx, my]) => {
                for zoom in min_z..=max_z {
                    slice_point(zoom, mx, my, &mut tiled_points);
                }
            }
        }
    }

    let polygons = tiled_mpolys
        .into_iter()
        .filter(|(_, mpoly)| !mpoly.is_empty())
        .map(|(key, mpoly)| (key, SlicedGeom::Polygons(mpoly)));
    let line_strings = tiled_line_strings
        .into_iter()
        .filter(|(_, mline_string)| !mline_string.is_empty())
        .map(|(key, mline_string)| (key, SlicedGeom::LineStrings(mline_string)));
    let points = tiled_points
        .into_iter()
        .filter(|(_, mpoints)| !mpoints.is_empty())
        .map(|(key, mpoints)| (key, SlicedGeom::Points(mpoints)));
    polygons.chain(line_strings).chain(points).collect()
}

fn slice_polygon(
    zoom: u8,
    extent: u32,
    buffer: u32,
    poly: &Polygon2,
    out: &mut HashMap<TileKey, MultiPolygon2<'static>>,
) {
    let z_scale = (1 << zoom) as f64;
    let buf_width = buffer as f64 / extent as f64;
    let mut new_ring_buffer: Vec<[f64; 2]> = Vec::with_capacity(poly.exterior().len() + 1);

    // Slice along Y-axis
    let y_range = {
        let (min_y, max_y) = poly
            .exterior()
            .iter()
            .fold((f64::MAX, f64::MIN), |(min_y, max_y), c| {
                (min_y.min(c[1]), max_y.max(c[1]))
            });
        (min_y * z_scale).floor() as u32..(max_y * z_scale).ceil() as u32
    };

    let mut y_sliced_polys = Vec::with_capacity(y_range.len());

    for yi in y_range.clone() {
        let k1 = (yi as f64 - buf_width) / z_scale;
        let k2 = ((yi + 1) as f64 + buf_width) / z_scale;
        let mut y_sliced_poly = Polygon2::new();

        // todo?: check interior bbox to optimize

        for ring in poly.rings() {
            if ring.raw_coords().is_empty() {
                continue;
            }

            new_ring_buffer.clear();
            ring.iter_closed()
                .fold(None, |a, b| {
                    let Some(a) = a else { return Some(b) };

                    if a[1] < k1 {
                        if b[1] > k1 {
                            let x = (b[0] - a[0]) * (k1 - a[1]) / (b[1] - a[1]) + a[0];
                            new_ring_buffer.push([x, k1])
                        }
                    } else if a[1] > k2 {
                        if b[1] < k2 {
                            let x = (b[0] - a[0]) * (k2 - a[1]) / (b[1] - a[1]) + a[0];
                            new_ring_buffer.push([x, k2])
                        }
                    } else {
                        new_ring_buffer.push(a)
                    }

                    if b[1] < k1 && a[1] > k1 {
                        let x = (b[0] - a[0]) * (k1 - a[1]) / (b[1] - a[1]) + a[0];
                        new_ring_buffer.push([x, k1])
                    } else if b[1] > k2 && a[1] < k2 {
                        let x = (b[0] - a[0]) * (k2 - a[1]) / (b[1] - a[1]) + a[0];
                        new_ring_buffer.push([x, k2])
                    }

                    Some(b)
                })
                .unwrap();

            y_sliced_poly.add_ring(new_ring_buffer.iter().copied());
        }

        y_sliced_polys.push(y_sliced_poly);
    }

    let mut norm_coords_buf = Vec::new();

    // Slice along X-axis
    for (yi, y_sliced_poly) in y_range.zip(y_sliced_polys.iter()) {
        let x_range = {
            let (min_x, max_x) = y_sliced_poly
                .exterior()
                .iter()
                .fold((f64::MAX, f64::MIN), |(min_x, max_x), c| {
                    (min_x.min(c[0]), max_x.max(c[0]))
                });
            (min_x * z_scale).floor() as i32..(max_x * z_scale).ceil() as i32
        };

        for xi in x_range {
            let k1 = (xi as f64 - buf_width) / z_scale;
            let k2 = ((xi + 1) as f64 + buf_width) / z_scale;

            // todo?: check interior bbox to optimize ...

            let key = (
                zoom,
                xi.rem_euclid(1 << zoom) as u32, // handling geometry crossing the antimeridian
                yi,
            );
            let tile_mpoly = out.entry(key).or_default();

            for (ri, ring) in y_sliced_poly.rings().enumerate() {
                if ring.raw_coords().is_empty() {
                    continue;
                }

                new_ring_buffer.clear();
                ring.iter_closed()
                    .fold(None, |a, b| {
                        let Some(a) = a else { return Some(b) };

                        if a[0] < k1 {
                            if b[0] > k1 {
                                let y = (b[1] - a[1]) * (k1 - a[0]) / (b[0] - a[0]) + a[1];
                                new_ring_buffer.push([k1, y])
                            }
                        } else if a[0] > k2 {
                            if b[0] < k2 {
                                let y = (b[1] - a[1]) * (k2 - a[0]) / (b[0] - a[0]) + a[1];
                                new_ring_buffer.push([k2, y])
                            }
                        } else {
                            new_ring_buffer.push(a)
                        }

                        if b[0] < k1 && a[0] > k1 {
                            let y = (b[1] - a[1]) * (k1 - a[0]) / (b[0] - a[0]) + a[1];
                            new_ring_buffer.push([k1, y])
                        } else if b[0] > k2 && a[0] < k2 {
                            let y = (b[1] - a[1]) * (k2 - a[0]) / (b[0] - a[0]) + a[1];
                            new_ring_buffer.push([k2, y])
                        }

                        Some(b)
                    })
                    .unwrap();

                // get integer coordinates and simplify the ring
                {
                    norm_coords_buf.clear();
                    norm_coords_buf.extend(new_ring_buffer.iter().map(|&[x, y]| {
                        let tx = x * z_scale - xi as f64;
                        let ty = y * z_scale - yi as f64;
                        [tx, ty]
                    }));

                    // remove closing point if exists
                    if norm_coords_buf.len() >= 2
                        && norm_coords_buf[0] == *norm_coords_buf.last().unwrap()
                    {
                        norm_coords_buf.pop();
                    }

                    if norm_coords_buf.len() < 3 {
                        continue;
                    }
                }

                let mut ring = LineString2::from_raw(norm_coords_buf.clone().into());
                ring.reverse_inplace();

                match ri {
                    0 => tile_mpoly.add_exterior(ring.iter()),
                    _ => tile_mpoly.add_interior(ring.iter()),
                };
            }
        }
    }
}

fn slice_line_string(
    zoom: u8,
    extent: u32,
    buffer: u32,
    line_string: &LineString2,
    out: &mut HashMap<TileKey, MultiLineString2<'static>>,
) {
    let z_scale = (1 << zoom) as f64;
    let buf_width = buffer as f64 / extent as f64;
    let mut new_ring_buffer: Vec<[f64; 2]> = Vec::with_capacity(line_string.len() + 1);

    // Slice along Y-axis
    let y_range = {
        let (min_y, max_y) = line_string
            .iter()
            .fold((f64::MAX, f64::MIN), |(min_y, max_y), c| {
                (min_y.min(c[1]), max_y.max(c[1]))
            });
        (min_y * z_scale).floor() as u32..(max_y * z_scale).ceil() as u32
    };

    let mut y_sliced_line_strings = Vec::with_capacity(y_range.len());

    for yi in y_range.clone() {
        let k1 = (yi as f64 - buf_width) / z_scale;
        let k2 = ((yi + 1) as f64 + buf_width) / z_scale;
        let mut y_sliced_line_string = LineString2::new();

        // todo?: check interior bbox to optimize

        let last_coord = line_string
            .iter()
            .fold(None, |a, b| {
                let Some(a) = a else { return Some(b) };

                if a[1] < k1 {
                    if b[1] > k1 {
                        let x = (b[0] - a[0]) * (k1 - a[1]) / (b[1] - a[1]) + a[0];
                        y_sliced_line_string.push([x, k1])
                    }
                } else if a[1] > k2 {
                    if b[1] < k2 {
                        let x = (b[0] - a[0]) * (k2 - a[1]) / (b[1] - a[1]) + a[0];
                        y_sliced_line_string.push([x, k2])
                    }
                } else {
                    y_sliced_line_string.push(a)
                }

                if b[1] < k1 && a[1] > k1 {
                    let x = (b[0] - a[0]) * (k1 - a[1]) / (b[1] - a[1]) + a[0];
                    y_sliced_line_string.push([x, k1])
                } else if b[1] > k2 && a[1] < k2 {
                    let x = (b[0] - a[0]) * (k2 - a[1]) / (b[1] - a[1]) + a[0];
                    y_sliced_line_string.push([x, k2])
                }

                Some(b)
            })
            .unwrap();

        // Process the last coordinate that wasn't handled in the fold
        if last_coord[1] >= k1 && last_coord[1] <= k2 {
            y_sliced_line_string.push(last_coord);
        }

        y_sliced_line_strings.push(y_sliced_line_string);
    }

    // Slice along X-axis
    for (yi, y_sliced_line_string) in y_range.zip(y_sliced_line_strings.iter()) {
        if y_sliced_line_string.raw_coords().is_empty() {
            continue;
        }
        let x_range = {
            let (min_x, max_x) = y_sliced_line_string
                .iter()
                .fold((f64::MAX, f64::MIN), |(min_x, max_x), c| {
                    (min_x.min(c[0]), max_x.max(c[0]))
                });
            (min_x * z_scale).floor() as i32..(max_x * z_scale).ceil() as i32
        };

        let mut norm_coords_buf = Vec::new();
        for xi in x_range {
            let k1 = (xi as f64 - buf_width) / z_scale;
            let k2 = ((xi + 1) as f64 + buf_width) / z_scale;

            let key = (
                zoom,
                xi.rem_euclid(1 << zoom) as u32, // handling geometry crossing the antimeridian
                yi,
            );
            new_ring_buffer.clear();
            let last_coord = y_sliced_line_string
                .iter()
                .fold(None, |a, b| {
                    let Some(a) = a else { return Some(b) };

                    if a[0] < k1 {
                        if b[0] > k1 {
                            let y = (b[1] - a[1]) * (k1 - a[0]) / (b[0] - a[0]) + a[1];
                            new_ring_buffer.push([k1, y])
                        }
                    } else if a[0] > k2 {
                        if b[0] < k2 {
                            let y = (b[1] - a[1]) * (k2 - a[0]) / (b[0] - a[0]) + a[1];
                            new_ring_buffer.push([k2, y])
                        }
                    } else {
                        new_ring_buffer.push(a)
                    }

                    if b[0] < k1 && a[0] > k1 {
                        let y = (b[1] - a[1]) * (k1 - a[0]) / (b[0] - a[0]) + a[1];
                        new_ring_buffer.push([k1, y])
                    } else if b[0] > k2 && a[0] < k2 {
                        let y = (b[1] - a[1]) * (k2 - a[0]) / (b[0] - a[0]) + a[1];
                        new_ring_buffer.push([k2, y])
                    }

                    Some(b)
                })
                .unwrap();

            // Process the last coordinate that wasn't handled in the fold
            if last_coord[0] >= k1 && last_coord[0] <= k2 {
                new_ring_buffer.push(last_coord);
            }

            // get integer coordinates and simplify the ring
            {
                norm_coords_buf.clear();
                norm_coords_buf.extend(new_ring_buffer.iter().map(|&[x, y]| {
                    let tx = x * z_scale - xi as f64;
                    let ty = y * z_scale - yi as f64;
                    [tx, ty]
                }));

                // linestrings must have at least two points
                if norm_coords_buf.len() < 2 {
                    continue;
                }
            }

            let ring = LineString2::from_raw(norm_coords_buf.clone().into());
            let mline_string = out.entry(key).or_default();
            mline_string.add_linestring(ring.iter());
        }
    }
}

fn slice_point(zoom: u8, mx: f64, my: f64, out: &mut HashMap<TileKey, MultiPoint2<'static>>) {
    let z_scale = (1 << zoom) as f64;

    // Calculate tile coordinates for the point
    let xi = (mx * z_scale).floor() as i32;
    let yi = (my * z_scale).floor() as i32;

    let key = (
        zoom,
        xi.rem_euclid(1 << zoom) as u32, // handling geometry crossing the antimeridian
        yi as u32,
    );

    // Normalize coordinates relative to tile
    let tx = mx * z_scale - xi as f64;
    let ty = my * z_scale - yi as f64;

    let mpoint = out.entry(key).or_default();
    mpoint.push([tx, ty]);
}
