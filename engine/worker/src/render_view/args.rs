//! The `render-view` command line, which is the worker's half of the
//! `/render-view` request shape in
//! `server/api/internal/infrastructure/cloudrunworker/worker.go`.
//!
//! Defaults deliberately match `ViewOptions::default()` so a request that omits
//! a field renders exactly as the CLI would.

use clap::{Arg, ArgAction, ArgMatches, Command};
use reearth_flow_feature_view::TextureCodec;

use super::report::Shape;

/// A parsed, validated request.
#[derive(Debug, Clone)]
pub struct RenderViewArgs {
    pub(crate) input: String,
    pub(crate) output: String,
    pub(crate) report_url: String,
    pub(crate) name: String,
    pub(crate) shape: Shape,
    pub(crate) row: Option<usize>,
    pub(crate) filter: Option<String>,
    pub(crate) draco: bool,
    pub(crate) texel_size: f64,
    pub(crate) texture_codec: TextureCodec,
    pub(crate) target_tile_size: u64,
    pub(crate) min_zoom: u8,
    pub(crate) max_zoom: u8,
    pub(crate) extent: i32,
    pub(crate) max_tile_bytes: u64,
}

pub fn build_render_view_command() -> Command {
    Command::new("render-view")
        .about("Render a run's intermediate data into a view, and write a report.")
        .long_about(
            "Read the intermediate-data JSONL a finished run left on one output port and render \
             it for a viewer, then write a JSON report describing the outcome. `gltf` renders one \
             row into a glb; `tiles` renders a selection into 3D Tiles when any 3D geometry is \
             present and vector tiles for an all-2D selection. A render that draws nothing is \
             recorded in the report and exits successfully: the reason belongs to the user, not \
             to the exit code.",
        )
        .arg(required_arg("input", "Intermediate-data file location.", 1))
        .arg(required_arg("output", "Root the view is written under.", 2))
        .arg(required_arg(
            "report-url",
            "Where the JSON report is written.",
            3,
        ))
        .arg(required_arg(
            "name",
            "Base name for the output within the root.",
            4,
        ))
        .arg(
            Arg::new("shape")
                .long("shape")
                .help("Which view to render.")
                .value_parser(["gltf", "tiles"])
                .required(true)
                .display_order(5),
        )
        .arg(
            Arg::new("row")
                .long("row")
                .help("Row to render, 0-based. Required by `gltf`, rejected by `tiles`.")
                .value_parser(clap::value_parser!(usize))
                .display_order(6),
        )
        .arg(
            Arg::new("filter")
                .long("filter")
                .help("Flow expression each feature must satisfy. `tiles` only.")
                .display_order(7),
        )
        .arg(
            Arg::new("no-draco")
                .long("no-draco")
                .help("Disable Draco mesh compression, which is on by default.")
                .action(ArgAction::SetTrue)
                .display_order(8),
        )
        .arg(
            Arg::new("texel-size")
                .long("texel-size")
                .help("Target texel size in metres per pixel. 0 keeps full detail.")
                .value_parser(clap::value_parser!(f64))
                .default_value("0")
                .display_order(9),
        )
        .arg(
            Arg::new("texture-codec")
                .long("texture-codec")
                .help("Texture encoding for 3D output.")
                .value_parser(["jpeg", "png", "ktx2-etc1s", "ktx2-uastc", "untextured"])
                .default_value("jpeg")
                .display_order(10),
        )
        .arg(
            Arg::new("target-tile-size")
                .long("target-tile-size")
                .help("3D only. Target content size per tile, in bytes.")
                .value_parser(clap::value_parser!(u64))
                .default_value("1048576")
                .display_order(11),
        )
        .arg(
            Arg::new("min-zoom")
                .long("min-zoom")
                .help("2D only. Lowest zoom the tile pyramid is sliced at.")
                .value_parser(clap::value_parser!(u8))
                .default_value("0")
                .display_order(12),
        )
        .arg(
            Arg::new("max-zoom")
                .long("max-zoom")
                .help("2D only. Highest zoom the tile pyramid is sliced at, inclusive.")
                .value_parser(clap::value_parser!(u8))
                .default_value("15")
                .display_order(13),
        )
        .arg(
            Arg::new("extent")
                .long("extent")
                .help("2D only. Coordinate grid resolution within a tile.")
                .value_parser(clap::value_parser!(i32))
                .default_value("4096")
                .display_order(14),
        )
        .arg(
            Arg::new("max-tile-bytes")
                .long("max-tile-bytes")
                .help("2D only. Size cap per tile, in bytes.")
                .value_parser(clap::value_parser!(u64))
                .default_value("500000")
                .display_order(15),
        )
}

fn required_arg(name: &'static str, help: &'static str, order: usize) -> Arg {
    Arg::new(name)
        .long(name)
        .help(help)
        .required(true)
        .display_order(order)
}

