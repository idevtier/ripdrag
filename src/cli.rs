use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(about, version)]
pub struct Cli {
    /// Be verbose
    #[arg(short, long)]
    pub verbose: bool,

    /// Act as a target instead of source
    #[arg(short, long)]
    pub target: bool,

    /// With --target, keep files to drag out
    #[arg(short, long, requires = "target")]
    pub keep: bool,

    /// With --target, keep files to drag out
    #[arg(short, long, requires = "target")]
    pub print_path: bool,

    /// Make the window resizable
    #[arg(short, long)]
    pub resizable: bool,

    /// Exit after first successful drag or drop
    #[arg(short = 'x', long)]
    pub and_exit: bool,

    /// Only display icons, no labels
    #[arg(short, long)]
    pub icons_only: bool,

    /// Don't load thumbnails from images
    #[arg(short, long)]
    pub disable_thumbnails: bool,

    /// Size of icons and thumbnails
    #[arg(short = 's', long, value_name = "SIZE", default_value_t = 32)]
    pub icon_size: i32,

    /// Min width of the main window
    #[arg(short = 'W', long, value_name = "WIDTH", default_value_t = 360)]
    pub content_width: i32,

    /// Default height of the main window
    #[arg(short = 'H', long, value_name = "HEIGHT", default_value_t = 360)]
    pub content_height: i32,

    /// Accept paths from stdin
    #[arg(short = 'I', long)]
    pub from_stdin: bool,

    /// Drag all the items together
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Show only the number of items and drag them together
    #[arg(short = 'A', long)]
    pub all_compact: bool,

    /// Paths to the files you want to drag
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}
