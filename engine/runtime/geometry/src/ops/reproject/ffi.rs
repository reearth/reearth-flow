//! Thin safe wrapper over the PROJ C API (`proj-sys`) for true 3D point
//! reprojection.
//!
//! The high-level `proj` crate only exposes 2D conversion (its `convert()`
//! hardcodes `z = 0` and reads back only `x, y`), so we call `proj_trans`
//! directly with a 4D `PJ_COORD` and read the transformed `z` back.
//!
//! [`Transformer`] is a caller-owned, single-entry cache of the live PROJ
//! objects for one `(source, target)` EPSG pair. It is accessed `&mut`, holds
//! raw `*mut PJ` pointers and is therefore neither `Send` nor `Sync`: create one
//! per single-threaded unit of work (e.g. per `process()` invocation) and let it
//! be reused across all the leaves of one geometry, which usually share a frame
//! so the underlying `PJ` is built once.

use std::ffi::{CStr, CString};
use std::ptr;

use nusamai_projection::crs::EpsgCode;
use proj_sys::{
    proj_context_create, proj_context_destroy, proj_context_errno, proj_context_errno_string,
    proj_create_crs_to_crs, proj_destroy, proj_errno, proj_errno_reset,
    proj_normalize_for_visualization, proj_trans, PJ, PJ_CONTEXT, PJ_COORD, PJ_DIRECTION_PJ_FWD,
    PJ_XYZT,
};

use crate::error::{Error, Result};

/// A caller-owned cache of the live PROJ transformation for one `(from, to)`
/// EPSG pair. See the module docs for the threading contract.
#[derive(Default)]
pub struct Transformer {
    current: Option<Entry>,
}

/// The live PROJ objects for one `(from, to)` pair. Owns its context and the
/// normalized transformation; both are freed on drop.
struct Entry {
    from: EpsgCode,
    to: EpsgCode,
    ctx: *mut PJ_CONTEXT,
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

impl Transformer {
    /// An empty cache; the first `transform` call builds the projection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform a single 3D point from `from` to `to` (EPSG codes).
    ///
    /// Reuses the cached projection when `(from, to)` is unchanged, otherwise
    /// (re)builds it. Coordinates are in PROJ's visualization order, i.e.
    /// `x = longitude/easting`, `y = latitude/northing`.
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
                    // No temporal epoch: HUGE_VAL disables any time-dependent step.
                    t: f64::INFINITY,
                },
            };
            let out = proj_trans(entry.pj, PJ_DIRECTION_PJ_FWD, input);
            let errno = proj_errno(entry.pj);
            if errno != 0 {
                return Err(Error::projection(format!(
                    "proj_trans EPSG:{from}->EPSG:{to} failed (errno {errno}): {}",
                    ctx_errno_string(entry.ctx)
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

            // Normalize so coordinates are always longitude/latitude (x/y),
            // regardless of the CRS authority axis order.
            let pj_norm = proj_normalize_for_visualization(ctx, pj);
            proj_destroy(pj);
            if pj_norm.is_null() {
                let msg = ctx_errno_string(ctx);
                proj_context_destroy(ctx);
                return Err(Error::projection(format!(
                    "failed to normalize transform EPSG:{from}->EPSG:{to}: {msg}"
                )));
            }

            Ok(Self {
                from,
                to,
                ctx,
                pj: pj_norm,
            })
        }
    }
}

/// Read the current error string for `ctx`. Must be called while `ctx` is alive.
///
/// SAFETY: `ctx` must be a valid, non-null PROJ context.
unsafe fn ctx_errno_string(ctx: *mut PJ_CONTEXT) -> String {
    let errno = proj_context_errno(ctx);
    let s = proj_context_errno_string(ctx, errno);
    if s.is_null() {
        format!("proj errno {errno}")
    } else {
        CStr::from_ptr(s).to_string_lossy().into_owned()
    }
}
