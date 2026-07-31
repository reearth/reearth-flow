use std::str::FromStr;
use std::sync::Arc;

use clap::{Arg, ArgAction, ArgMatches, Command};
use reearth_flow_common::uri::Uri;
use reearth_flow_feature_view::{
    default_name, load_selected, render_feature, render_tileset, Destination, Selection,
    TextureCodec, ViewOptions,
};
use reearth_flow_storage::resolve::StorageResolver;

/// The `view` subcommand.
///
/// Each shape takes only the selection it can act on: a row picks one feature
/// out of the table for a glb, a filter narrows a whole edge for a tileset.
pub fn build_view_command() -> Command {
    Command::new("view")
        .about("Render intermediate data into a viewable 3D form.")
        .long_about(
            "Read the intermediate-data JSONL a run left on an edge and render its features for \
             a 3D viewer. `gltf` renders one row into a glb, which is what showing a single \
             feature wants; `3dtiles` renders a whole edge into a tiled tileset for looking at \
             all of it at once.",
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            shared_args(Command::new("gltf"))
                .about("Render one row into a glb.")
                .arg(
                    Arg::new("row")
                        .long("row")
                        .help(
                            "Row to render, 0-based, as numbered in the intermediate-data table. \
                             This is the path behind clicking a table row: the scan stops there, \
                             and no later line is read.",
                        )
                        .value_parser(clap::value_parser!(usize))
                        .required(true)
                        .display_order(4),
                )
                .display_order(1),
        )
        .subcommand(
            shared_args(Command::new("3dtiles"))
                .about("Render a whole edge into a tiled 3D Tiles tileset.")
                .arg(
                    Arg::new("filter")
                        .long("filter")
                        .help(
                            "Flow expression evaluated against each feature; only features it \
                             returns true for are rendered. Omit to render every feature. The \
                             expression sees the feature alone, not workflow variables.",
                        )
                        .display_order(4),
                )
                .arg(
                    Arg::new("max-zoom")
                        .long("max-zoom")
                        .help("Deepest quadtree level a feature may be placed at.")
                        .value_parser(clap::value_parser!(u8))
                        .default_value("18")
                        .display_order(5),
                )
                .display_order(2),
        )
}

/// The arguments both shapes take.
fn shared_args(command: Command) -> Command {
    command
        .arg(
            Arg::new("input")
                .long("input")
                .help(
                    "Intermediate-data file location, e.g. \
                     .../feature-store/<node-id>.<port>.jsonl.zst.",
                )
                .required(true)
                .display_order(1),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .help("Directory the view is written under.")
                .required(true)
                .display_order(2),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .help(
                    "Base name for the output. A glTF view writes <name>.glb; a tileset writes \
                     <name>/tileset.json. Defaults to the input file's name.",
                )
                .display_order(3),
        )
        .arg(
            Arg::new("no-draco")
                .long("no-draco")
                .help(
                    "Skip Draco mesh compression. On by default; turn it off for a viewer that \
                     does not support KHR_draco_mesh_compression.",
                )
                .action(ArgAction::SetTrue)
                .display_order(7),
        )
        .arg(
            Arg::new("texel-size")
                .long("texel-size")
                .help("Target texel size in metres per pixel. 0 keeps full texture detail.")
                .value_parser(clap::value_parser!(f64))
                .default_value("0")
                .display_order(8),
        )
        .arg(
            Arg::new("texture-codec")
                .long("texture-codec")
                .help(
                    "Image codec for textures. JPEG is the default: it encodes far faster than \
                     the KTX2 forms at a comparable file size, which is what a view built on \
                     demand wants. KTX2 is GPU-compressed and cheaper to serve repeatedly, but \
                     its encode dominates render time. 'untextured' skips texturing entirely \
                     and renders geometry in its neutral colour.",
                )
                .value_parser(["jpeg", "png", "ktx2-etc1s", "ktx2-uastc", "untextured"])
                .default_value("jpeg")
                .display_order(9),
        )
}

#[derive(Debug, PartialEq)]
pub struct ViewCliCommand {
    input: String,
    output: String,
    name: Option<String>,
    draco: bool,
    texel_size: f64,
    texture_codec: TextureCodec,
    shape: Shape,
}

/// The view shape, holding the selection only that shape accepts.
#[derive(Debug, PartialEq)]
enum Shape {
    Gltf {
        row: usize,
    },
    Cesium3DTiles {
        filter: Option<String>,
        max_zoom: u8,
    },
}

