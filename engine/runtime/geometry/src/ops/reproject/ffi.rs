//! PROJ-backed coordinate transformation for the reprojection ops.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::OnceLock;

use parking_lot::RwLock;

use crate::coordinate::EpsgCode;
use proj_sys::{
    proj_context_create, proj_context_destroy, proj_context_errno, proj_context_errno_string,
    proj_create, proj_create_crs_to_crs, proj_crs_get_coordinate_system, proj_crs_get_sub_crs,
    proj_cs_get_axis_count, proj_cs_get_axis_info, proj_destroy, proj_errno, proj_errno_reset,
    proj_trans, PJ, PJ_CONTEXT, PJ_COORD, PJ_DIRECTION_PJ_FWD, PJ_XYZT,
};

use crate::error::{Error, Result};

/// Caches the live PROJ transformation for one source/target EPSG pair.
#[derive(Default)]
pub struct ReprojectionCache {
    /// The cached transformation, if any.
    current: Option<Entry>,
}

/// The PROJ objects for one `(from, to)` transformation.
struct Entry {
    /// Source EPSG code.
    from: EpsgCode,
    /// Target EPSG code.
    to: EpsgCode,
    /// The PROJ context.
    ctx: *mut PJ_CONTEXT,
    /// The PROJ transformation.
    pj: *mut PJ,
}

impl Drop for Entry {
    fn drop(&mut self) {
        // SAFETY: `pj` and `ctx` were created by PROJ and are owned solely by
        // this `Entry`; freeing the transformation before the context matches
        // PROJ's ownership model.
        unsafe {
            if !self.pj.is_null() {
                proj_destroy(self.pj);
            }
            if !self.ctx.is_null() {
                proj_context_destroy(self.ctx);
            }
        }
    }
}

impl ReprojectionCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform a single 3D point from `from` to `to` (EPSG codes).
    pub(crate) fn transform(
        &mut self,
        from: EpsgCode,
        to: EpsgCode,
        p: [f64; 3],
    ) -> Result<[f64; 3]> {
        if self.current.as_ref().map(|e| (e.from, e.to)) != Some((from, to)) {
            self.current = Some(Entry::build(from, to)?);
        }
        let entry = self.current.as_ref().expect("just populated");

        // SAFETY: `entry.pj` is a valid, non-null transformation for the whole
        // lifetime of `entry`; `proj_trans` takes/returns `PJ_COORD` by value.
        unsafe {
            proj_errno_reset(entry.pj);
            let input = PJ_COORD {
                xyzt: PJ_XYZT {
                    x: p[0],
                    y: p[1],
                    z: p[2],
                    t: f64::INFINITY,
                },
            };
            let out = proj_trans(entry.pj, PJ_DIRECTION_PJ_FWD, input);
            let errno = proj_errno(entry.pj);
            if errno != 0 {
                return Err(Error::projection(format!(
                    "proj_trans EPSG:{from}->EPSG:{to} failed (errno {errno}): {}",
                    errno_string(entry.ctx, errno)
                )));
            }
            let o = out.xyzt;
            if !o.x.is_finite() || !o.y.is_finite() || !o.z.is_finite() {
                return Err(Error::projection(format!(
                    "proj_trans EPSG:{from}->EPSG:{to} produced non-finite output"
                )));
            }
            Ok([o.x, o.y, o.z])
        }
    }
}

impl Entry {
    /// Build the PROJ transformation for the `(from, to)` EPSG pair.
    fn build(from: EpsgCode, to: EpsgCode) -> Result<Self> {
        // SAFETY: each PROJ object is null-checked before use; errno is read
        // while the context is still alive; on any failure all objects created
        // so far are freed before returning.
        unsafe {
            let ctx = proj_context_create();
            if ctx.is_null() {
                return Err(Error::projection("proj_context_create returned null"));
            }

            let c_from = CString::new(format!("EPSG:{from}")).map_err(Error::projection)?;
            let c_to = CString::new(format!("EPSG:{to}")).map_err(Error::projection)?;

            let pj = proj_create_crs_to_crs(ctx, c_from.as_ptr(), c_to.as_ptr(), ptr::null_mut());
            if pj.is_null() {
                let msg = ctx_errno_string(ctx);
                proj_context_destroy(ctx);
                return Err(Error::projection(format!(
                    "failed to create transform EPSG:{from}->EPSG:{to}: {msg}"
                )));
            }

            Ok(Self { from, to, ctx, pj })
        }
    }
}

