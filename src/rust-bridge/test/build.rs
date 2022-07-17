use std::collections::HashSet;
use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-env-changed=DDNET_TEST_LIBRARIES");
    println!("cargo:rerun-if-env-changed=DDNET_TEST_NO_LINK");
    if env::var_os("DDNET_TEST_NO_LINK").is_some() {
        return;
    }
    if env::var_os("CARGO_FEATURE_LINK_TEST_LIBRARIES").is_some() {
        let libraries = env::var("DDNET_TEST_LIBRARIES")
            .expect("environment variable DDNET_TEST_LIBRARIES required but not found");
        let mut seen_library_dirs = HashSet::new();
        for library in libraries.split(':') {
            let library = Path::new(library);
            if let Some(parent) = library.parent() {
                let parent = parent.to_str().expect("should have errored earlier");
                if !seen_library_dirs.contains(&parent) {
                    println!("cargo:rustc-link-search={}", parent);
                    seen_library_dirs.insert(parent);
                }
            }
            let mut name = library.file_stem()
                .expect("library name")
                .to_str()
                .expect("should have errored earlier");
            if name.starts_with("lib") {
                name = &name[3..];
            }
            println!("cargo:rustc-link-lib={}", name);
        }
        //println!("cargo:rustc-link-lib=stdc++");
    }
}
