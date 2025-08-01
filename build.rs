use std::path::PathBuf;

fn main() {
    // Only build on macOS
    if !cfg!(target_os = "macos") {
        panic!("Hypervisor Framework is only available on macOS");
    }

    println!("cargo:rustc-link-lib=framework=Hypervisor");

    // Only generate bindings if the feature is enabled
    #[cfg(feature = "generate-bindings")]
    {
        generate_bindings();
    }
}

#[cfg(feature = "generate-bindings")]
fn generate_bindings() {
    use std::process::Command;

    fn get_macos_sdk_path() -> Option<String> {
        if let Ok(output) = Command::new("xcrun").args(["--show-sdk-path"]).output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let path = path.trim().to_string();
                    if std::path::Path::new(&path).exists() {
                        return Some(path);
                    }
                }
            }
        }
        None
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    // Get the macOS SDK path
    let sdk_path = get_macos_sdk_path().expect("Could not find macOS SDK path");

    // Generate bindings
    let mut builder = bindgen::Builder::default().header("wrapper.h");

    builder = builder
        .clang_arg(format!("-isysroot{sdk_path}"))
        .clang_arg(format!("-F{sdk_path}/System/Library/Frameworks"))
        .clang_arg(format!(
            "-I{sdk_path}/System/Library/Frameworks/Hypervisor.framework/Headers"
        ));

    let bindings = builder
        // Allowlist the Hypervisor Framework functions and types
        .allowlist_function("hv_.*")
        .allowlist_type("hv_.*")
        .allowlist_var("HV_.*")
        // Generate bitfield methods for flag types
        .constified_enum_module("hv_exit_reason_t")
        // Use core instead of std for no_std compatibility
        .use_core()
        // Add derives for common traits
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_ord(true)
        .derive_partialeq(true)
        .derive_partialord(true)
        // Disable layout tests (can be flaky in CI)
        .layout_tests(false)
        // Parse callbacks for cargo integration
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to src/bindings/ directory
    let bindings_dir = PathBuf::from("src/bindings");
    std::fs::create_dir_all(&bindings_dir).expect("Could not create bindings directory");

    // Get macOS version for filename
    let macos_version = get_macos_version();
    let bindings_file = bindings_dir.join(format!("macos_{macos_version}.rs"));

    bindings
        .write_to_file(&bindings_file)
        .expect("Couldn't write bindings!");

    println!(
        "cargo:warning=Bindings written to: {}",
        bindings_file.display()
    );
}

#[cfg(feature = "generate-bindings")]
fn get_macos_version() -> String {
    use std::process::Command;

    if let Ok(output) = Command::new("sw_vers").args(["-productVersion"]).output() {
        if output.status.success() {
            if let Ok(version) = String::from_utf8(output.stdout) {
                let version = version.trim();
                // Convert "15.1.0" to "15_1"
                let parts: Vec<&str> = version.split('.').take(2).collect();
                return parts.join("_");
            }
        }
    }
    "unknown".to_string()
}
