//! Throwaway benchmark: isolate where `proj_create_crs_to_crs` time goes for
//! EPSG:6697 (JGD2011 compound, horizontal+vertical) -> EPSG:4979 (WGS84 3D
//! geographic). Not part of the crate's public API; delete after use.

use std::ffi::CString;
use std::ptr;
use std::time::Instant;

use proj_sys::{
    proj_context_create, proj_context_destroy, proj_create_crs_to_crs, proj_destroy,
    proj_normalize_for_visualization,
};

fn one_shot_fresh_context(from: &str, to: &str) {
    unsafe {
        let t_ctx = Instant::now();
        let ctx = proj_context_create();
        let ctx_dur = t_ctx.elapsed();

        let c_from = CString::new(from).unwrap();
        let c_to = CString::new(to).unwrap();

        let t_crs = Instant::now();
        let pj = proj_create_crs_to_crs(ctx, c_from.as_ptr(), c_to.as_ptr(), ptr::null_mut());
        let crs_dur = t_crs.elapsed();
        assert!(!pj.is_null(), "proj_create_crs_to_crs returned null");

        let t_norm = Instant::now();
        let pj_norm = proj_normalize_for_visualization(ctx, pj);
        let norm_dur = t_norm.elapsed();
        assert!(!pj_norm.is_null());

        println!(
            "[fresh ctx] context_create={ctx_dur:?} create_crs_to_crs={crs_dur:?} normalize={norm_dur:?} total={:?}",
            ctx_dur + crs_dur + norm_dur
        );

        proj_destroy(pj);
        proj_destroy(pj_norm);
        proj_context_destroy(ctx);
    }
}

fn repeated_shared_context(from: &str, to: &str, n: usize) {
    unsafe {
        let ctx = proj_context_create();
        let c_from = CString::new(from).unwrap();
        let c_to = CString::new(to).unwrap();
        for i in 0..n {
            let t = Instant::now();
            let pj = proj_create_crs_to_crs(ctx, c_from.as_ptr(), c_to.as_ptr(), ptr::null_mut());
            let dur = t.elapsed();
            assert!(!pj.is_null());
            println!("[shared ctx, call {i}] create_crs_to_crs={dur:?}");
            proj_destroy(pj);
        }
        proj_context_destroy(ctx);
    }
}

fn main() {
    println!("=== 5x fresh context + crs_to_crs + normalize, EPSG:6697 -> EPSG:4979 ===");
    for i in 0..5 {
        println!("-- iteration {i} --");
        one_shot_fresh_context("EPSG:6697", "EPSG:4979");
    }

    println!("\n=== 10x crs_to_crs reusing ONE context, EPSG:6697 -> EPSG:4979 ===");
    repeated_shared_context("EPSG:6697", "EPSG:4979", 10);

    println!("\n=== 5x fresh context + crs_to_crs + normalize, EPSG:6697 -> EPSG:4978 ===");
    for i in 0..5 {
        println!("-- iteration {i} --");
        one_shot_fresh_context("EPSG:6697", "EPSG:4978");
    }
}
