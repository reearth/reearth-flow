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

pub trait Projection {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error>;
}

impl Projection for Coordinate2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        let (y, x, _) = projection
            .project_forward(self.y, self.x, self.z.into())
            .map_err(Error::projection)?;
        self.x = x;
        self.y = y;
        Ok(())
    }
}

impl Projection for Coordinate3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        let (y, x, z) = projection
            .project_forward(self.y, self.x, self.z)
            .map_err(Error::projection)?;
        self.x = x;
        self.y = y;
        self.z = z;
        Ok(())
    }
}

impl Projection for Point2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.0.projection(projection)
    }
}

impl Projection for Point3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.0.projection(projection)
    }
}

impl Projection for MultiPoint2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for coord in self.0.iter_mut() {
            coord.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for MultiPoint3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for coord in self.0.iter_mut() {
            coord.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for Line2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.start.projection(projection)?;
        self.end.projection(projection)?;
        Ok(())
    }
}

impl Projection for Line3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.start.projection(projection)?;
        self.end.projection(projection)?;
        Ok(())
    }
}

impl Projection for LineString2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for coord in self.0.iter_mut() {
            coord.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for LineString3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for coord in self.0.iter_mut() {
            coord.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for MultiLineString2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for line in self.0.iter_mut() {
            line.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for MultiLineString3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for line in self.0.iter_mut() {
            line.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for Polygon2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.exterior.projection(projection)?;
        self.exterior.close();
        for interior in &mut self.interiors {
            interior.projection(projection)?;
        }
        for interior in &mut self.interiors {
            interior.close();
        }
        Ok(())
    }
}

impl Projection for Polygon3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.exterior.projection(projection)?;
        self.exterior.close();
        for interior in &mut self.interiors {
            interior.projection(projection)?;
        }
        for interior in &mut self.interiors {
            interior.close();
        }
        Ok(())
    }
}

impl Projection for Solid2D<f64> {
    fn projection(
        &mut self,
        _projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl Projection for Solid3D<f64> {
    fn projection(
        &mut self,
        _projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl Projection for MultiPolygon2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for polygon in self.0.iter_mut() {
            polygon.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for MultiPolygon3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for polygon in self.0.iter_mut() {
            polygon.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for Triangle2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.0.projection(projection)?;
        self.1.projection(projection)?;
        self.2.projection(projection)?;
        Ok(())
    }
}

impl Projection for Triangle3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.0.projection(projection)?;
        self.1.projection(projection)?;
        self.2.projection(projection)?;
        Ok(())
    }
}

impl Projection for Rect2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.min.projection(projection)?;
        self.max.projection(projection)?;
        Ok(())
    }
}

impl Projection for Rect3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        self.min.projection(projection)?;
        self.max.projection(projection)?;
        Ok(())
    }
}

impl Projection for Geometry2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        match self {
            Geometry2D::Point(p) => p.projection(projection),
            Geometry2D::MultiPoint(mp) => mp.projection(projection),
            Geometry2D::Line(l) => l.projection(projection),
            Geometry2D::MultiLineString(ml) => ml.projection(projection),
            Geometry2D::Polygon(p) => p.projection(projection),
            Geometry2D::MultiPolygon(mp) => mp.projection(projection),
            Geometry2D::Rect(r) => r.projection(projection),
            Geometry2D::Solid(s) => s.projection(projection),
            Geometry2D::Triangle(t) => t.projection(projection),
            Geometry2D::GeometryCollection(gc) => {
                for geometry in gc.iter_mut() {
                    geometry.projection(projection)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl Projection for Geometry3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        match self {
            Geometry3D::Point(p) => p.projection(projection),
            Geometry3D::MultiPoint(mp) => mp.projection(projection),
            Geometry3D::Line(l) => l.projection(projection),
            Geometry3D::MultiLineString(ml) => ml.projection(projection),
            Geometry3D::Polygon(p) => p.projection(projection),
            Geometry3D::MultiPolygon(mp) => mp.projection(projection),
            Geometry3D::Rect(r) => r.projection(projection),
            Geometry3D::Solid(s) => s.projection(projection),
            Geometry3D::Triangle(t) => t.projection(projection),
            Geometry3D::GeometryCollection(gc) => {
                for geometry in gc.iter_mut() {
                    geometry.projection(projection)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl Projection for GeometryCollection2D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for geometry in self.0.iter_mut() {
            geometry.projection(projection)?;
        }
        Ok(())
    }
}

impl Projection for GeometryCollection3D<f64> {
    fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for geometry in self.0.iter_mut() {
            geometry.projection(projection)?;
        }
        Ok(())
    }
}