/// Format a PROJ `errno` into its message string.
// SAFETY: `ctx` must be a valid, non-null PROJ context.
unsafe fn errno_string(ctx: *mut PJ_CONTEXT, errno: c_int) -> String {
    let s = proj_context_errno_string(ctx, errno);
    if s.is_null() {
        format!("proj errno {errno}")
    } else {
        CStr::from_ptr(s).to_string_lossy().into_owned()
    }
}

/// Read and format the current error of `ctx`.
// SAFETY: `ctx` must be a valid, non-null PROJ context.
unsafe fn ctx_errno_string(ctx: *mut PJ_CONTEXT) -> String {
    errno_string(ctx, proj_context_errno(ctx))
}

/// Process-wide memoization of computed orientation signs, keyed by EPSG code.
/// The sign is a fixed property of a CRS, so a value cached once stays valid for
/// the life of the process. Only successful lookups are cached; an unknown or
/// unsupported CRS is a rare error path not worth memoizing.
fn sign_cache() -> &'static RwLock<HashMap<EpsgCode, i8>> {
    static CACHE: OnceLock<RwLock<HashMap<EpsgCode, i8>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// The orientation sign of `epsg`: `+1` when the CRS's declared axis basis is
/// right-handed in canonical `(East, North[, Up])` order, `-1` when reflected.
/// Errors when the CRS is unknown or its axes are not aligned to those directions.
///
/// Memoized per EPSG code: the first call for a CRS pays the PROJ lookup, later
/// calls read the cached sign.
pub(crate) fn axis_order_sign(epsg: EpsgCode) -> Result<i8> {
    if let Some(&sign) = sign_cache().read().get(&epsg) {
        return Ok(sign);
    }
    let sign = axis_order_sign_uncached(epsg)?;
    sign_cache().write().insert(epsg, sign);
    Ok(sign)
}

/// Compute the orientation sign of `epsg` directly from PROJ, without consulting
/// or populating the cache.
fn axis_order_sign_uncached(epsg: EpsgCode) -> Result<i8> {
    // SAFETY: each PROJ object is null-checked before use and every path frees
    // the objects it created; the axis-direction strings are owned by `cs` and
    // read while it is alive.
    unsafe {
        let ctx = proj_context_create();
        if ctx.is_null() {
            return Err(Error::projection("proj_context_create returned null"));
        }
        let def = CString::new(format!("EPSG:{epsg}")).map_err(Error::projection)?;
        let crs = proj_create(ctx, def.as_ptr());
        if crs.is_null() {
            let msg = ctx_errno_string(ctx);
            proj_context_destroy(ctx);
            return Err(Error::projection(format!(
                "failed to create CRS EPSG:{epsg}: {msg}"
            )));
        }
        let result = axis_sign_for_crs(ctx, crs, epsg);

        proj_destroy(crs);
        proj_context_destroy(ctx);
        result
    }
}

/// The orientation sign of a CRS, descending into a compound CRS's horizontal
/// sub-CRS (index 0) when the CRS has no single coordinate system of its own.
// SAFETY: `ctx` and `crs` must be valid, non-null PROJ objects.
unsafe fn axis_sign_for_crs(ctx: *mut PJ_CONTEXT, crs: *const PJ, epsg: EpsgCode) -> Result<i8> {
    let cs = proj_crs_get_coordinate_system(ctx, crs);
    if !cs.is_null() {
        let result = axis_sign_from_cs(ctx, cs, epsg);
        proj_destroy(cs);
        return result;
    }

    let horizontal = proj_crs_get_sub_crs(ctx, crs, 0);
    if horizontal.is_null() {
        return Err(Error::projection(format!(
            "EPSG:{epsg} has no coordinate system: {}",
            ctx_errno_string(ctx)
        )));
    }
    let cs = proj_crs_get_coordinate_system(ctx, horizontal);
    let result = if cs.is_null() {
        Err(Error::projection(format!(
            "EPSG:{epsg} horizontal sub-CRS has no coordinate system: {}",
            ctx_errno_string(ctx)
        )))
    } else {
        let sign = axis_sign_from_cs(ctx, cs, epsg);
        proj_destroy(cs);
        sign
    };
    proj_destroy(horizontal);
    result
}

