use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use downloader::{Download, Downloader};

// This allows build support to be unit-tested as well as packaged with the crate.
#[path = "build_helper.rs"]
mod build_helper;

use build_helper::parse_deps;
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
        // TODO: modify for better defaults
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
        println!("cargo::warning=Using only '{choice}', but this default selection may change in future releases.");
    }

    cfg.define_bool("MLN_WITH_OPENGL", with_opengl);
    cfg.define_bool("MLN_WITH_METAL", with_metal);
    cfg.define_bool("MLN_WITH_VULKAN", with_vulkan);
    cfg.define_bool("MLN_WITH_WERROR", false);

    // The default profile should be release even in a debug mode, otherwise it gets huge
    println!("cargo:rerun-if-env-changed=MLN_BUILD_PROFILE");
    cfg.profile(
        env::var("MLN_BUILD_PROFILE")
            .as_deref()
            .unwrap_or("Release"),
    );

    cfg
}

/// Check that a given dir contains valid maplibre-native code
fn validate_mln(dir: &Path, revision: &str) {
    assert!(
        dir.read_dir().expect("Can't read dir").next().is_some(),
        r"
MapLibre-native source directory is empty: {}
For local development, make sure to use
    git submodule update --init --recursive
You may also set MLN_FROM_SOURCE to the path of the maplibre-native directory.
",
        dir.display()
    );

    let dest_disp = dir.display();
    let rev = Command::new("git")
        .current_dir(dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get git revision");
    assert!(rev.status.success(), "Failed to validate git repo");
    let rev = String::from_utf8(rev.stdout).expect("Failed to parse git rev response");
    assert_eq!(
        rev.trim_ascii(),
        revision,
        "Unexpected git revision in {dest_disp}, please update the build.rs with the new value '{rev}'",
    );
}

fn clone_mln(dir: &Path, repo: &str, revision: &str) {
    let dir_disp = dir.display();
    println!("cargo:warning=Cloning {repo} to {dir_disp} for rev {revision}");

    // git(
    //     dir,
    //     [
    //         "clone",
    //         "--depth=40",
    //         "--recurse-submodules",
    //         "--shallow-submodules",
    //         repo,
    //         dir.to_str().unwrap(),
    //     ],
    // );
    // git(dir, ["reset", "--hard", revision]);

    // Ideally we want this method as it will only fetch the commit of interest.

    // Adapted from https://stackoverflow.com/a/3489576/177275
    // # make a new blank repository in the current directory
    git(dir, ["init"]);
    // # add a remote
    git(dir, ["remote", "add", "origin", repo]);
    // # fetch a commit (or branch or tag) of interest
    // # Note: the full history up to this commit will be retrieved unless
    // #       you limit it with '--depth=...' or '--shallow-since=...'
    git(dir, ["fetch", "origin", revision, "--depth=1"]);
    // # reset this repository's master branch to the commit of interest
    git(dir, ["reset", "--hard", "FETCH_HEAD"]);
    // # fetch submodules
    git(
        dir,
        [
            "submodule",
            "update",
            "--init",
            "--recursive",
            "--depth=1",
            "--jobs=8",
        ],
    );
}

fn git<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(dir: &Path, args: I) {
    fs::create_dir_all(dir).unwrap_or_else(|e| panic!("Failed to create {}: {e}", dir.display()));

    let args = args
        .into_iter()
        .map(|v| v.as_ref().to_os_string())
        .collect::<Vec<_>>();

    let mut cmd = Command::new("git");

    // let git_dir = dir.join(".git");
    // if git_dir.exists() {
    //     eprintln!(
    //         "Running git {args:?} in {} with GIT_DIR={}",
    //         dir.display(),
    //         dir.display()
    //     );
    //     cmd.env("GIT_DIR", dir);
    // } else {
    //     eprintln!("Running git {args:?} in {} without GIT_DIR", dir.display());
    // }

    cmd.current_dir(dir)
        .args(args.clone())
        .status()
        .map_err(|e| e.to_string())
        .and_then(|v| {
            if v.success() {
                Ok(())
            } else {
                Err(v.to_string())
            }
        })
        .unwrap_or_else(|e| panic!("Failed to run git {args:?}: {e}"));
}

const MLN_GIT_REPO: &str = "https://github.com/maplibre/maplibre-native.git";
const MLN_REVISION: &str = "3fc93a0b024e34514dafcbb424db93593ff540be";

fn clone_or_download(root: &Path) -> PathBuf {
    println!("cargo:rerun-if-env-changed=MLN_CLONE_REPO");
    println!("cargo:rerun-if-env-changed=MLN_FROM_SOURCE");
    let cpp_root = env::var_os("MLN_FROM_SOURCE").map(PathBuf::from);
    let sub_module = root.join("maplibre-native");
    let mut out_dir: PathBuf = env::var_os("OUT_DIR").expect("OUT_DIR is not set").into();
    out_dir.push("maplibre-native");

    let cpp_root = if let Some(cpp_root) = cpp_root {
        // User specified MLN_FROM_SOURCE - use that if it has CMakeLists.txt
        let cpp_disp = cpp_root.display();
        assert!(
            cpp_root.join("CMakeLists.txt").exists(),
            "Directory {cpp_disp} does not contain maplibre-native"
        );
        println!("cargo:warning=Using maplibre-native at {cpp_disp}");
        cpp_root
    } else if env::var_os("MLN_CLONE_REPO").is_some() {
        // Clone the repo into OUT_DIR - probably because this is part of dependency build
        // Warnings shouldn't show up in the final build output unless there's an error
        clone_mln(&out_dir, MLN_GIT_REPO, MLN_REVISION);
        out_dir
    } else if sub_module.is_dir() {
        // this is a local development that should have the submodule checked out.
        // Do not print any warnings - using the submodule directly
        validate_mln(&sub_module, MLN_REVISION);
        sub_module
    } else {
        // Defaults to downloading the static library
        #[cfg(target_os = "ios")]
        let url = "https://github.com/louwers/maplibre-native/releases/download/core-e4739013392a18e26956471b57480dd3950d33db/libmaplibre-native-core-ios-metal.a";
        #[cfg(target_os = "linux")]
        let url = "http://localhost:8000/libimpl.a";
        // let url = "https://github.com/louwers/maplibre-native/releases/download/core-e4739013392a18e26956471b57480dd3950d33db/libmaplibre-native-core-linux-opengl.a";
        #[cfg(target_os = "macos")]
        let url = "https://github.com/louwers/maplibre-native/releases/download/core-e4739013392a18e26956471b57480dd3950d33db/libmaplibre-native-core-macos-metal.a";

        println!(
            "cargo:warning=Downloading precompiled maplibre-native core library from {url} into {}",
            out_dir.display()
        );
        fs::create_dir_all(&out_dir).expect("Failed to create output directory");
        let mut downloader = Downloader::builder()
            .download_folder(&out_dir)
            .parallel_requests(1)
            .build()
            .expect("Failed to create downloader");
        let mut iter = downloader
            .download(&[Download::new(url)])
            .expect("Failed to download maplibre-native static lib")
            .into_iter();
        let (Some(result), None) = (iter.next(), iter.next()) else {
            panic!("Unexpected number of results from downloader");
        };
        // Returning the downloaded file, bypassing CMakeLists.txt check
        return result.expect("Error downloading static MLN").file_name;
    };

    let check_cmake_list = cpp_root.join("CMakeLists.txt");
    assert!(
        check_cmake_list.exists(),
        "{} does not exist",
        check_cmake_list.display(),
    );

    cpp_root
}

/// Build the "mbgl-core-deps" target first so that mbgl-core-deps.txt is generated.
fn add_link_targets(cpp_root: &Path) {
    let deps_build_dir = create_cmake_config(cpp_root)
        .build_target("mbgl-core-deps")
        .build();
    let deps_file = deps_build_dir.join("build").join("mbgl-core-deps.txt");
    let deps_contents = fs::read_to_string(&deps_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", deps_file.display()));

    // Parse the deps file into a list of Cargo instructions.
    for instr in parse_deps(&deps_contents, &deps_build_dir.join("build"), true) {
        println!("{instr}");
    }

    // FIXME:  These should not be manually set like this here
    println!("cargo:rustc-link-lib=icuuc");
    println!("cargo:rustc-link-lib=icui18n");
    println!("cargo:rustc-link-lib=jpeg");
    println!("cargo:rustc-link-lib=png");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=curl");
}

/// Build the actual "mbgl-core" static library target.
fn build_static_lib(cpp_root: &Path) {
    let core_build_dir = create_cmake_config(cpp_root)
        .build_target("mbgl-core")
        .build()
        .join("build");
    let static_lib_base = core_build_dir.to_str().unwrap();
    println!("cargo:rustc-link-search=native={static_lib_base}");
}

/// Gather include directories and build the C++ bridge using `cxx_build`.
fn build_bridge(root: &Path, cpp_root: &Path) {
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

    println!("cargo:rerun-if-changed=src/renderer/bridge.rs");
    println!("cargo:rerun-if-changed=include/map_renderer.h");
    cxx_build::bridge("src/renderer/bridge.rs")
        .includes(&include_dirs)
        .file("src/renderer/bridge.cpp")
        .flag_if_supported("-std=c++20")
        .compile("maplibre_rust_map_renderer_bindings");

    // Link mbgl-core after the bridge - or else `cargo test` won't be able to find the symbols.
    println!("cargo:rustc-link-lib=static=mbgl-core");
}

fn build_mln() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cpp_root = clone_or_download(&root);
    if cpp_root.is_dir() {
        add_link_targets(&cpp_root);
        build_static_lib(&cpp_root);
    } else {
        println!(
            "cargo:warning=Using precompiled maplibre-native static library from {}",
            cpp_root.display()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            cpp_root
                .to_str()
                .expect("Invalid UTF-8 in static MLN filename")
        );
    }
    build_bridge(&root, &cpp_root);
}

fn main() {
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping build.rs when building for docs.rs");
        println!("cargo::rustc-cfg=docsrs");
        println!("cargo:rustc-check-cfg=cfg(docsrs)");
    } else {
        build_mln();
    }
}
