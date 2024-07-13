/*
This Source Code Form is subject to the terms of the Mozilla Public
Copyright 2019 Easymov Robotics
Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted, provided that the above copyright notice and this permission notice appear in all copies.
THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

use clipper_sys::{
    clean, execute, free_polygons, offset, offset_simplify_clean, simplify, ClipType,
    ClipType_ctDifference, ClipType_ctIntersection, ClipType_ctUnion, ClipType_ctXor,
    EndType as ClipperEndType, EndType_etClosedLine, EndType_etClosedPolygon, EndType_etOpenButt,
    EndType_etOpenRound, EndType_etOpenSquare, JoinType as ClipperJoinType, JoinType_jtMiter,
    JoinType_jtRound, JoinType_jtSquare, Path, PolyFillType as ClipperPolyFillType,
    PolyFillType_pftEvenOdd, PolyFillType_pftNegative, PolyFillType_pftNonZero,
    PolyFillType_pftPositive, PolyType, PolyType_ptClip, PolyType_ptSubject,
    Polygon as ClipperPolygon, Polygons, Vertice,
};

use crate::types::{
    coordinate::{Coordinate2D, Coordinate3D},
    coordnum::{CoordFloat, CoordFloatT, CoordNum, CoordNumT},
    line_string::{LineString2D, LineString3D},
    multi_line_string::{MultiLineString2D, MultiLineString3D},
    multi_polygon::{MultiPolygon2D, MultiPolygon3D},
    polygon::{Polygon2D, Polygon3D},
};

#[derive(Clone, Copy)]
pub enum JoinType {
    Square,
    Round(f64),
    Miter(f64),
}

#[derive(Clone, Copy)]
pub enum EndType {
    ClosedPolygon,
    ClosedLine,
    OpenButt,
    OpenSquare,
    OpenRound(f64),
}

#[derive(Clone, Copy)]
pub enum PolyFillType {
    EvenOdd,
    NonZero,
    Positive,
    Negative,
}

impl From<JoinType> for ClipperJoinType {
    fn from(jt: JoinType) -> Self {
        match jt {
            JoinType::Square => JoinType_jtSquare,
            JoinType::Round(_) => JoinType_jtRound,
            JoinType::Miter(_) => JoinType_jtMiter,
        }
    }
}

impl From<EndType> for ClipperEndType {
    fn from(et: EndType) -> Self {
        match et {
            EndType::ClosedPolygon => EndType_etClosedPolygon,
            EndType::ClosedLine => EndType_etClosedLine,
            EndType::OpenButt => EndType_etOpenButt,
            EndType::OpenSquare => EndType_etOpenSquare,
            EndType::OpenRound(_) => EndType_etOpenRound,
        }
    }
}

impl From<PolyFillType> for ClipperPolyFillType {
    fn from(pft: PolyFillType) -> Self {
        match pft {
            PolyFillType::EvenOdd => PolyFillType_pftEvenOdd,
            PolyFillType::NonZero => PolyFillType_pftNonZero,
            PolyFillType::Positive => PolyFillType_pftPositive,
            PolyFillType::Negative => PolyFillType_pftNegative,
        }
    }
}

struct ClipperPolygons<F: CoordFloat> {
    pub polygons: Polygons,
    pub factor: F,
}

struct ClipperPath<F: CoordFloat> {
    pub path: Path,
    pub factor: F,
}

impl<F: CoordFloat> From<ClipperPolygons<F>> for MultiPolygon2D<F> {
    fn from(polygons: ClipperPolygons<F>) -> Self {
        polygons
            .polygons
            .polygons()
            .iter()
            .filter_map(|polygon| {
                let paths = polygon.paths();
                Some(Polygon2D::new(
                    ClipperPath {
                        path: *paths.first()?,
                        factor: polygons.factor,
                    }
                    .into(),
                    paths
                        .iter()
                        .skip(1)
                        .map(|path| {
                            ClipperPath {
                                path: *path,
                                factor: polygons.factor,
                            }
                            .into()
                        })
                        .collect(),
                ))
            })
            .collect()
    }
}

impl<F: CoordFloatT> From<ClipperPolygons<F>> for MultiPolygon3D<F> {
    fn from(polygons: ClipperPolygons<F>) -> Self {
        polygons
            .polygons
            .polygons()
            .iter()
            .filter_map(|polygon| {
                let paths = polygon.paths();
                Some(Polygon3D::new(
                    ClipperPath {
                        path: *paths.first()?,
                        factor: polygons.factor,
                    }
                    .into(),
                    paths
                        .iter()
                        .skip(1)
                        .map(|path| {
                            ClipperPath {
                                path: *path,
                                factor: polygons.factor,
                            }
                            .into()
                        })
                        .collect(),
                ))
            })
            .collect()
    }
}

impl<F: CoordFloat> From<ClipperPolygons<F>> for MultiLineString2D<F> {
    fn from(polygons: ClipperPolygons<F>) -> Self {
        MultiLineString2D::new(
            polygons
                .polygons
                .polygons()
                .iter()
                .flat_map(|polygon| {
                    polygon.paths().iter().map(|path| {
                        ClipperPath {
                            path: *path,
                            factor: polygons.factor,
                        }
                        .into()
                    })
                })
                .collect(),
        )
    }
}

impl<F: CoordFloatT> From<ClipperPolygons<F>> for MultiLineString3D<F> {
    fn from(polygons: ClipperPolygons<F>) -> Self {
        MultiLineString3D::new(
            polygons
                .polygons
                .polygons()
                .iter()
                .flat_map(|polygon| {
                    polygon.paths().iter().map(|path| {
                        ClipperPath {
                            path: *path,
                            factor: polygons.factor,
                        }
                        .into()
                    })
                })
                .collect(),
        )
    }
}

impl<F: CoordFloat> From<ClipperPath<F>> for LineString2D<F> {
    fn from(path: ClipperPath<F>) -> Self {
        path.path
            .vertices()
            .iter()
            .map(|vertice| {
                Coordinate2D::new_(
                    F::from(vertice[0]).unwrap() / path.factor,
                    F::from(vertice[1]).unwrap() / path.factor,
                )
            })
            .collect()
    }
}

impl<F: CoordFloatT> From<ClipperPath<F>> for LineString3D<F> {
    fn from(path: ClipperPath<F>) -> Self {
        path.path
            .vertices()
            .iter()
            .map(|vertice| {
                Coordinate3D::new__(
                    F::from(vertice[0]).unwrap() / path.factor,
                    F::from(vertice[1]).unwrap() / path.factor,
                    F::zero(),
                )
            })
            .collect()
    }
}

/// Marker trait to signify a type as an open path type
pub trait OpenPath {}
/// Marker trait to signify a type as an closed polygon type
pub trait ClosedPoly {}

impl<T: CoordNum> OpenPath for MultiLineString2D<T> {}
impl<T: CoordNumT> OpenPath for MultiLineString3D<T> {}
impl<T: CoordNum> OpenPath for LineString2D<T> {}
impl<T: CoordNumT> OpenPath for LineString3D<T> {}
impl<T: CoordNum> ClosedPoly for MultiPolygon2D<T> {}
impl<T: CoordNumT> ClosedPoly for MultiPolygon3D<T> {}
impl<T: CoordNum> ClosedPoly for Polygon2D<T> {}
impl<T: CoordNumT> ClosedPoly for Polygon3D<T> {}

pub struct OwnedPolygon {
    polygons: Vec<ClipperPolygon>,
    paths: Vec<Vec<Path>>,
    vertices: Vec<Vec<Vec<Vertice>>>,
}

pub trait ToOwnedPolygon<F: CoordFloat = f64> {
    fn to_polygon_owned(&self, poly_type: PolyType, factor: F) -> OwnedPolygon;
}

impl<F: CoordFloat> ToOwnedPolygon<F> for MultiPolygon2D<F> {
    fn to_polygon_owned(&self, poly_type: PolyType, factor: F) -> OwnedPolygon {
        OwnedPolygon {
            polygons: Vec::with_capacity(self.0.len()),
            paths: Vec::with_capacity(self.0.len()),
            vertices: Vec::with_capacity(self.0.len()),
        }
        .add_polygons2d(self, poly_type, factor)
    }
}

impl<F: CoordFloatT> ToOwnedPolygon<F> for MultiPolygon3D<F> {
    fn to_polygon_owned(&self, poly_type: PolyType, factor: F) -> OwnedPolygon {
        OwnedPolygon {
            polygons: Vec::with_capacity(self.0.len()),
            paths: Vec::with_capacity(self.0.len()),
            vertices: Vec::with_capacity(self.0.len()),
        }
        .add_polygons3d(self, poly_type, factor)
    }
}

impl<F: CoordFloat> ToOwnedPolygon<F> for Polygon2D<F> {
    fn to_polygon_owned(&self, poly_type: PolyType, factor: F) -> OwnedPolygon {
        OwnedPolygon {
            polygons: Vec::with_capacity(1),
            paths: Vec::with_capacity(1),
            vertices: Vec::with_capacity(1),
        }
        .add_polygon2d(self, poly_type, factor)
    }
}

impl<F: CoordFloatT> ToOwnedPolygon<F> for Polygon3D<F> {
    fn to_polygon_owned(&self, poly_type: PolyType, factor: F) -> OwnedPolygon {
        OwnedPolygon {
            polygons: Vec::with_capacity(1),
            paths: Vec::with_capacity(1),
            vertices: Vec::with_capacity(1),
        }
        .add_polygon3d(self, poly_type, factor)
    }
}

impl<F: CoordFloat> ToOwnedPolygon<F> for MultiLineString2D<F> {
    fn to_polygon_owned(&self, poly_type: PolyType, factor: F) -> OwnedPolygon {
        OwnedPolygon {
            polygons: Vec::with_capacity(self.0.len()),
            paths: Vec::with_capacity(self.0.len()),
            vertices: Vec::with_capacity(self.0.len()),
        }
        .add_line_strings2d(self, poly_type, factor)
    }
}

impl<F: CoordFloatT> ToOwnedPolygon<F> for MultiLineString3D<F> {
    fn to_polygon_owned(&self, poly_type: PolyType, factor: F) -> OwnedPolygon {
        OwnedPolygon {
            polygons: Vec::with_capacity(self.0.len()),
            paths: Vec::with_capacity(self.0.len()),
            vertices: Vec::with_capacity(self.0.len()),
        }
        .add_line_strings3d(self, poly_type, factor)
    }
}

impl OwnedPolygon {
    pub fn get_clipper_polygons(&mut self) -> &Vec<ClipperPolygon> {
        for (polygon, (paths, paths_vertices)) in self
            .polygons
            .iter_mut()
            .zip(self.paths.iter_mut().zip(self.vertices.iter_mut()))
        {
            for (path, vertices) in paths.iter_mut().zip(paths_vertices.iter_mut()) {
                path.vertices = vertices.as_mut_ptr();
                path.vertices_count = vertices.len();
            }

            polygon.paths = paths.as_mut_ptr();
            polygon.paths_count = paths.len();
        }
        &self.polygons
    }

    fn add_polygon2d<F: CoordFloat>(
        mut self,
        polygon: &Polygon2D<F>,
        poly_type: PolyType,
        factor: F,
    ) -> Self {
        let path_count = polygon.interiors().len() + 1;
        self.paths.push(Vec::with_capacity(path_count));
        self.vertices.push(Vec::with_capacity(path_count));
        let last_path = self.paths.last_mut().unwrap();
        let last_path_vertices = self.vertices.last_mut().unwrap();

        for line_string in std::iter::once(polygon.exterior()).chain(polygon.interiors().iter()) {
            last_path_vertices.push(Vec::with_capacity(line_string.0.len().saturating_sub(1)));
            let last_vertices = last_path_vertices.last_mut().unwrap();

            for coordinate in line_string.0.iter().skip(1) {
                last_vertices.push([
                    (coordinate.x * factor).to_i64().unwrap(),
                    (coordinate.y * factor).to_i64().unwrap(),
                ]);
            }

            last_path.push(Path {
                vertices: std::ptr::null_mut(),
                vertices_count: 0,
                closed: 1,
            });
        }

        self.polygons.push(ClipperPolygon {
            paths: std::ptr::null_mut(),
            paths_count: 0,
            type_: poly_type,
        });

        self
    }

    fn add_polygon3d<F: CoordFloatT>(
        mut self,
        polygon: &Polygon3D<F>,
        poly_type: PolyType,
        factor: F,
    ) -> Self {
        let path_count = polygon.interiors().len() + 1;
        self.paths.push(Vec::with_capacity(path_count));
        self.vertices.push(Vec::with_capacity(path_count));
        let last_path = self.paths.last_mut().unwrap();
        let last_path_vertices = self.vertices.last_mut().unwrap();

        for line_string in std::iter::once(polygon.exterior()).chain(polygon.interiors().iter()) {
            last_path_vertices.push(Vec::with_capacity(line_string.0.len().saturating_sub(1)));
            let last_vertices = last_path_vertices.last_mut().unwrap();

            for coordinate in line_string.0.iter().skip(1) {
                last_vertices.push([
                    (coordinate.x * factor).to_i64().unwrap(),
                    (coordinate.y * factor).to_i64().unwrap(),
                ]);
            }

            last_path.push(Path {
                vertices: std::ptr::null_mut(),
                vertices_count: 0,
                closed: 1,
            });
        }

        self.polygons.push(ClipperPolygon {
            paths: std::ptr::null_mut(),
            paths_count: 0,
            type_: poly_type,
        });

        self
    }

    fn add_line_strings2d<F: CoordFloat>(
        mut self,
        line_strings: &MultiLineString2D<F>,
        poly_type: PolyType,
        factor: F,
    ) -> Self {
        let path_count = line_strings.0.len();
        self.paths.push(Vec::with_capacity(path_count));
        self.vertices.push(Vec::with_capacity(path_count));
        let last_path = self.paths.last_mut().unwrap();
        let last_path_vertices = self.vertices.last_mut().unwrap();

        for line_string in line_strings.0.iter() {
            last_path_vertices.push(Vec::with_capacity(line_string.0.len().saturating_sub(1)));
            let last_vertices = last_path_vertices.last_mut().unwrap();

            for coordinate in line_string.0.iter() {
                last_vertices.push([
                    (coordinate.x * factor).to_i64().unwrap(),
                    (coordinate.y * factor).to_i64().unwrap(),
                ]);
            }

            last_path.push(Path {
                vertices: std::ptr::null_mut(),
                vertices_count: 0,
                closed: 0,
            });
        }

        self.polygons.push(ClipperPolygon {
            paths: std::ptr::null_mut(),
            paths_count: 0,
            type_: poly_type,
        });

        self
    }

    fn add_line_strings3d<F: CoordFloatT>(
        mut self,
        line_strings: &MultiLineString3D<F>,
        poly_type: PolyType,
        factor: F,
    ) -> Self {
        let path_count = line_strings.0.len();
        self.paths.push(Vec::with_capacity(path_count));
        self.vertices.push(Vec::with_capacity(path_count));
        let last_path = self.paths.last_mut().unwrap();
        let last_path_vertices = self.vertices.last_mut().unwrap();

        for line_string in line_strings.0.iter() {
            last_path_vertices.push(Vec::with_capacity(line_string.0.len().saturating_sub(1)));
            let last_vertices = last_path_vertices.last_mut().unwrap();

            for coordinate in line_string.0.iter() {
                last_vertices.push([
                    (coordinate.x * factor).to_i64().unwrap(),
                    (coordinate.y * factor).to_i64().unwrap(),
                ]);
            }

            last_path.push(Path {
                vertices: std::ptr::null_mut(),
                vertices_count: 0,
                closed: 0,
            });
        }

        self.polygons.push(ClipperPolygon {
            paths: std::ptr::null_mut(),
            paths_count: 0,
            type_: poly_type,
        });

        self
    }

    fn add_polygons2d<F: CoordFloat>(
        self,
        polygon: &MultiPolygon2D<F>,
        poly_type: PolyType,
        factor: F,
    ) -> Self {
        polygon.0.iter().fold(self, |polygons, polygon| {
            polygons.add_polygon2d(polygon, poly_type, factor)
        })
    }

    fn add_polygons3d<F: CoordFloatT>(
        self,
        polygon: &MultiPolygon3D<F>,
        poly_type: PolyType,
        factor: F,
    ) -> Self {
        polygon.0.iter().fold(self, |polygons, polygon| {
            polygons.add_polygon3d(polygon, poly_type, factor)
        })
    }
}

fn execute_offset_operation2d<F: CoordFloat, T: ToOwnedPolygon<F> + ?Sized>(
    polygons: &T,
    delta: F,
    jt: JoinType,
    et: EndType,
    factor: F,
) -> MultiPolygon2D<F> {
    let miter_limit = match jt {
        JoinType::Miter(limit) => limit,
        _ => 0.0,
    };

    let round_precision = match jt {
        JoinType::Round(precision) => precision,
        _ => match et {
            EndType::OpenRound(precision) => precision,
            _ => 0.0,
        },
    };

    let mut owned = polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut get_clipper = owned.get_clipper_polygons().clone();
    let clipper_polygons = Polygons {
        polygons: get_clipper.as_mut_ptr(),
        polygons_count: get_clipper.len(),
    };
    let solution = unsafe {
        offset(
            miter_limit,
            round_precision,
            jt.into(),
            et.into(),
            clipper_polygons,
            delta.to_f64().unwrap(),
        )
    };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

fn execute_offset_operation3d<F: CoordFloatT, T: ToOwnedPolygon<F> + ?Sized>(
    polygons: &T,
    delta: F,
    jt: JoinType,
    et: EndType,
    factor: F,
) -> MultiPolygon3D<F> {
    let miter_limit = match jt {
        JoinType::Miter(limit) => limit,
        _ => 0.0,
    };

    let round_precision = match jt {
        JoinType::Round(precision) => precision,
        _ => match et {
            EndType::OpenRound(precision) => precision,
            _ => 0.0,
        },
    };

    let mut owned = polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut get_clipper = owned.get_clipper_polygons().clone();
    let clipper_polygons = Polygons {
        polygons: get_clipper.as_mut_ptr(),
        polygons_count: get_clipper.len(),
    };
    let solution = unsafe {
        offset(
            miter_limit,
            round_precision,
            jt.into(),
            et.into(),
            clipper_polygons,
            delta.to_f64().unwrap(),
        )
    };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

fn execute_offset_simplify_clean_operation2d<F: CoordFloat, T: ToOwnedPolygon<F> + ?Sized>(
    polygons: &T,
    delta: F,
    jt: JoinType,
    et: EndType,
    pft: PolyFillType,
    distance: F,
    factor: F,
) -> MultiLineString2D<F> {
    let miter_limit = match jt {
        JoinType::Miter(limit) => limit,
        _ => 0.0,
    };

    let round_precision = match jt {
        JoinType::Round(precision) => precision,
        _ => match et {
            EndType::OpenRound(precision) => precision,
            _ => 0.0,
        },
    };

    let mut owned = polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut get_clipper = owned.get_clipper_polygons().clone();
    let clipper_polygons = Polygons {
        polygons: get_clipper.as_mut_ptr(),
        polygons_count: get_clipper.len(),
    };
    let solution = unsafe {
        offset_simplify_clean(
            clipper_polygons,
            miter_limit,
            round_precision,
            jt.into(),
            et.into(),
            delta.to_f64().unwrap(),
            pft.into(),
            distance.to_f64().unwrap(),
        )
    };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

fn execute_offset_simplify_clean_operation3d<F: CoordFloatT, T: ToOwnedPolygon<F> + ?Sized>(
    polygons: &T,
    delta: F,
    jt: JoinType,
    et: EndType,
    pft: PolyFillType,
    distance: F,
    factor: F,
) -> MultiLineString3D<F> {
    let miter_limit = match jt {
        JoinType::Miter(limit) => limit,
        _ => 0.0,
    };

    let round_precision = match jt {
        JoinType::Round(precision) => precision,
        _ => match et {
            EndType::OpenRound(precision) => precision,
            _ => 0.0,
        },
    };

    let mut owned = polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut get_clipper = owned.get_clipper_polygons().clone();
    let clipper_polygons = Polygons {
        polygons: get_clipper.as_mut_ptr(),
        polygons_count: get_clipper.len(),
    };
    let solution = unsafe {
        offset_simplify_clean(
            clipper_polygons,
            miter_limit,
            round_precision,
            jt.into(),
            et.into(),
            delta.to_f64().unwrap(),
            pft.into(),
            distance.to_f64().unwrap(),
        )
    };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

fn execute_boolean_operation<
    F: CoordFloat,
    T: ToOwnedPolygon<F> + ?Sized,
    U: ToOwnedPolygon<F> + ?Sized,
    R: From<ClipperPolygons<F>>,
>(
    clip_type: ClipType,
    subject_polygons: &T,
    clip_polygons: &U,
    factor: F,
) -> R {
    let mut subject_owned = subject_polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut clip_owned = clip_polygons.to_polygon_owned(PolyType_ptClip, factor);
    let mut polygons: Vec<ClipperPolygon> = subject_owned
        .get_clipper_polygons()
        .iter()
        .chain(clip_owned.get_clipper_polygons().iter())
        .cloned()
        .collect();
    let clipper_polygons = Polygons {
        polygons: polygons.as_mut_ptr(),
        polygons_count: polygons.len(),
    };

    let solution = unsafe {
        execute(
            clip_type,
            clipper_polygons,
            PolyFillType_pftNonZero,
            PolyFillType_pftNonZero,
        )
    };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

fn execute_simplify_operation2d<F: CoordFloat, T: ToOwnedPolygon<F> + ?Sized>(
    polygons: &T,
    pft: PolyFillType,
    factor: F,
) -> MultiLineString2D<F> {
    let mut owned = polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut get_clipper = owned.get_clipper_polygons().clone();
    let clipper_polygons = Polygons {
        polygons: get_clipper.as_mut_ptr(),
        polygons_count: get_clipper.len(),
    };
    let solution = unsafe { simplify(clipper_polygons, pft.into()) };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

fn execute_simplify_operation3d<F: CoordFloatT, T: ToOwnedPolygon<F> + ?Sized>(
    polygons: &T,
    pft: PolyFillType,
    factor: F,
) -> MultiLineString3D<F> {
    let mut owned = polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut get_clipper = owned.get_clipper_polygons().clone();
    let clipper_polygons = Polygons {
        polygons: get_clipper.as_mut_ptr(),
        polygons_count: get_clipper.len(),
    };
    let solution = unsafe { simplify(clipper_polygons, pft.into()) };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

fn execute_clean_operation2d<F: CoordFloat, T: ToOwnedPolygon<F> + ?Sized>(
    polygons: &T,
    distance: F,
    factor: F,
) -> MultiLineString2D<F> {
    let mut owned = polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut get_clipper = owned.get_clipper_polygons().clone();
    let clipper_polygons = Polygons {
        polygons: get_clipper.as_mut_ptr(),
        polygons_count: get_clipper.len(),
    };
    let solution = unsafe { clean(clipper_polygons, distance.to_f64().unwrap()) };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

fn execute_clean_operation3d<F: CoordFloatT, T: ToOwnedPolygon<F> + ?Sized>(
    polygons: &T,
    distance: F,
    factor: F,
) -> MultiLineString3D<F> {
    let mut owned = polygons.to_polygon_owned(PolyType_ptSubject, factor);
    let mut get_clipper = owned.get_clipper_polygons().clone();
    let clipper_polygons = Polygons {
        polygons: get_clipper.as_mut_ptr(),
        polygons_count: get_clipper.len(),
    };
    let solution = unsafe { clean(clipper_polygons, distance.to_f64().unwrap()) };

    let result = ClipperPolygons {
        polygons: solution,
        factor,
    }
    .into();
    unsafe {
        free_polygons(solution);
    }
    result
}

/// This trait defines the boolean and offset operations on polygons
///
/// The `factor` parameter in its methods is used to scale shapes before and after applying the operation
/// to avoid precision loss since Clipper (the underlaying library) performs integer computation.
pub trait Clipper2D<F: CoordFloat = f64> {
    fn difference2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon2D<F>;
    fn intersection2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon2D<F>;
    fn union2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon2D<F>;
    fn xor2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon2D<F>;
    fn offset2d(
        &self,
        delta: F,
        join_type: JoinType,
        end_type: EndType,
        factor: F,
    ) -> MultiPolygon2D<F>;
    fn offset_simplify_clean2d(
        &self,
        delta: F,
        jt: JoinType,
        et: EndType,
        pft: PolyFillType,
        distance: F,
        factor: F,
    ) -> MultiLineString2D<F>;
    fn simplify2d(&self, fill_type: PolyFillType, factor: F) -> MultiLineString2D<F>;
    fn clean2d(&self, distance: F, factor: F) -> MultiLineString2D<F>;
}

pub trait Clipper3D<F: CoordFloatT = f64> {
    fn difference3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon3D<F>;
    fn intersection3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon3D<F>;
    fn union3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon3D<F>;
    fn xor3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon3D<F>;
    fn offset3d(
        &self,
        delta: F,
        join_type: JoinType,
        end_type: EndType,
        factor: F,
    ) -> MultiPolygon3D<F>;
    fn offset_simplify_clean3d(
        &self,
        delta: F,
        jt: JoinType,
        et: EndType,
        pft: PolyFillType,
        distance: F,
        factor: F,
    ) -> MultiLineString3D<F>;
    fn simplify3d(&self, fill_type: PolyFillType, factor: F) -> MultiLineString3D<F>;
    fn clean3d(&self, distance: F, factor: F) -> MultiLineString3D<F>;
}

pub trait ClipperOpen2D<F: CoordFloat = f64> {
    fn difference2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiLineString2D<F>;
    fn intersection2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiLineString2D<F>;
    fn offset2d(
        &self,
        delta: F,
        join_type: JoinType,
        end_type: EndType,
        factor: F,
    ) -> MultiPolygon2D<F>;
}

pub trait ClipperOpen3D<F: CoordFloatT = f64> {
    fn difference3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiLineString3D<F>;
    fn intersection3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiLineString3D<F>;
    fn offset3d(
        &self,
        delta: F,
        join_type: JoinType,
        end_type: EndType,
        factor: F,
    ) -> MultiPolygon3D<F>;
}

impl<F: CoordFloat, U: ToOwnedPolygon<F> + ClosedPoly + ?Sized> Clipper2D<F> for U {
    fn difference2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon2D<F> {
        execute_boolean_operation(ClipType_ctDifference, self, other, factor)
    }

    fn intersection2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon2D<F> {
        execute_boolean_operation(ClipType_ctIntersection, self, other, factor)
    }

    fn union2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon2D<F> {
        execute_boolean_operation(ClipType_ctUnion, self, other, factor)
    }

    fn xor2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon2D<F> {
        execute_boolean_operation(ClipType_ctXor, self, other, factor)
    }

    fn offset2d(
        &self,
        delta: F,
        join_type: JoinType,
        end_type: EndType,
        factor: F,
    ) -> MultiPolygon2D<F> {
        execute_offset_operation2d(self, delta * factor, join_type, end_type, factor)
    }

    fn offset_simplify_clean2d(
        &self,
        delta: F,
        jt: JoinType,
        et: EndType,
        pft: PolyFillType,
        distance: F,
        factor: F,
    ) -> MultiLineString2D<F> {
        execute_offset_simplify_clean_operation2d(
            self,
            delta * factor,
            jt,
            et,
            pft,
            distance,
            factor,
        )
    }

    fn simplify2d(&self, fill_type: PolyFillType, factor: F) -> MultiLineString2D<F> {
        execute_simplify_operation2d(self, fill_type, factor)
    }

    fn clean2d(&self, distance: F, factor: F) -> MultiLineString2D<F> {
        execute_clean_operation2d(self, distance * factor, factor)
    }
}

impl<F: CoordFloatT, U: ToOwnedPolygon<F> + ClosedPoly + ?Sized> Clipper3D<F> for U {
    fn difference3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon3D<F> {
        execute_boolean_operation(ClipType_ctDifference, self, other, factor)
    }

    fn intersection3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon3D<F> {
        execute_boolean_operation(ClipType_ctIntersection, self, other, factor)
    }

    fn union3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon3D<F> {
        execute_boolean_operation(ClipType_ctUnion, self, other, factor)
    }

    fn xor3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiPolygon3D<F> {
        execute_boolean_operation(ClipType_ctXor, self, other, factor)
    }

    fn offset3d(
        &self,
        delta: F,
        join_type: JoinType,
        end_type: EndType,
        factor: F,
    ) -> MultiPolygon3D<F> {
        execute_offset_operation3d(self, delta * factor, join_type, end_type, factor)
    }

    fn offset_simplify_clean3d(
        &self,
        delta: F,
        jt: JoinType,
        et: EndType,
        pft: PolyFillType,
        distance: F,
        factor: F,
    ) -> MultiLineString3D<F> {
        execute_offset_simplify_clean_operation3d(
            self,
            delta * factor,
            jt,
            et,
            pft,
            distance,
            factor,
        )
    }

    fn simplify3d(&self, fill_type: PolyFillType, factor: F) -> MultiLineString3D<F> {
        execute_simplify_operation3d(self, fill_type, factor)
    }

    fn clean3d(&self, distance: F, factor: F) -> MultiLineString3D<F> {
        execute_clean_operation3d(self, distance * factor, factor)
    }
}

impl<F: CoordFloat, U: ToOwnedPolygon<F> + OpenPath + ?Sized> ClipperOpen2D<F> for U {
    fn difference2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiLineString2D<F> {
        execute_boolean_operation(ClipType_ctDifference, self, other, factor)
    }

    fn intersection2d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiLineString2D<F> {
        execute_boolean_operation(ClipType_ctIntersection, self, other, factor)
    }

    fn offset2d(
        &self,
        delta: F,
        join_type: JoinType,
        end_type: EndType,
        factor: F,
    ) -> MultiPolygon2D<F> {
        execute_offset_operation2d(self, delta * factor, join_type, end_type, factor)
    }
}

impl<F: CoordFloatT, U: ToOwnedPolygon<F> + OpenPath + ?Sized> ClipperOpen3D<F> for U {
    fn difference3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiLineString3D<F> {
        execute_boolean_operation(ClipType_ctDifference, self, other, factor)
    }

    fn intersection3d<T: ToOwnedPolygon<F> + ClosedPoly + ?Sized>(
        &self,
        other: &T,
        factor: F,
    ) -> MultiLineString3D<F> {
        execute_boolean_operation(ClipType_ctIntersection, self, other, factor)
    }

    fn offset3d(
        &self,
        delta: F,
        join_type: JoinType,
        end_type: EndType,
        factor: F,
    ) -> MultiPolygon3D<F> {
        execute_offset_operation3d(self, delta * factor, join_type, end_type, factor)
    }
}

#[cfg(test)]
mod tests {
    use crate::coord;

    use super::*;

    #[test]
    fn test_closed_clip() {
        let expected = MultiPolygon2D::new(vec![Polygon2D::new(
            LineString2D::new(vec![
                coord! { x: 240.0, y: 200.0 },
                coord! { x: 190.0, y: 200.0 },
                coord! { x: 190.0, y: 150.0 },
                coord! { x: 240.0, y: 150.0 },
            ]),
            vec![LineString2D::new(vec![
                coord! { x: 200.0, y: 190.0 },
                coord! { x: 230.0, y: 190.0 },
                coord! { x: 215.0, y: 160.0 },
            ])],
        )]);

        let subject = Polygon2D::new(
            LineString2D::new(vec![
                coord! { x: 180.0, y: 200.0 },
                coord! { x: 260.0, y: 200.0 },
                coord! { x: 260.0, y: 150.0 },
                coord! { x: 180.0, y: 150.0 },
            ]),
            vec![LineString2D::new(vec![
                coord! { x: 215.0, y: 160.0 },
                coord! { x: 230.0, y: 190.0 },
                coord! { x: 200.0, y: 190.0 },
            ])],
        );

        let clip = Polygon2D::new(
            LineString2D::new(vec![
                coord! { x: 190.0, y: 210.0 },
                coord! { x: 240.0, y: 210.0 },
                coord! { x: 240.0, y: 130.0 },
                coord! { x: 190.0, y: 130.0 },
            ]),
            vec![],
        );

        let result = subject.intersection2d(&clip, 1.0);
        assert_eq!(expected, result);
    }
}
