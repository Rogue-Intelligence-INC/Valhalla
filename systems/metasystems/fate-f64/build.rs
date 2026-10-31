fn main() {
    println!("cargo:rerun-if-env-changed=CUDA_PATH");
    if std::env::var("CARGO_FEATURE_NATIVE_CUDA").is_err() {
        return;
    }
    let cuda = std::env::var("CUDA_PATH").unwrap_or_else(|_| {
        r"C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4".to_string()
    });
    let lib = std::path::Path::new(&cuda).join("lib").join("x64");
    println!("cargo:rustc-link-search=native={}", lib.display());
}