/// The orientation sign of a coordinate system, from its axis directions.
// SAFETY: `ctx` and `cs` must be valid, non-null PROJ objects.
unsafe fn axis_sign_from_cs(ctx: *mut PJ_CONTEXT, cs: *const PJ, epsg: EpsgCode) -> Result<i8> {
    let n = proj_cs_get_axis_count(ctx, cs);
    if !(2..=3).contains(&n) {
        return Err(Error::projection(format!(
            "EPSG:{epsg} has an unsupported axis count ({n})"
        )));
    }
    let n = n as usize;

    // Each axis contributes a canonical unit column vector; the sign of the
    // determinant of those columns is the frame's orientation sign.
    let mut axes: Vec<[f64; 3]> = Vec::with_capacity(n);
    for i in 0..n {
        let mut direction: *const c_char = ptr::null();
        let ok = proj_cs_get_axis_info(
            ctx,
            cs,
            i as c_int,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut direction,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        if ok == 0 || direction.is_null() {
            return Err(Error::projection(format!(
                "EPSG:{epsg} axis {i} has no direction"
            )));
        }
        let dir = CStr::from_ptr(direction).to_string_lossy();
        let (row, sign) = canonical_axis(dir.as_ref()).ok_or_else(|| {
            Error::projection(format!(
                "EPSG:{epsg} axis {i} direction `{dir}` is not axis-aligned"
            ))
        })?;
        let mut axis = [0.0f64; 3];
        axis[row] = sign;
        axes.push(axis);
    }

    let det = if n == 2 {
        axes[0][0] * axes[1][1] - axes[0][1] * axes[1][0]
    } else {
        let (a, b, c) = (axes[0], axes[1], axes[2]);
        a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0])
    };
    if det > 0.0 {
        Ok(1)
    } else if det < 0.0 {
        Ok(-1)
    } else {
        Err(Error::projection(format!(
            "EPSG:{epsg} axes are not orthonormal in the (East, North, Up) basis"
        )))
    }
}

/// Map a PROJ axis direction to its `(row, sign)` in the canonical
/// `(East, North, Up)` basis, or `None` if it is not aligned to an axis.
///
/// Geocentric (ECEF) axes are treated as a right-handed basis in `X, Y, Z`
/// order, so a geocentric CRS resolves to orientation sign `+1`.
fn canonical_axis(direction: &str) -> Option<(usize, f64)> {
    match direction.to_ascii_lowercase().as_str() {
        "east" => Some((0, 1.0)),
        "west" => Some((0, -1.0)),
        "north" => Some((1, 1.0)),
        "south" => Some((1, -1.0)),
        "up" => Some((2, 1.0)),
        "down" => Some((2, -1.0)),
        "geocentricx" => Some((0, 1.0)),
        "geocentricy" => Some((1, 1.0)),
        "geocentricz" => Some((2, 1.0)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sign(code: u16) -> i8 {
        axis_order_sign(EpsgCode::new(code)).unwrap()
    }

    #[test]
    fn latitude_first_geographic_is_negative() {
        assert_eq!(sign(4326), -1); // WGS84 2D (lat, lon)
        assert_eq!(sign(4979), -1); // WGS84 3D (lat, lon, height)
        assert_eq!(sign(6697), -1); // JGD2011 + height (lat, lon, height)
    }

    #[test]
    fn northing_first_projected_is_negative() {
        assert_eq!(sign(6669), -1); // JGD2011 plane rectangular I (northing, easting)
    }

    #[test]
    fn easting_first_projected_is_positive() {
        assert_eq!(sign(3857), 1); // Web Mercator (easting, northing)
        assert_eq!(sign(32633), 1); // UTM 33N (easting, northing)
    }

    #[test]
    fn geocentric_is_positive() {
        assert_eq!(sign(4978), 1); // WGS84 geocentric (ECEF), right-handed X/Y/Z
    }

    #[test]
    fn unknown_crs_errors() {
        assert!(axis_order_sign(EpsgCode::new(1)).is_err());
    }

    #[test]
    fn sign_is_memoized_per_code() {
        let code = EpsgCode::new(32633);
        let computed = axis_order_sign(code).unwrap();
        // The first call stored the sign under this code, and a second call
        // returns the same value from the cache rather than recomputing it.
        assert_eq!(sign_cache().read().get(&code), Some(&computed));
        assert_eq!(axis_order_sign(code).unwrap(), computed);
    }
}