/// Validate a parsed command line into a request.
///
/// A violation here is a caller bug rather than a render outcome, so it is
/// returned as an error and no report is written: there is nothing to report on.
pub fn parse(mut matches: ArgMatches) -> Result<RenderViewArgs, String> {
    let shape = match matches.remove_one::<String>("shape").as_deref() {
        Some("gltf") => Shape::Gltf,
        Some("tiles") => Shape::Tiles,
        other => return Err(format!("unknown shape: {other:?}")),
    };
    let row = matches.remove_one::<usize>("row");
    let filter = matches.remove_one::<String>("filter");

    match shape {
        Shape::Gltf => {
            if row.is_none() {
                return Err("a gltf view renders one row, so --row is required".to_string());
            }
            if filter.is_some() {
                return Err("a gltf view renders one row; --filter applies to tiles".to_string());
            }
        }
        Shape::Tiles => {
            if row.is_some() {
                return Err("a tiles view renders a selection; --row applies to gltf".to_string());
            }
        }
    }

    let min_zoom = matches.remove_one::<u8>("min-zoom").unwrap_or(0);
    let max_zoom = matches.remove_one::<u8>("max-zoom").unwrap_or(15);
    if min_zoom > max_zoom {
        return Err(format!(
            "--min-zoom {min_zoom} is above --max-zoom {max_zoom}"
        ));
    }

    let texture_codec = match matches.remove_one::<String>("texture-codec").as_deref() {
        Some("png") => TextureCodec::Png,
        Some("ktx2-etc1s") => TextureCodec::Ktx2Etc1s,
        Some("ktx2-uastc") => TextureCodec::Ktx2Uastc,
        Some("untextured") => TextureCodec::Untextured,
        Some("jpeg") | None => TextureCodec::Jpeg,
        Some(other) => return Err(format!("unknown texture codec: {other}")),
    };

    let input = matches
        .remove_one::<String>("input")
        .ok_or_else(|| "missing --input".to_string())?;
    let output = matches
        .remove_one::<String>("output")
        .ok_or_else(|| "missing --output".to_string())?;
    let report_url = matches
        .remove_one::<String>("report-url")
        .ok_or_else(|| "missing --report-url".to_string())?;
    let name = matches
        .remove_one::<String>("name")
        .ok_or_else(|| "missing --name".to_string())?;

    Ok(RenderViewArgs {
        input,
        output,
        report_url,
        name,
        shape,
        row,
        filter,
        draco: !matches.get_flag("no-draco"),
        texel_size: matches.remove_one::<f64>("texel-size").unwrap_or(0.0),
        texture_codec,
        target_tile_size: matches
            .remove_one::<u64>("target-tile-size")
            .unwrap_or(1_048_576),
        min_zoom,
        max_zoom,
        extent: matches.remove_one::<i32>("extent").unwrap_or(4096),
        max_tile_bytes: matches
            .remove_one::<u64>("max-tile-bytes")
            .unwrap_or(500_000),
    })
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;

    const REQUIRED: &[&str] = &[
        "--input",
        "gs://b/in.jsonl.zst",
        "--output",
        "gs://b/out",
        "--report-url",
        "gs://b/out/report.json",
        "--name",
        "view",
    ];

    /// The required flags plus `extra`, as an owned argv.
    fn with(extra: &[&str]) -> Vec<String> {
        REQUIRED
            .iter()
            .chain(extra.iter())
            .map(|s| s.to_string())
            .collect()
    }

    fn parse_args(argv: &[String]) -> Result<RenderViewArgs, String> {
        let mut full = vec!["render-view".to_string()];
        full.extend_from_slice(argv);
        let matches = build_render_view_command()
            .try_get_matches_from(full)
            .map_err(|e| e.to_string())?;
        parse(matches)
    }

    #[test]
    fn a_tiles_request_parses_with_a_filter() {
        let args = parse_args(&with(&["--shape", "tiles", "--filter", "foo > 1"])).unwrap();
        assert_eq!(args.shape, Shape::Tiles);
        assert_eq!(args.filter.as_deref(), Some("foo > 1"));
        assert_eq!(args.row, None);
    }

    #[test]
    fn a_gltf_request_parses_with_a_row() {
        let args = parse_args(&with(&["--shape", "gltf", "--row", "4"])).unwrap();
        assert_eq!(args.shape, Shape::Gltf);
        assert_eq!(args.row, Some(4));
    }

    #[test]
    fn gltf_without_a_row_is_rejected() {
        // A glb renders one row; without one there is nothing to render.
        let err = parse_args(&with(&["--shape", "gltf"])).unwrap_err();
        assert!(err.contains("row"), "error was: {err}");
    }

    #[test]
    fn gltf_with_a_filter_is_rejected() {
        let err =
            parse_args(&with(&["--shape", "gltf", "--row", "1", "--filter", "x"])).unwrap_err();
        assert!(err.contains("filter"), "error was: {err}");
    }

    #[test]
    fn tiles_with_a_row_is_rejected() {
        let err = parse_args(&with(&["--shape", "tiles", "--row", "1"])).unwrap_err();
        assert!(err.contains("row"), "error was: {err}");
    }

    #[test]
    fn an_unknown_shape_is_rejected() {
        assert!(parse_args(&with(&["--shape", "wireframe"])).is_err());
    }

    #[test]
    fn a_min_zoom_above_max_zoom_is_rejected() {
        let err = parse_args(&with(&[
            "--shape",
            "tiles",
            "--min-zoom",
            "12",
            "--max-zoom",
            "4",
        ]))
        .unwrap_err();
        assert!(err.contains("zoom"), "error was: {err}");
    }

    #[test]
    fn defaults_match_the_library() {
        // Anything the caller omits must land on the value ViewOptions would
        // have chosen, so a request and an equivalent CLI run agree.
        let args = parse_args(&with(&["--shape", "tiles"])).unwrap();
        assert_eq!(args.target_tile_size, 1_048_576);
        assert_eq!(args.min_zoom, 0);
        assert_eq!(args.max_zoom, 15);
        assert_eq!(args.extent, 4096);
        assert_eq!(args.max_tile_bytes, 500_000);
        assert_eq!(args.texel_size, 0.0);
        assert!(args.draco);
    }

    #[test]
    fn every_texture_codec_the_consumer_can_send_parses() {
        for codec in ["jpeg", "png", "ktx2-etc1s", "ktx2-uastc", "untextured"] {
            assert!(
                parse_args(&with(&["--shape", "tiles", "--texture-codec", codec])).is_ok(),
                "{codec} must parse: featureview.TextureCodec can send it",
            );
        }
    }
}
