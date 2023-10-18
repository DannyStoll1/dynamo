use crate::error::ScriptError;
use crate::transpiler::Transpiler;
use fractal_gui::interface::Interface;
use fractal_gui::interface_holder::InterfaceHolder;
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::process::Command;

fn file_hash<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error>
{
    let bytes = std::fs::read(path.as_ref())?;
    Ok(sha256::digest(bytes))
}

#[derive(Debug)]
pub struct Loader<'a>
{
    pub toml_path: &'a Path,
    pub output_path: &'a Path,
    pub image_height: usize,
    lib_path: Option<PathBuf>,
}

impl<'a> Loader<'a>
{
    #[must_use]
    pub fn new(toml_path: &'a Path, image_height: usize) -> Self
    {
        let output_path = Path::new("scripting/output");
        Self {
            toml_path,
            output_path,
            image_height,
            lib_path: None,
        }
    }

    fn transpile_toml(&self) -> Result<(), ScriptError>
    {
        let transpiler = Transpiler::from_toml_path(self.toml_path)?;
        let rust_path = self.output_path.join("src/generated");
        transpiler.write(&rust_path)
    }

    fn get_lib_path(&mut self) -> &PathBuf
    {
        self.lib_path.get_or_insert_with(|| {
            let lib_id = &file_hash(self.toml_path).unwrap_or_default()[0..12];
            self.output_path
                .join(format!("../compiled/libscripts_{}.so", lib_id))
        })
    }

    fn build(&mut self) -> Result<(), ScriptError>
    {
        Command::new("cargo")
            .args(["fmt", "-p", "transpiled-scripts"])
            .current_dir(self.output_path);

        #[cfg(debug_assertions)]
        let status = Command::new("cargo")
            .args(["build", "-p", "transpiled-scripts"])
            .current_dir(self.output_path)
            .status()
            .map_err(ScriptError::CargoCommandFailed)?;

        #[cfg(not(debug_assertions))]
        let status = Command::new("cargo")
            .args(&["build", "--release", "-p", "transpiled-scripts"])
            .current_dir(&self.output_path)
            .status()
            .map_err(ScriptError::CargoCommandFailed)?;

        #[cfg(debug_assertions)]
        let orig_lib_path = self
            .output_path
            .join("../../target/debug/libtranspiled_scripts.so");

        #[cfg(not(debug_assertions))]
        let orig_lib_path = self
            .output_path
            .join("../../target/release/libtranspiled_scripts.so");

        std::fs::rename(orig_lib_path, self.get_lib_path())
            .map_err(ScriptError::ErrorMovingLibrary)?;

        if status.success()
        {
            Ok(())
        }
        else
        {
            Err(ScriptError::CompilationFailed)
        }
    }

    /// Load the library in `scripting/compiled` created by `self.build`.
    ///
    /// # Safety
    ///
    /// This method makes no assumptions about the library file
    /// loaded into memory. Under normal circumstances, this library should always match output
    /// created by the Rust compiler in `self.build`.
    ///
    /// However, no checks are currently performed to ensure that the flags passed to
    /// `cargo` in `self.build` match those with which `fractal-explorer` was compiled.
    /// If these flags do not match, the ABIs will likely be incompatible, leading to undefined
    /// behavior.
    unsafe fn load<'i>(mut self) -> Result<InterfaceHolder<'i>, ScriptError>
    {
        // Load the dynamic library
        let lib = Library::new(self.get_lib_path()).map_err(ScriptError::ErrorLoadingLibrary)?;

        // Get the constructor function from the dynamic library
        type Constructor = unsafe fn() -> *mut dyn Interface;
        let constructor: Symbol<Constructor> = lib
            .get(b"create_interface")
            .map_err(ScriptError::ErrorLoadingLibrary)?;

        let mut interface = Box::from_raw(constructor());
        interface.change_height(self.image_height);

        let holder = InterfaceHolder::new(interface, lib);

        // Convert the raw pointer to a Box
        Ok(holder)
    }

    /// Transpile the user script into Rust, compile it to a library, and load the library together
    /// with the interface defined by the script.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure the same flags are passed to `cargo` during `build`
    /// as were used to compile `fractal-explorer`. See the safety notes for `load`.
    ///
    /// Additionally, we must ensure that the library remains in memory for as long as the
    /// interface does. The InterfaceHolder structure provides this protection.
    ///
    /// Finally, the OS might have cached an older version of the library, since the libraries are
    /// dynamically written and loaded at runtime. To prevent this, we append a hash to the library
    /// filename based on the content of the user's script.
    pub unsafe fn run<'i>(mut self) -> Result<InterfaceHolder<'i>, ScriptError>
    {
        println!("Transpiling script...");
        self.transpile_toml()?;

        println!("Building script...");
        self.build()?;

        println!("Loading script...");
        self.load()
    }

    /// Same as `run`, but avoid recompiling if the script's hash matches an existing library file.
    ///
    /// # Safety
    ///
    /// While this method would be quite useful to reduce load times for reused scripts, it is also
    /// very unstable. In particular, if fractal-explorer is updated or ported to a new machine,
    /// but the user script libraries are not cleaned (e.g. by running `clear_scripts.sh`),
    /// then the old libraries may have an ABI mismatch with the newer `fractal-explorer`, leading
    /// to undefined behavior.
    ///
    /// FIXME: Add a build script to clear old script libraries from `scripting/compiled` whenever `fractal-explorer` is recompiled.
    pub unsafe fn run_lazy<'i>(mut self) -> Result<InterfaceHolder<'i>, ScriptError>
    {
        if self.get_lib_path().exists()
        {
            println!("Library found, skipping compilation.");
        }
        else
        {
            println!("Transpiling script...");
            self.transpile_toml()?;

            println!("Building script...");
            self.build()?;
        }

        println!("Loading script...");
        self.load()
    }
}
