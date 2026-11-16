use std::{env, path::Path};

fn main() {
    println!("cargo:rerun-if-file-changed=.env");

    if dotenv::dotenv().is_ok() {
        println!("cargo:warning=Successfully loaded .env file");
    }

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set by Cargo");

    tauri_build::build()
}
