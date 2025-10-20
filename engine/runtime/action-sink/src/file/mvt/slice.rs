use std::collections::HashMap;

use flatgeom::{LineString2, MultiLineString2, MultiPolygon2, Polygon2};
use indexmap::IndexMap;
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryType, GeometryValue};
use serde::{Deserialize, Serialize};
use tinymvt::webmercator::lnglat_to_web_mercator;

use super::tiling::TileContent;

pub type TileZXYName = (u8, u32, u32, String);

#[derive(Serialize, Deserialize)]
pub(super) struct SlicedFeature<'a> {
    pub(super) typename: String,
    pub(super) multi_polygons: MultiPolygon2<'a>,
    pub(super) multi_line_strings: MultiLineString2<'a>,
    pub(super) properties: IndexMap<Attribute, AttributeValue>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn slice_cityobj_geoms<'a>(
    feature: &Feature,
    layer_name: &str,
    min_z: u8,
    max_z: u8,
    max_detail: u32,
    buffer_pixels: u32,
    polygon_func: impl Fn(TileZXYName, MultiPolygon2<'a>) -> Result<(), crate::errors::SinkError>,
    line_string_func: impl Fn(TileZXYName, MultiLineString2<'a>) -> Result<(), crate::errors::SinkError>,
) -> Result<TileContent, crate::errors::SinkError> {
    let mut tiled_mpolys = HashMap::new();
    let mut tiled_line_strings = HashMap::new();

    let extent = 1 << max_detail;
    let buffer = extent * buffer_pixels / 256;
    let geometry = &feature.geometry;
    if geometry.is_empty() {
        return Err(crate::errors::SinkError::MvtWriter(
            "Feature does not have geometry".to_string(),
        ));
    };
    let GeometryValue::CityGmlGeometry(city_geometry) = &geometry.value else {
        return Err(crate::errors::SinkError::MvtWriter(
            "Feature does not have geometry value".to_string(),
        ));
    };

    let mut tile_content = TileContent::default();
    city_geometry
        .gml_geometries
        .iter()
        .for_each(|entry| match entry.ty {
            GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                for flow_poly in entry.polygons.iter() {
                    let idx_poly: Polygon2 = flow_poly.clone().into();
                    idx_poly.raw_coords().iter().for_each(|[lng, lat]| {
                        tile_content.min_lng = tile_content.min_lng.min(*lng);
                        tile_content.max_lng = tile_content.max_lng.max(*lng);
                        tile_content.min_lat = tile_content.min_lat.min(*lat);
                        tile_content.max_lat = tile_content.max_lat.max(*lat);
                    });
                    let poly = idx_poly.transform(|[lng, lat]| {
                        let (mx, my) = lnglat_to_web_mercator(*lng, *lat);
                        [mx, my]
                    });

                    if !poly.exterior().is_cw() {
                        continue;
                    }
                    let area = poly.area();

                    for zoom in min_z..=max_z {
                        // Skip if the polygon is smaller than 4 square subpixels
                        //
                        // TODO: emulate the 'tiny-polygon-reduction' of tippecanoe
                        if area * (4u64.pow(zoom as u32 + max_detail) as f64) < 4.0 {
                            continue;
                        }
                        slice_polygon(zoom, extent, buffer, &poly, layer_name, &mut tiled_mpolys);
                    }
                }
            }
            GeometryType::Curve => {
                for flow_line_string in entry.line_strings.iter() {
                    let idx_line_string: LineString2 = flow_line_string.clone().into();
                    idx_line_string.raw_coords().iter().for_each(|[lng, lat]| {
                        tile_content.min_lng = tile_content.min_lng.min(*lng);
                        tile_content.max_lng = tile_content.max_lng.max(*lng);
                        tile_content.min_lat = tile_content.min_lat.min(*lat);
                        tile_content.max_lat = tile_content.max_lat.max(*lat);
                    });
                    let line_string = idx_line_string.transform(|[lng, lat]| {
                        let (mx, my) = lnglat_to_web_mercator(*lng, *lat);
                        [mx, my]
                    });

                    for zoom in min_z..=max_z {
                        slice_line_string(
                            zoom,
                            extent,
                            buffer,
                            &line_string,
                            layer_name,
                            &mut tiled_line_strings,
                        );
                    }
                }
            }
            GeometryType::Point => {
                unimplemented!()
            }
        });

    for ((z, x, y, typename), mpoly) in tiled_mpolys {
        if mpoly.is_empty() {
            continue;
        }
        polygon_func((z, x, y, typename), mpoly)?;
    }

    for ((z, x, y, typename), mline_string) in tiled_line_strings {
        if mline_string.is_empty() {
            continue;
        }
        line_string_func((z, x, y, typename), mline_string)?;
    }
    Ok(tile_content)

    // TODO: linestring, point
}

