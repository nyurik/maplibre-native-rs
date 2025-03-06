use cxx::{CxxString, UniquePtr};

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
