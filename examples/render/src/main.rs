use std::{fs, path::PathBuf, time::Instant};

use clap::Parser;
use maplibre_native::{ImageRendererOptions, MapDebugOptions};

/// MapLibre Native rendering tool
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
    /// once-off still image of an arbitrary viewport
    Static,
    /// once-off still image of a single tile
    Tile,
    /// continually updating map
    Continuous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum DebugMode {
    /// Edges of tile boundaries are shown as thick, red lines to help diagnose tile clipping issues.
    TileBorders,
    ParseStatus,
    /// Each tile shows a timestamp indicating when it was loaded.
    Timestamps,
    /// Edges of glyphs and symbols are shown as faint, green lines to help diagnose collision and label placement issues.
    Collision,
    /// Each drawing operation is replaced by a translucent fill. Overlapping drawing operations appear more prominent to help diagnose overdrawing.
    Overdraw,
    StencilClip,
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

impl Args{
    fn render(self)->Vec<u8>{
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
                    self.x as f64,
                    self.y as f64,
                    self.zoom as f64,
                    self.bearing,
                    self.pitch,
                );
                map.render_static().as_slice().to_vec()
            }
            Mode::Tile => {
                if self.bearing != 0.0 || self.pitch != 0.0 {
                    println!(
                        "Warning: bearing and pitch are not supported in tile-mode and will be ignored"
                    );
                }
                let mut map = map.build_tile_renderer();
                map.set_style_url(&self.style);
                if let Some(debug) = self.debug {
                    map.set_debug_flags(debug.into());
                }
                map.render_tile(self.zoom, self.x, self.y)
                    .as_slice()
                    .to_vec()
            }
            Mode::Continuous => {
                todo!("not yet implemented in the wrapper")
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    println!("Rendering arguments: {:#?}", args);
    let output = args.output.clone();

    let before_initalisation = Instant::now();
    let data = args.render();
    println!(
        "Rendering successfull in {elapsed:?}, writing result to {output:?}",
        elapsed = before_initalisation.elapsed()
    );
    println!("Note: Rendering subsequent tiles/images would be faster as initialization is amortized.");
    fs::write(&output, &data)
        .expect(&format!("Failed to write rendered map to {output:?}"));
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
        assert!(!data.is_empty());
        
        let args = Args {
            width: 64,
            height: 64,
            mode: Mode::Tile,
            ..Args::parse()
        };
        let data = args.render();
        assert!(!data.is_empty());
    }
}