impl ViewCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let (shape, mut matches) = matches
            .remove_subcommand()
            .ok_or(crate::errors::Error::init("No view shape provided"))?;
        let shape = match shape.as_str() {
            "gltf" => Shape::Gltf {
                row: matches
                    .remove_one::<usize>("row")
                    .ok_or(crate::errors::Error::init("No row provided"))?,
            },
            "3dtiles" => Shape::Cesium3DTiles {
                filter: matches.remove_one::<String>("filter"),
                max_zoom: matches.remove_one::<u8>("max-zoom").unwrap_or(18),
            },
            other => return Err(crate::errors::Error::unknown_command(other)),
        };
        let input = matches
            .remove_one::<String>("input")
            .ok_or(crate::errors::Error::init("No input uri provided"))?;
        let output = matches
            .remove_one::<String>("output")
            .ok_or(crate::errors::Error::init("No output uri provided"))?;
        let texture_codec = match matches.remove_one::<String>("texture-codec").as_deref() {
            Some("ktx2-etc1s") => TextureCodec::Ktx2Etc1s,
            Some("ktx2-uastc") => TextureCodec::Ktx2Uastc,
            Some("png") => TextureCodec::Png,
            Some("untextured") => TextureCodec::Untextured,
            Some("jpeg") | None => TextureCodec::Jpeg,
            Some(other) => {
                return Err(crate::errors::Error::init(format!(
                    "Unknown texture codec: {other}"
                )))
            }
        };
        Ok(ViewCliCommand {
            input,
            output,
            name: matches.remove_one::<String>("name"),
            draco: !matches.get_flag("no-draco"),
            texel_size: matches.remove_one::<f64>("texel-size").unwrap_or(0.0),
            texture_codec,
            shape,
        })
    }

    pub fn execute(&self) -> crate::Result<()> {
        let storage_resolver = StorageResolver::new();
        let input = Uri::from_str(self.input.as_str()).map_err(crate::errors::Error::init)?;
        let output = Uri::from_str(self.output.as_str()).map_err(crate::errors::Error::init)?;

        let options = ViewOptions {
            draco: self.draco,
            texel_size: self.texel_size,
            texture_codec: self.texture_codec,
            ..ViewOptions::default()
        };
        let name = self.name(&input)?;
        let destination = Destination {
            root: &output,
            prefix: name.as_str(),
            storage_resolver: &storage_resolver,
        };

        // One streaming read either way: naming a row decodes only as far as
        // it, a filter runs against each feature as it is parsed, and neither
        // holds a feature the view will not render.
        match &self.shape {
            Shape::Gltf { row } => {
                let loaded = load_selected(&input, Selection::Row(*row), &storage_resolver)
                    .map_err(crate::errors::Error::run)?;
                let Some(selected) = loaded.selection.first() else {
                    println!("Row {row} is past the end of the table; nothing was written");
                    return Ok(());
                };
                let view = render_feature(selected, &options, &destination)
                    .map_err(crate::errors::Error::run)?;
                match view.entry_point {
                    Some(entry_point) => {
                        println!("Rendered row {row} into {}", entry_point.as_str())
                    }
                    None => {
                        println!("Row {row} carried no renderable geometry; nothing was written")
                    }
                }
            }
            Shape::Cesium3DTiles { filter, max_zoom } => {
                let selection = match filter {
                    Some(expr) => Selection::Filter {
                        expr,
                        env_vars: Arc::new(serde_json::Map::new()),
                    },
                    None => Selection::All,
                };
                let loaded = load_selected(&input, selection, &storage_resolver)
                    .map_err(crate::errors::Error::run)?;
                let selected = loaded.selection.len();
                let view = render_tileset(&loaded.selection, *max_zoom, &options, &destination)
                    .map_err(crate::errors::Error::run)?;
                match view.entry_point {
                    Some(entry_point) => println!(
                        "Rendered {} of {selected} selected features ({} scanned) into {} \
                         ({} file(s))",
                        view.rendered_features,
                        loaded.scanned,
                        entry_point.as_str(),
                        view.written.len()
                    ),
                    None => println!(
                        "None of the {selected} selected features ({} scanned) carried \
                         renderable geometry; nothing was written",
                        loaded.scanned
                    ),
                }
            }
        }
        Ok(())
    }

    /// The output base name: what `--name` says, else the input file's name
    /// with its intermediate-data extension dropped.
    fn name(&self, input: &Uri) -> crate::Result<String> {
        if let Some(name) = &self.name {
            return Ok(name.clone());
        }
        default_name(input)
            .ok_or_else(|| crate::errors::Error::init(format!("{} has no file name", self.input)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<ArgMatches, clap::Error> {
        let mut argv = vec!["view", args[0], "--input", "in.jsonl", "--output", "out"];
        argv.extend_from_slice(&args[1..]);
        build_view_command().try_get_matches_from(argv)
    }

    /// Each shape takes its own selection argument and rejects the other's, so
    /// an invalid pairing cannot reach the renderer.
    #[test]
    fn a_shape_takes_only_its_own_selection() {
        assert!(parse(&["gltf", "--row", "1"]).is_ok());
        assert!(parse(&["3dtiles", "--filter", "true"]).is_ok());
        assert!(parse(&["3dtiles"]).is_ok(), "a tileset may take every row");

        assert!(parse(&["gltf", "--filter", "true"]).is_err());
        assert!(parse(&["3dtiles", "--row", "1"]).is_err());
        assert!(parse(&["gltf"]).is_err(), "a glb needs the rows to render");
    }

    #[test]
    fn the_row_and_shape_survive_parsing() {
        let matches = parse(&["gltf", "--row", "3"]).expect("parses");
        let command = ViewCliCommand::parse_cli_args(matches).expect("parses");

        assert_eq!(command.shape, Shape::Gltf { row: 3 });
        assert_eq!(command.texture_codec, TextureCodec::Jpeg);

        assert!(
            parse(&["gltf", "--row", "3", "--row", "1"]).is_err(),
            "a glb holds one row"
        );
    }
}
