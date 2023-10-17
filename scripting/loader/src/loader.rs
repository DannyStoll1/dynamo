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
    Ok(sha256::digest(&bytes))
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
        let transpiler = Transpiler::from_toml_path(&self.toml_path)?;
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
            .args(&["fmt", "-p", "transpiled-scripts"])
            .current_dir(&self.output_path);

        #[cfg(debug_assertions)]
        let status = Command::new("cargo")
            .args(&["build", "-p", "transpiled-scripts"])
            .current_dir(&self.output_path)
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

    pub unsafe fn run<'i>(mut self) -> Result<InterfaceHolder<'i>, ScriptError>
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
