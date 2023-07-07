#[cfg(feature = "mpsolve")]
fn main()
{
    use std::env;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;

    // Check the standard include paths
    let local_dir = format!("{}/.local", env::var("HOME").unwrap());
    let local_include = format!("{local_dir}/include");
    let include_paths = vec![
        "/usr/include",
        "/usr/local/include",
        "/usr/include/x86_64-linux-gnu",
        "/usr/local/include/x86_64-linux-gnu",
        &local_include,
    ];

    let mut header_path = None;
    for path in &include_paths
    {
        let potential_path = PathBuf::from(path).join("mps/mps.h");
        if potential_path.exists()
        {
            header_path = Some(potential_path);
            break;
        }
    }
    let header_path = header_path.expect("Failed to find mps.h in include paths");
    let header_dir = header_path.parent().unwrap().parent().unwrap(); // get the directory containing the 'mps' directory

    // Generate bindings with bindgen
    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap()) // use absolute path to mps.h
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .clang_arg(format!("-I{}", header_dir.to_str().unwrap())) // add the parent directory to the include paths
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the `src/bindings.rs` file.
    let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let bindings_path = out_path.join("src/bindings.rs");

    {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&bindings_path)
            .expect("Couldn't open bindings!");

        writeln!(file, "#![allow(warnings)]\n").expect("Couldn't write warnings!");
        bindings
            .write(Box::new(file))
            .expect("Couldn't write bindings!");
    }

    let lib_directories = vec![
        "/usr/lib".to_owned(),
        "/usr/local/lib".to_owned(),
        format!("{local_dir}/lib"), // Replace with your actual home directory
    ];

    for directory in lib_directories
    {
        println!("cargo:rustc-link-search=native={}", directory);
    }

    println!("cargo:rustc-link-lib=dylib=mps"); // Or dynamic, depending on your setup

    // Link the MPSolve library
    println!("cargo:rerun-if-changed={}", header_path.display());
}

#[cfg(not(feature = "mpsolve"))]
fn main() {}
