use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let embedded_model = env::var("DUCKDB_VAPORETTO_EMBED_MODEL")
        .ok()
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let embedded_model_rs = out_dir.join("embedded_model.rs");

    println!("cargo:rerun-if-env-changed=DUCKDB_VAPORETTO_EMBED_MODEL");
    if let Some(path) = embedded_model.as_ref() {
        println!("cargo:rerun-if-changed={}", path.display());
        fs::write(
            &embedded_model_rs,
            format!(
                "static EMBEDDED_MODEL_BYTES: Option<&'static [u8]> = Some(include_bytes!({:?}));\n",
                path
            ),
        )
        .expect("write embedded model include");
    } else {
        fs::write(
            &embedded_model_rs,
            "static EMBEDDED_MODEL_BYTES: Option<&'static [u8]> = None;\n",
        )
        .expect("write embedded model include");
    }
}