fn slice_polygon(
    zoom: u8,
    extent: u32,
    buffer: u32,
    poly: &Polygon2,
    typename: &str,
    out: &mut HashMap<(u8, u32, u32, String), MultiPolygon2>,
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
                            // let z = (b[2] - a[2]) * (k1 - a[1]) / (b[1] - a[1]) + a[2];
                            new_ring_buffer.push([x, k1])
                        }
                    } else if a[1] > k2 {
                        if b[1] < k2 {
                            let x = (b[0] - a[0]) * (k2 - a[1]) / (b[1] - a[1]) + a[0];
                            // let z = (b[2] - a[2]) * (k2 - a[1]) / (b[1] - a[1]) + a[2];
                            new_ring_buffer.push([x, k2])
                        }
                    } else {
                        new_ring_buffer.push(a)
                    }

                    if b[1] < k1 && a[1] > k1 {
                        let x = (b[0] - a[0]) * (k1 - a[1]) / (b[1] - a[1]) + a[0];
                        // let z = (b[2] - a[2]) * (k1 - a[1]) / (b[1] - a[1]) + a[2];
                        new_ring_buffer.push([x, k1])
                    } else if b[1] > k2 && a[1] < k2 {
                        let x = (b[0] - a[0]) * (k2 - a[1]) / (b[1] - a[1]) + a[0];
                        // let z = (b[2] - a[2]) * (k2 - a[1]) / (b[1] - a[1]) + a[2];
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
                typename.to_string(),
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
                                // let z = (b[2] - a[2]) * (k1 - a[0]) / (b[0] - a[0]) + a[2];
                                new_ring_buffer.push([k1, y])
                            }
                        } else if a[0] > k2 {
                            if b[0] < k2 {
                                let y = (b[1] - a[1]) * (k2 - a[0]) / (b[0] - a[0]) + a[1];
                                // let z = (b[2] - a[2]) * (k2 - a[0]) / (b[0] - a[0]) + a[2];
                                new_ring_buffer.push([k2, y])
                            }
                        } else {
                            new_ring_buffer.push(a)
                        }

                        if b[0] < k1 && a[0] > k1 {
                            let y = (b[1] - a[1]) * (k1 - a[0]) / (b[0] - a[0]) + a[1];
                            // let z = (b[2] - a[2]) * (k1 - a[0]) / (b[0] - a[0]) + a[2];
                            new_ring_buffer.push([k1, y])
                        } else if b[0] > k2 && a[0] < k2 {
                            let y = (b[1] - a[1]) * (k2 - a[0]) / (b[0] - a[0]) + a[1];
                            // let z = (b[2] - a[2]) * (k2 - a[0]) / (b[0] - a[0]) + a[2];
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
    typename: &str,
    out: &mut HashMap<(u8, u32, u32, String), MultiLineString2>,
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

        line_string
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
                typename.to_string(),
            );
            new_ring_buffer.clear();
            y_sliced_line_string
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
