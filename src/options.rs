use std::marker::PhantomData;

use cxx::UniquePtr;

use crate::map_renderer::ffi;
use crate::{ImageRenderer, MapMode, Static, Tile};

#[derive(Debug, Clone)]
pub struct ImageRendererOptions {
    width: u32,
    height: u32,
    pixel_ratio: f32,
    // FIXME: can we make this an Option<PathBuf>
    cache_path: String,
    // FIXME: can we make this an Option<PathBuf>
    asset_root: String,
    // TODO: remove?
    api_key: String,

    base_url: String,
    uri_scheme_alias: String,
    api_key_parameter_name: String,
    source_template: String,
    style_template: String,
    sprites_template: String,
    glyphs_template: String,
    tile_template: String,
    default_style_url: String,
    requires_api_key: bool,
}

impl Default for ImageRendererOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageRendererOptions {
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 512,
            height: 512,
            pixel_ratio: 1.0,
            cache_path: "cache.sqlite".to_string(),
            asset_root: ".".to_string(),
            api_key: String::new(),
            base_url: "https://demotiles.maplibre.org".to_string(),
            uri_scheme_alias: "maplibre".to_string(),
            api_key_parameter_name: String::new(),
            source_template: "/tiles/{domain}.json".to_string(),
            style_template: "{path}.json".to_string(),
            sprites_template: "/{path}/sprite{scale}.{format}".to_string(),
            glyphs_template: "/font/{fontstack}/{start}-{end}.pbf".to_string(),
            tile_template: "/{path}".to_string(),
            default_style_url: String::from("https://demotiles.maplibre.org/style.json"),
            requires_api_key: false,
        }
    }

    pub fn with_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_pixel_ratio(&mut self, pixel_ratio: f32) -> &mut Self {
        self.pixel_ratio = pixel_ratio;
        self
    }

    pub fn with_cache_path(&mut self, cache_path: String) -> &mut Self {
        self.cache_path = cache_path;
        self
    }

    pub fn with_asset_root(&mut self, asset_root: String) -> &mut Self {
        self.asset_root = asset_root;
        self
    }

    pub fn with_api_key(&mut self, api_key: String) -> &mut Self {
        self.api_key = api_key;
        self
    }

    pub fn with_base_url(&mut self, base_url: String) -> &mut Self {
        self.base_url = base_url;
        self
    }

    pub fn with_uri_scheme_alias(&mut self, uri_scheme_alias: String) -> &mut Self {
        self.uri_scheme_alias = uri_scheme_alias;
        self
    }

    pub fn with_api_key_parameter_name(&mut self, api_key_parameter_name: String) -> &mut Self {
        self.api_key_parameter_name = api_key_parameter_name;
        self
    }

    pub fn with_source_template(&mut self, source_template: String) -> &mut Self {
        self.source_template = source_template;
        self
    }

    pub fn with_style_template(&mut self, style_template: String) -> &mut Self {
        self.style_template = style_template;
        self
    }

    pub fn with_sprites_template(&mut self, sprites_template: String) -> &mut Self {
        self.sprites_template = sprites_template;
        self
    }

    pub fn with_glyphs_template(&mut self, glyphs_template: String) -> &mut Self {
        self.glyphs_template = glyphs_template;
        self
    }

    pub fn with_tile_template(&mut self, tile_template: String) -> &mut Self {
        self.tile_template = tile_template;
        self
    }

    pub fn with_default_style_url(&mut self, default_style_url: String) -> &mut Self {
        self.default_style_url = default_style_url;
        self
    }

    pub fn set_requires_api_key(&mut self, requires_api_key: bool) -> &mut Self {
        self.requires_api_key = requires_api_key;
        self
    }

    #[must_use]
    pub fn build_static_renderer(self) -> ImageRenderer<Static> {
        // TODO: Should the width/height be passed in here, or have another `build_static_with_size` method?
        ImageRenderer::new(MapMode::Static, &self)
    }

    #[must_use]
    pub fn build_tile_renderer(self) -> ImageRenderer<Tile> {
        // TODO: Is the width/height used for this mode?
        ImageRenderer::new(MapMode::Tile, &self)
    }
}

impl<S> ImageRenderer<S> {
    /// Private constructor.
    fn new(map_mode: MapMode, opts: &ImageRendererOptions) -> Self {
        let map = ffi::MapRenderer_new(
            map_mode,
            opts.width,
            opts.height,
            opts.pixel_ratio,
            &opts.cache_path,
            &opts.asset_root,
            &opts.api_key,
            &opts.base_url,
            &opts.uri_scheme_alias,
            &opts.api_key_parameter_name,
            &opts.source_template,
            &opts.style_template,
            &opts.sprites_template,
            &opts.glyphs_template,
            &opts.tile_template,
            &opts.default_style_url,
            opts.requires_api_key,
        );

        Self(map, PhantomData)
    }
}
