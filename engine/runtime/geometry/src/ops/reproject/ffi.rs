//! PROJ-backed coordinate transformation for the reprojection ops.

use std::ffi::{CStr, CString};
use std::os::raw::c_int;
use std::ptr;

use nusamai_projection::crs::EpsgCode;
use proj_sys::{
    proj_context_create, proj_context_destroy, proj_context_errno, proj_context_errno_string,
    proj_create_crs_to_crs, proj_destroy, proj_errno, proj_errno_reset,
    proj_normalize_for_visualization, proj_trans, PJ, PJ_CONTEXT, PJ_COORD, PJ_DIRECTION_PJ_FWD,
    PJ_XYZT,
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
