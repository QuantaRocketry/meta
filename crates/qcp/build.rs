use glob::glob;
use std::{io::Result, path::PathBuf};
fn main() -> Result<()> {
    let path_iter = glob("proto/*.proto").expect("Couldn't find proto files");
    let protos: Vec<PathBuf> = path_iter.filter_map(|p| p.ok()).collect();
    prost_build::compile_protos(&protos, &["proto/"])?;

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
