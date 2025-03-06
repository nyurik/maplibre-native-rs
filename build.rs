use std::path::{Path, PathBuf};
use std::{env, fs};

// This allows build support to be unit-tested as well as packaged with the crate.
#[path = "build_support/src/lib.rs"]
mod build_support;

use build_support::parse_deps;
use walkdir::WalkDir;

trait CfgBool {
    fn define_bool(&mut self, key: &str, value: bool);
}

impl CfgBool for cmake::Config {
    fn define_bool(&mut self, key: &str, value: bool) {
        self.define(key, if value { "ON" } else { "OFF" });
    }
}

/// Helper that returns a new [`cmake::Config`] with common settings.
/// It selects the renderer based on Cargo features: the user must enable exactly one of:
/// "metal", "opengl", or "vulkan". If none are explicitly enabled, on iOS/macOS the default is metal,
/// and on all other platforms the default is vulkan.
fn create_cmake_config(cpp_root: &Path) -> cmake::Config {
    let mut cfg = cmake::Config::new(cpp_root);
    cfg.generator("Ninja");
    cfg.define("CMAKE_C_COMPILER_LAUNCHER", "ccache");
    cfg.define("CMAKE_CXX_COMPILER_LAUNCHER", "ccache");
    cfg.define_bool("MLN_DRAWABLE_RENDERER", true);
    cfg.define_bool("MLN_WITH_OPENGL", false);

    let with_opengl = env::var("CARGO_FEATURE_OPENGL").is_ok();
    let mut with_metal = env::var("CARGO_FEATURE_METAL").is_ok();
    let mut with_vulkan = env::var("CARGO_FEATURE_VULKAN").is_ok();

    let is_macos = cfg!(any(target_os = "ios", target_os = "macos"));
    if !with_opengl && !with_metal && !with_vulkan {
        if is_macos {
            with_metal = true;
        } else {
            with_vulkan = true;
        }
    } else if u8::from(with_metal) + u8::from(with_opengl) + u8::from(with_vulkan) > 1 {
        // This might not be the best logic, but it can change at any moment because it's a fallback with a warning
        // Current logic: if opengl is enabled, always use that, otherwise pick metal on macOS and vulkan on other platforms
        let choice = if with_opengl {
            with_metal = false;
            with_vulkan = false;
            "opengl"
        } else if is_macos {
            with_metal = true;
            with_vulkan = false;
            "metal"
        } else {
            with_vulkan = true;
            with_metal = false;
            "vulkan"
        };

        println!("cargo::warning=Features 'metal', 'opengl', and 'vulkan' are mutually exclusive.");
        println!("cargo::warning=Using '{choice}', but the selection defaults may change later.");
    }

    cfg.define_bool("MLN_WITH_OPENGL", with_opengl);
    cfg.define_bool("MLN_WITH_METAL", with_metal);
    cfg.define_bool("MLN_WITH_VULKAN", with_vulkan);
    cfg.define_bool("MLN_WITH_WERROR", false);

    // The default profile should be release even in a debug mode, otherwise it gets huge
    cfg.profile(
        env::var("MLN_BUILD_PROFILE")
            .as_deref()
            .unwrap_or("Release"),
    );

    cfg
}

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cpp_root = root.join("maplibre-native");
    let check_cmake_list = cpp_root.join("CMakeLists.txt");
    assert!(
        check_cmake_list.exists(),
        "{} does not exist, did you forget to run `git submodule update --init --recursive`?",
        check_cmake_list.display(),
    );

    // ------------------------------------------------------------------------
    // 1. Build the "mbgl-core-deps" target first so that mbgl-core-deps.txt is generated.
    // Since CMake installs targets into a "build" subdirectory, we look for the file there.
    // ------------------------------------------------------------------------
    let deps_build_dir = create_cmake_config(&cpp_root)
        .build_target("mbgl-core-deps")
        .build();
    let deps_file = deps_build_dir.join("build").join("mbgl-core-deps.txt");
    let deps_contents = fs::read_to_string(&deps_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", deps_file.display()));

    // Parse the deps file into a list of Cargo instructions.
    for instr in parse_deps(&deps_contents, &deps_build_dir.join("build"), true) {
        println!("{instr}");
    }

    // ------------------------------------------------------------------------
    // 2. Build the actual "mbgl-core" static library target.
    // ------------------------------------------------------------------------
    let core_build_dir = create_cmake_config(&cpp_root)
        .build_target("mbgl-core")
        .build()
        .join("build");
    let static_lib_base = core_build_dir.to_str().unwrap();
    println!("cargo:rustc-link-search=native={static_lib_base}",);

    // ------------------------------------------------------------------------
    // 3. Gather include directories and build the C++ bridge using cxx_build.
    // ------------------------------------------------------------------------
    // TODO: This is a temporary solution. We should get this list from CMake as well.
    let mut include_dirs = vec![
        root.join("include"),
        cpp_root.join("include"),
        cpp_root.join("platform/default/include"),
    ];
    for entry in WalkDir::new(cpp_root.join("vendor")) {
        let entry = entry.expect("Failed reading maplibre-native/vendor directory");
        if entry.file_type().is_dir() && !entry.path_is_symlink() && entry.file_name() == "include"
        {
            include_dirs.push(entry.path().to_path_buf());
        }
    }

    println!("cargo:rerun-if-changed=src/map_renderer.rs");
    println!("cargo:rerun-if-changed=src/map_renderer/map_renderer.h");
    cxx_build::bridge("src/map_renderer.rs")
        .includes(&include_dirs)
        // .file("src/wrapper.cpp")  // we may need this later
        .flag_if_supported("-std=c++20")
        .compile("maplibre_rust_map_renderer_bindings");

    // Link mbgl-core after the bridge - or else `cargo test` won't be able to find the symbols.
    println!("cargo:rustc-link-lib=static=mbgl-core");

    // ------------------------------------------------------------------------
    // 4. Instruct Cargo when to re-run the build script.
    // ------------------------------------------------------------------------
}
