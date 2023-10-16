use crate::error::UserScriptError;
use crate::transpiler::Transpiler;
use fractal_gui::interface::Interface;
use fractal_gui::interface_holder::InterfaceHolder;
use libloading::{Library, Symbol};
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct Loader<'a>
{
    pub toml_path: &'a Path,
    pub crate_path: &'a Path,
}

impl<'a> Loader<'a>
{
    fn transpile_toml(&self) -> Result<(), UserScriptError>
    {
        let transpiler = Transpiler::from_toml_path(&self.toml_path)?;
        let rust_filename = format!("src/generated/user_profile.rs");
        let rust_path = self.crate_path.join(rust_filename);
        transpiler.write(&rust_path)
    }

    fn build(&self) -> Result<(), UserScriptError>
    {
        Command::new("cargo")
            .args(&["fmt"])
            .current_dir(&self.crate_path);

        let status = Command::new("cargo")
            .args(&["build", "-p", "user_profiles"])
            // .args(&["build", "--release", "-p", "user_profiles"])
            .current_dir(&self.crate_path)
            .status()
            .expect("Failed to execute command");

        if status.success()
        {
            Ok(())
        }
        else
        {
            Err(UserScriptError::CompilationFailed)
        }
    }

    unsafe fn load<'i>(self, img_height: usize) -> Result<InterfaceHolder<'i>, UserScriptError>
    {
        let lib_path = self
            .crate_path
            .join("../target/debug/libuser_profiles.so");

        // Load the dynamic library
        let lib = Library::new(lib_path).map_err(UserScriptError::ErrorLoadingLibrary)?;

        // Get the constructor function from the dynamic library
        type Constructor = unsafe fn() -> *mut dyn Interface;
        let constructor: Symbol<Constructor> = lib
            .get(b"create_interface")
            .map_err(UserScriptError::ErrorLoadingLibrary)?;

        let mut interface = Box::from_raw(constructor());
        interface.change_height(img_height);

        let holder = InterfaceHolder::new(interface, lib);

        // Convert the raw pointer to a Box
        Ok(holder)
    }

    pub unsafe fn run<'i>(self, img_height: usize) -> Result<InterfaceHolder<'i>, UserScriptError>
    {
        println!("Transpiling...");
        self.transpile_toml()?;

        println!("Building...");
        self.build()?;

        println!("Loading...");
        self.load(img_height)
    }
}
