use std::f64::consts::PI;
use std::marker::PhantomData;
use std::path::Path;

use cxx::{CxxString, UniquePtr};

use crate::options::ImageRendererOptions;
use crate::{MapDebugOptions, MapMode};

#[cxx::bridge(namespace = "mln::bridge")]
pub mod ffi {
    //
    // CXX validates enum types against the C++ definition during compilation
    //

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MapMode {
        Continuous,
        Static,
        Tile,
    }

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MapDebugOptions {
        NoDebug = 0,
        TileBorders = 0b0000_0010, // 1 << 1
        ParseStatus = 0b0000_0100, // 1 << 2
        Timestamps = 0b0000_1000,  // 1 << 3
        Collision = 0b0001_0000,   // 1 << 4
        Overdraw = 0b0010_0000,    // 1 << 5
        StencilClip = 0b0100_0000, // 1 << 6
        DepthBuffer = 0b1000_0000, // 1 << 7
    }

    #[namespace = "mbgl"]
    unsafe extern "C++" {
        include!("mbgl/map/mode.hpp");

        type MapMode;
        type MapDebugOptions;
    }

    unsafe extern "C++" {
        include!("map_renderer.h");
        // include!("maplibre-native/src/map_renderer/map_renderer.h");

        type MapRenderer;

        #[allow(clippy::too_many_arguments)]
        fn MapRenderer_new(
            mapMode: MapMode,
            width: u32,
            height: u32,
            pixelRatio: f32,
            cachePath: &str,
            assetRoot: &str,
            apiKey: &str,
            baseUrl: &str,
            uriSchemeAlias: &str,
            apiKeyParameterName: &str,
            sourceTemplate: &str,
            styleTemplate: &str,
            spritesTemplate: &str,
            glyphsTemplate: &str,
            tileTemplate: &str,
            defaultStyleUrl: &str,
            requiresApiKey: bool,
        ) -> UniquePtr<MapRenderer>;
        fn MapRenderer_render(obj: Pin<&mut MapRenderer>) -> UniquePtr<CxxString>;
        fn MapRenderer_setDebugFlags(obj: Pin<&mut MapRenderer>, flags: MapDebugOptions);
        fn MapRenderer_setCamera(
            obj: Pin<&mut MapRenderer>,
            lat: f64,
            lon: f64,
            zoom: f64,
            bearing: f64,
            pitch: f64,
        );
        fn MapRenderer_setStyleUrl(obj: Pin<&mut MapRenderer>, url: &str);
    }
}

/// A rendered map image.
///
/// The image is stored as a PNG byte array in a buffer allocated by the C++ code.
pub struct Image(UniquePtr<CxxString>);

impl Image {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Internal state type to render a static map image.
pub struct Static;
/// Internal state type to render a map tile.
pub struct Tile;

/// Configuration options for a tile server.
pub struct ImageRenderer<S>(
    pub(crate) UniquePtr<ffi::MapRenderer>,
    pub(crate) PhantomData<S>,
);

impl<S> ImageRenderer<S> {
    /// Set the style URL for the map.
    // FIXME: without this call, renderer just hangs
    pub fn set_style_url(&mut self, url: &str) -> &mut Self {
        // FIXME: return a result instead of panicking
        assert!(url.contains("://"));
        ffi::MapRenderer_setStyleUrl(self.0.pin_mut(), url);
        self
    }

    pub fn set_style_path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        // TODO: check if the file exists?
        // FIXME: return a result instead of panicking
        let path = path.as_ref().to_str().expect("Path is not valid UTF-8");
        ffi::MapRenderer_setStyleUrl(self.0.pin_mut(), &format!("file://{path}"));
        self
    }

    pub fn set_camera(
        &mut self,
        lat: f64,
        lon: f64,
        zoom: f64,
        bearing: f64,
        pitch: f64,
    ) -> &mut Self {
        ffi::MapRenderer_setCamera(self.0.pin_mut(), lat, lon, zoom, bearing, pitch);
        self
    }

    pub fn set_debug_flags(&mut self, flags: MapDebugOptions) -> &mut Self {
        ffi::MapRenderer_setDebugFlags(self.0.pin_mut(), flags);
        self
    }
}

impl ImageRenderer<Static> {
    pub fn render_static(&mut self) -> Image {
        Image(ffi::MapRenderer_render(self.0.pin_mut()))
    }
}

impl ImageRenderer<Tile> {
    pub fn render_tile(&mut self, zoom: u8, x: u64, y: u64) -> Image {
        let (lat, lon) = coords_to_lat_lon(f64::from(zoom), x, y);
        ffi::MapRenderer_setCamera(self.0.pin_mut(), lat, lon, f64::from(zoom), 0.0, 0.0);
        Image(ffi::MapRenderer_render(self.0.pin_mut()))
    }
}

#[allow(clippy::cast_precision_loss)]
fn coords_to_lat_lon(zoom: f64, x: u64, y: u64) -> (f64, f64) {
    // https://github.com/oldmammuth/slippy_map_tilenames/blob/058678480f4b50b622cda7a48b98647292272346/src/lib.rs#L114
    let zz = 2_f64.powf(zoom);
    let lng = (x as f64 + 0.5) / zz * 360_f64 - 180_f64;
    let lat = ((PI * (1_f64 - 2_f64 * (y as f64 + 0.5) / zz)).sinh())
        .atan()
        .to_degrees();
    (lat, lng)
}
