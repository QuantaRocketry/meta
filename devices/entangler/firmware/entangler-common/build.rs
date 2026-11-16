// Copyright © 2025 David Haig
// SPDX-License-Identifier: MIT

fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let library_paths = std::collections::HashMap::from([(
        "sleek-ui".to_string(),
        manifest_dir.join("ui/libraries/sleek-ui/ui/sleek-ui"),
    )]);
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer)
        .with_library_paths(library_paths);
    slint_build::compile_with_config("./ui/main.slint", config).unwrap();
    slint_build::print_rustc_flags().unwrap();
}
