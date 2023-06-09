use std::env;
use std::path::PathBuf;

#[cfg(feature = "mpsolve")]
fn main()
{
    // Get include paths from environment variables
    let include_paths = env::var("C_INCLUDE_PATH").unwrap_or_default();

    // Split paths by ':'
    let mut include_paths: Vec<_> = include_paths.split(':').map(String::from).collect();

    // Add common paths
    include_paths.push("/usr/include".to_string());
    include_paths.push("/usr/local/include".to_string());
    include_paths.push(env::var("HOME").unwrap() + "/.local/include");

    let mut header_path = None;
    for path in include_paths
    {
        let potential_path = PathBuf::from(&path).join("mps.h");
        let potential_subdir_path = PathBuf::from(&path).join("mps/mps.h"); // Check within potential 'mps' subdirectory
        if potential_path.exists()
        {
            header_path = Some(potential_path);
            break;
        }
        else if potential_subdir_path.exists()
        {
            header_path = Some(potential_subdir_path);
            break;
        }
    }
    let header_path = header_path.expect("Failed to find mps.h in include paths");

    // Get directory of mps.h
    let header_dir = header_path.parent().unwrap();

    // Generate bindings with bindgen
    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap()) // use found path to mps.h
        .clang_arg(format!("-I{}", header_dir.to_str().unwrap())) // Add directory of mps.h to clang include paths
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the `src/bindings.rs` file.
    let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("src/bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link the MPSolve library
    println!("cargo:rerun-if-changed={}", header_path.display());
}

#[cfg(not(feature = "mpsolve"))]
fn main() {}
