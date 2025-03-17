use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use maplibre_native::{Image, ImageRendererOptions, MapDebugOptions};

/// Command-line tool to render a map via [`mapLibre-native`](https://github.com/maplibre/maplibre-native)
#[derive(Parser, Debug)]
struct Args {
    /// API key
    #[arg(short = 't', long = "apikey", env = "MLN_API_KEY")]
    apikey: Option<String>,

    /// Map stylesheet
    #[arg(
        short = 's',
        long = "style",
        default_value = "https://demotiles.maplibre.org/style.json"
    )]
    style: String,

    /// Output file name
    #[arg(short = 'o', long = "output", default_value = "out.png")]
    output: PathBuf,

    /// Cache database file name
    #[arg(short = 'c', long = "cache", default_value = "cache.sqlite")]
    cache: PathBuf,

    /// Directory to which `asset://` URLs will resolve
    #[arg(short = 'a', long = "assets", default_value = ".")]
    asset_root: PathBuf,

    /// Adds an debug overlay
    #[arg(long)]
    debug: Option<DebugMode>,

    /// Image scale factor
    #[arg(short = 'r', long = "ratio", default_value_t = 1.0)]
    ratio: f32,

    /// Zoom level
    #[arg(short = 'z', long = "zoom", default_value_t = 0)]
    zoom: u8,

    /// Longitude
    #[arg(short = 'x', long = "x", default_value_t = 0)]
    x: u32,

    /// Latitude
    #[arg(short = 'y', long = "y", default_value_t = 0)]
    y: u32,

    /// Bearing
    #[arg(short = 'b', long = "bearing", default_value_t = 0.0)]
    bearing: f64,

    /// Pitch
    #[arg(short = 'p', long = "pitch", default_value_t = 0.0)]
    pitch: f64,

    /// Image width
    #[arg(long = "width", default_value_t = 512)]
    width: u32,

    /// Image height
    #[arg(long = "height", default_value_t = 512)]
    height: u32,

    /// Map mode
    #[arg(short = 'm', long = "mode", default_value = "static")]
    mode: Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
enum Mode {
    #[default]
    /// Once-off still image of an arbitrary viewport
    Static,
    /// Once-off still image of a single tile
    Tile,
    /// Continually updating map
    Continuous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum DebugMode {
    /// Edges of tile boundaries are shown as thick, red lines.
    ///
    /// Can help diagnose tile clipping issues.
    TileBorders,
    ParseStatus,
    /// Each tile shows a timestamp indicating when it was loaded.
    Timestamps,
    /// Edges of glyphs and symbols are shown as faint, green lines.
    ///
    /// Can help diagnose collision and label placement issues.
    Collision,
    /// Each drawing operation is replaced by a translucent fill.
    ///
    /// Overlapping drawing operations appear more prominent to help diagnose overdrawing.
    Overdraw,
    /// The stencil buffer is shown instead of the color buffer.
    ///
    /// Note: This option does nothing in Release builds of the SDK.
    StencilClip,
    /// The depth buffer is shown instead of the color buffer.
    ///
    /// Note: This option does nothing in Release builds of the SDK
    DepthBuffer,
}

impl From<DebugMode> for MapDebugOptions {
    fn from(value: DebugMode) -> Self {
        match value {
            DebugMode::TileBorders => MapDebugOptions::TileBorders,
            DebugMode::ParseStatus => MapDebugOptions::ParseStatus,
            DebugMode::Timestamps => MapDebugOptions::Timestamps,
            DebugMode::Collision => MapDebugOptions::Collision,
            DebugMode::Overdraw => MapDebugOptions::Overdraw,
            DebugMode::StencilClip => MapDebugOptions::StencilClip,
            DebugMode::DepthBuffer => MapDebugOptions::DepthBuffer,
        }
    }
}

impl Args {
    fn render(self) -> Image {
        let mut map = ImageRendererOptions::new();
        map.with_api_key(self.apikey.unwrap_or_default());
        map.with_cache_path(self.cache.to_string_lossy().to_string());
        map.with_asset_root(self.asset_root.to_string_lossy().to_string());
        map.with_pixel_ratio(self.ratio);
        map.with_size(self.width, self.height);

        match self.mode {
            Mode::Static => {
                let mut map = map.build_static_renderer();
                if let Some(debug) = self.debug {
                    map.set_debug_flags(debug.into());
                }
                map.set_style_url(&self.style);
                map.set_camera(
                    f64::from(self.x),
                    f64::from(self.y),
                    f64::from(self.zoom),
                    self.bearing,
                    self.pitch,
                );
                map.render_static()
            }
            Mode::Tile => {
                if self.bearing != 0.0 {
                    println!("Warning: nonzero bearing is ignored in tile-mode");
                }
                if self.pitch != 0.0 {
                    println!("Warning: nonzero pitch is ignored in tile-mode");
                }
                let mut map = map.build_tile_renderer();
                map.set_style_url(&self.style);
                if let Some(debug) = self.debug {
                    map.set_debug_flags(debug.into());
                }
                map.render_tile(self.zoom, self.x, self.y)
            }
            Mode::Continuous => {
                todo!("not yet implemented in the wrapper")
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    println!("Rendering arguments: {args:#?}");
    let output = args.output.clone();

    let before_initalisation = Instant::now();
    let data = args.render();
    println!(
        "Rendering successfull in {elapsed:?}, writing result to {output:?}",
        elapsed = before_initalisation.elapsed()
    );
    println!("Note: Future renders using the same instance would be faster due to amortized initialization");
    fs::write(&output, data.as_slice())
        .unwrap_or_else(|e| panic!("Failed to write rendered map to {output:?} because of {e:?}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendering() {
        let args = Args {
            width: 32,
            height: 32,
            mode: Mode::Static,
            ..Args::parse()
        };
        let data = args.render();
        assert!(!data.as_slice().is_empty());

        let args = Args {
            width: 64,
            height: 64,
            mode: Mode::Tile,
            ..Args::parse()
        };
        let data = args.render();
        assert!(!data.as_slice().is_empty());
    }
}
