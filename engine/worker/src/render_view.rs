//! The `render-view` subcommand: render a finished run's intermediate data into
//! a view the UI can open, and leave a report saying what happened.

pub(crate) mod args;
pub(crate) mod report;

pub(crate) use args::build_render_view_command;
