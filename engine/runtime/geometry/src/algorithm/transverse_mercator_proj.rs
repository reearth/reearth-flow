use nusamai_projection::etmerc::ExtendedTransverseMercatorProjection;

use crate::{
    error::Error,
    types::{
        coordinate::{Coordinate2D, Coordinate3D},
        geometry::{Geometry2D, Geometry3D},
        geometry_collection::{GeometryCollection2D, GeometryCollection3D},
        line::{Line2D, Line3D},
        line_string::{LineString2D, LineString3D},
        multi_line_string::{MultiLineString2D, MultiLineString3D},
        multi_point::{MultiPoint2D, MultiPoint3D},
        multi_polygon::{MultiPolygon2D, MultiPolygon3D},
        point::{Point2D, Point3D},
        polygon::{Polygon2D, Polygon3D},
        rect::{Rect2D, Rect3D},
        solid::{Solid2D, Solid3D},
        triangle::{Triangle2D, Triangle3D},
    },
};

pub trait TransverseMercatorProjection {
    fn project_forward(
        &mut self,
        project: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error>;
}

impl TransverseMercatorProjection for Coordinate2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        _keep_z: bool,
    ) -> Result<(), Error> {
        let (x, y, _) = projection
            .project_forward(self.x, self.y, self.z.into())
            .map_err(Error::projection)?;
        self.x = x;
        self.y = y;
        Ok(())
    }
}

impl TransverseMercatorProjection for Coordinate3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        if keep_z {
            let (x, y, z) = projection
                .project_forward(self.x, self.y, self.z)
                .map_err(Error::projection)?;
            self.x = x;
            self.y = y;
            self.z = z;
        } else {
            let (x, y, _) = projection
                .project_forward(self.x, self.y, 0.)
                .map_err(Error::projection)?;
            self.x = x;
            self.y = y;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for Point2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.0.project_forward(projection, keep_z)
    }
}

impl TransverseMercatorProjection for Point3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.0.project_forward(projection, keep_z)
    }
}

impl TransverseMercatorProjection for MultiPoint2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for coord in self.0.iter_mut() {
            coord.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for MultiPoint3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for coord in self.0.iter_mut() {
            coord.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for Line2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.start.project_forward(projection, keep_z)?;
        self.end.project_forward(projection, keep_z)?;
        Ok(())
    }
}

impl TransverseMercatorProjection for Line3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.start.project_forward(projection, keep_z)?;
        self.end.project_forward(projection, keep_z)?;
        Ok(())
    }
}

impl TransverseMercatorProjection for LineString2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for coord in self.0.iter_mut() {
            coord.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for LineString3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for coord in self.0.iter_mut() {
            coord.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for MultiLineString2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for line in self.0.iter_mut() {
            line.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for MultiLineString3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for line in self.0.iter_mut() {
            line.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for Polygon2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.exterior.project_forward(projection, keep_z)?;
        self.exterior.close();
        for interior in &mut self.interiors {
            interior.project_forward(projection, keep_z)?;
        }
        for interior in &mut self.interiors {
            interior.close();
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for Polygon3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.exterior.project_forward(projection, keep_z)?;
        self.exterior.close();
        for interior in &mut self.interiors {
            interior.project_forward(projection, keep_z)?;
        }
        for interior in &mut self.interiors {
            interior.close();
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for Solid2D<f64> {
    fn project_forward(
        &mut self,
        _projection: &ExtendedTransverseMercatorProjection,
        _keep_z: bool,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl TransverseMercatorProjection for Solid3D<f64> {
    fn project_forward(
        &mut self,
        _projection: &ExtendedTransverseMercatorProjection,
        _keep_z: bool,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl TransverseMercatorProjection for MultiPolygon2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for polygon in self.0.iter_mut() {
            polygon.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for MultiPolygon3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for polygon in self.0.iter_mut() {
            polygon.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for Triangle2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.0.project_forward(projection, keep_z)?;
        self.1.project_forward(projection, keep_z)?;
        self.2.project_forward(projection, keep_z)?;
        Ok(())
    }
}

impl TransverseMercatorProjection for Triangle3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.0.project_forward(projection, keep_z)?;
        self.1.project_forward(projection, keep_z)?;
        self.2.project_forward(projection, keep_z)?;
        Ok(())
    }
}

impl TransverseMercatorProjection for Rect2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.min.project_forward(projection, keep_z)?;
        self.max.project_forward(projection, keep_z)?;
        Ok(())
    }
}

impl TransverseMercatorProjection for Rect3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        self.min.project_forward(projection, keep_z)?;
        self.max.project_forward(projection, keep_z)?;
        Ok(())
    }
}

impl TransverseMercatorProjection for Geometry2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        match self {
            Geometry2D::Point(p) => p.project_forward(projection, keep_z),
            Geometry2D::MultiPoint(mp) => mp.project_forward(projection, keep_z),
            Geometry2D::Line(l) => l.project_forward(projection, keep_z),
            Geometry2D::MultiLineString(ml) => ml.project_forward(projection, keep_z),
            Geometry2D::Polygon(p) => p.project_forward(projection, keep_z),
            Geometry2D::MultiPolygon(mp) => mp.project_forward(projection, keep_z),
            Geometry2D::Rect(r) => r.project_forward(projection, keep_z),
            Geometry2D::Solid(s) => s.project_forward(projection, keep_z),
            Geometry2D::Triangle(t) => t.project_forward(projection, keep_z),
            Geometry2D::GeometryCollection(gc) => {
                for geometry in gc.iter_mut() {
                    geometry.project_forward(projection, keep_z)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl TransverseMercatorProjection for Geometry3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        match self {
            Geometry3D::Point(p) => p.project_forward(projection, keep_z),
            Geometry3D::MultiPoint(mp) => mp.project_forward(projection, keep_z),
            Geometry3D::Line(l) => l.project_forward(projection, keep_z),
            Geometry3D::MultiLineString(ml) => ml.project_forward(projection, keep_z),
            Geometry3D::Polygon(p) => p.project_forward(projection, keep_z),
            Geometry3D::MultiPolygon(mp) => mp.project_forward(projection, keep_z),
            Geometry3D::Rect(r) => r.project_forward(projection, keep_z),
            Geometry3D::Solid(s) => s.project_forward(projection, keep_z),
            Geometry3D::Triangle(t) => t.project_forward(projection, keep_z),
            Geometry3D::GeometryCollection(gc) => {
                for geometry in gc.iter_mut() {
                    geometry.project_forward(projection, keep_z)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl TransverseMercatorProjection for GeometryCollection2D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for geometry in self.0.iter_mut() {
            geometry.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}

impl TransverseMercatorProjection for GeometryCollection3D<f64> {
    fn project_forward(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
        keep_z: bool,
    ) -> Result<(), Error> {
        for geometry in self.0.iter_mut() {
            geometry.project_forward(projection, keep_z)?;
        }
        Ok(())
    }
}
