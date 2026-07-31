use clap::{Arg, ArgAction, ArgMatches, Command};

/// The `view` subcommand.
pub fn build_view_command() -> Command {
    Command::new("view")
        .about("Render intermediate data into a viewable 3D form.")
        .long_about(
            "Read the intermediate-data JSONL a run left on an edge, keep the features a Flow \
             expression selects, and render them for a 3D viewer. The default is a single glb \
             holding the whole selection, which suits one feature or a handful; --format 3dtiles \
             renders a tiled 3D Tiles tileset instead, for looking at an entire edge at once.",
        )
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
            Arg::new("row")
                .long("row")
                .help(
                    "Row to render, 0-based, as numbered in the intermediate-data table. \
                     Repeatable. This is the path behind clicking a table row: only the named \
                     lines are parsed, so it does not pay to decode the whole edge. Cannot be \
                     combined with --filter.",
                )
                .value_parser(clap::value_parser!(usize))
                .action(ArgAction::Append)
                .conflicts_with("filter")
                .display_order(4),
        )
        .arg(
            Arg::new("filter")
                .long("filter")
                .help(
                    "Flow expression evaluated against each feature; only features it returns \
                     true for are rendered. Omit to render every feature.",
                )
                .display_order(4),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .help("View to render.")
                .value_parser(["gltf", "3dtiles"])
                .default_value("gltf")
                .display_order(5),
        )
        .arg(
            Arg::new("max-zoom")
                .long("max-zoom")
                .help("Deepest quadtree level a feature may be placed at. 3D Tiles only.")
                .value_parser(clap::value_parser!(u8))
                .default_value("18")
                .display_order(6),
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
    rows: Vec<usize>,
    filter: Option<String>,
    format: String,
    max_zoom: u8,
    draco: bool,
    texel_size: f64,
    texture_codec: String,
}

impl ViewCliCommand {
    pub fn parse_cli_args(mut matches: ArgMatches) -> crate::Result<Self> {
        let input = matches
            .remove_one::<String>("input")
            .ok_or(crate::errors::Error::init("No input uri provided"))?;
        let output = matches
            .remove_one::<String>("output")
            .ok_or(crate::errors::Error::init("No output uri provided"))?;
        Ok(ViewCliCommand {
            input,
            output,
            name: matches.remove_one::<String>("name"),
            rows: matches
                .remove_many::<usize>("row")
                .map(|rows| rows.collect())
                .unwrap_or_default(),
            filter: matches.remove_one::<String>("filter"),
            format: matches
                .remove_one::<String>("format")
                .unwrap_or_else(|| "gltf".to_string()),
            max_zoom: matches.remove_one::<u8>("max-zoom").unwrap_or(18),
            draco: !matches.get_flag("no-draco"),
            texel_size: matches.remove_one::<f64>("texel-size").unwrap_or(0.0),
            texture_codec: matches
                .remove_one::<String>("texture-codec")
                .unwrap_or_else(|| "jpeg".to_string()),
        })
    }

    pub fn execute(&self) -> crate::Result<()> {
        use std::sync::Arc;

        use reearth_flow_common::uri::Uri;
        use reearth_flow_feature_view::{
            load_selected, render, Destination, Selection, TextureCodec, ViewFormat, ViewOptions,
        };
        use reearth_flow_storage::resolve::StorageResolver;

        let storage_resolver = StorageResolver::new();
        let input = Uri::for_test(self.input.as_str());
        let output = Uri::for_test(self.output.as_str());

        // One streaming read either way: naming rows parses only those lines, a
        // filter runs against each feature as it is parsed, and neither holds a
        // feature the view will not render.
        let selection = if !self.rows.is_empty() {
            Selection::Rows(&self.rows)
        } else if let Some(expr) = self.filter.as_deref() {
            Selection::Filter {
                expr,
                env_vars: Arc::new(serde_json::Map::new()),
            }
        } else {
            Selection::All
        };
        let scan_stopped_early = !self.rows.is_empty();
        let loaded = load_selected(&input, selection, &storage_resolver)
            .map_err(crate::errors::Error::run)?;
        let selection = loaded.selection;
        let total = if scan_stopped_early {
            "requested".to_string()
        } else {
            loaded.scanned.to_string()
        };

        let options = ViewOptions {
            format: if self.format == "3dtiles" {
                ViewFormat::Cesium3DTiles {
                    max_zoom: self.max_zoom,
                }
            } else {
                ViewFormat::Gltf
            },
            draco: self.draco,
            texel_size: self.texel_size,
            texture_codec: match self.texture_codec.as_str() {
                "ktx2-etc1s" => TextureCodec::Ktx2Etc1s,
                "ktx2-uastc" => TextureCodec::Ktx2Uastc,
                "png" => TextureCodec::Png,
                "untextured" => TextureCodec::Untextured,
                _ => TextureCodec::Jpeg,
            },
            ..ViewOptions::default()
        };

        let name = self.default_name(&input)?;
        let destination = Destination {
            root: &output,
            prefix: name.as_str(),
            storage_resolver: &storage_resolver,
        };

        let view = render(&selection, &options, &destination).map_err(crate::errors::Error::run)?;

        match view.entry_point {
            Some(entry_point) => println!(
                "Rendered {} of {total} features into {} ({} file(s))",
                selection.len(),
                entry_point.as_str(),
                view.written.len()
            ),
            None => println!(
                "None of the {} selected features (of {total}) carried renderable geometry; \
                 nothing was written",
                selection.len()
            ),
        }
        Ok(())
    }

    /// The output base name: what `--name` says, else the input file's name
    /// with its intermediate-data extension dropped.
    fn default_name(&self, input: &reearth_flow_common::uri::Uri) -> crate::Result<String> {
        if let Some(name) = &self.name {
            return Ok(name.clone());
        }
        let name = input
            .file_name()
            .and_then(|name| name.to_str().map(str::to_string))
            .ok_or_else(|| {
                crate::errors::Error::init(format!("{} has no file name", self.input))
            })?;
        let stem = name
            .strip_suffix(".jsonl.zst")
            .or_else(|| name.strip_suffix(".jsonl"))
            .unwrap_or(name.as_str());
        Ok(stem.to_string())
    }
}
