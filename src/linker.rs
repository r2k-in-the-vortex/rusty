// This file is based on code from the Mun Programming Language
// https://github.com/mun-lang/mun

use which::which;

use crate::diagnostics::Diagnostic;
use std::{
    error::Error,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Linker {
    errors: Vec<LinkerError>,
    linker: Box<dyn LinkerInterface>,
}

trait LinkerInterface {
    fn add_obj(&mut self, path: &str);
    fn add_lib(&mut self, path: &str);
    fn add_lib_path(&mut self, path: &str);
    fn add_sysroot(&mut self, path: &str);
    fn build_shared_object(&mut self, path: &str);
    fn build_exectuable(&mut self, path: &str);
    fn build_relocatable(&mut self, path: &str);
    fn finalize(&mut self) -> Result<(), LinkerError>;
}

impl Linker {
    pub fn new(target: &str, linker: Option<&str>) -> Result<Linker, LinkerError> {
        let target_os = target.split('-').collect::<Vec<&str>>()[2];

        Ok(Linker {
            errors: Vec::default(),
            linker: match linker {
                Some(linker) => Box::new(CcLinker::new(linker)),

                // TODO: Linker for Windows is missing, see also:
                // https://github.com/PLC-lang/rusty/pull/702/files#r1052446296
                None => match target_os {
                    "win32" | "windows" => return Err(LinkerError::Target(target_os.into())),

                    _ => Box::new(LdLinker::new()),
                },
            },
        })
    }

    /// Add an object file or static library to linker input
    pub fn add_obj<'a>(&'a mut self, path: &str) -> &'a mut Self {
        self.linker.add_obj(path);
        self
    }

    /// Add a library seaBoxh path to look in for libraries
    pub fn add_lib_path<'a>(&'a mut self, path: &str) -> &'a mut Self {
        self.linker.add_lib_path(path);
        self
    }

    /// Add a library path to look in for libraries
    pub fn add_lib<'a>(&'a mut self, path: &str) -> &'a mut Self {
        self.linker.add_lib(path);
        self
    }

    /// Add path to system root
    pub fn add_sysroot<'a>(&'a mut self, path: &str) -> &'a mut Self {
        self.linker.add_sysroot(path);
        self
    }

    /// Set the output file and run the linker to generate a shared object
    pub fn build_shared_obj(&mut self, path: &Path) -> Result<(), LinkerError> {
        if let Some(file) = self.get_str_from_path(path) {
            self.linker.build_shared_object(file);
            self.linker.finalize()?;
        }
        Ok(())
    }

    /// Set the output file and run the linker to generate an executable
    pub fn build_exectuable(&mut self, path: &Path) -> Result<(), LinkerError> {
        if let Some(file) = self.get_str_from_path(path) {
            self.linker.build_exectuable(file);
            self.linker.finalize()?;
        }
        Ok(())
    }

    /// Set the output file and run the linker to generate a relocatable object for further linking
    pub fn build_relocatable(&mut self, path: &Path) -> Result<(), LinkerError> {
        if let Some(file) = self.get_str_from_path(path) {
            self.linker.build_relocatable(file);
            self.linker.finalize()?;
        }
        Ok(())
    }

    /// Check if the path is valid, log an error if it wasn't
    fn get_str_from_path<'a>(&mut self, path: &'a Path) -> Option<&'a str> {
        let filepath = path.to_str();
        if filepath.is_none() {
            self.errors.push(LinkerError::Path(path.into()));
        }
        filepath
    }
}

struct CcLinker {
    args: Vec<String>,
    linker: String,
}

impl CcLinker {
    fn new(linker: &str) -> CcLinker {
        CcLinker { args: Vec::default(), linker: linker.to_string() }
    }
}

impl LinkerInterface for CcLinker {
    fn add_obj(&mut self, path: &str) {
        self.args.push(path.into());
    }

    fn add_lib_path(&mut self, path: &str) {
        self.args.push(format!("-L{}", path));
    }

    fn add_lib(&mut self, path: &str) {
        self.args.push(format!("-l{}", path));
    }

    fn add_sysroot(&mut self, path: &str) {
        self.args.push(format!("--sysroot={}", path));
    }

    fn build_shared_object(&mut self, path: &str) {
        self.args.push("--shared".into());
        self.args.push("-o".into());
        self.args.push(path.into());
    }

    fn build_exectuable(&mut self, path: &str) {
        self.args.push("-o".into());
        self.args.push(path.into());
    }

    fn build_relocatable(&mut self, path: &str) {
        self.args.push("-relocatable".into());
        self.args.push("-o".into());
        self.args.push(path.into());
    }

    fn finalize(&mut self) -> Result<(), LinkerError> {
        let linker_location = which(&self.linker)
            .map_err(|e| LinkerError::Link(format!("{} for linker: {}", e, &self.linker)))?;

        #[cfg(feature = "debug")]
        println!("Linker command : {} {}", linker_location.to_string_lossy(), self.args.join(" "));

        let status = Command::new(linker_location).args(&self.args).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(LinkerError::Link("An error occured during linking".to_string()))
        }
    }
}

struct LdLinker {
    args: Vec<String>,
}

impl LdLinker {
    fn new() -> LdLinker {
        LdLinker { args: Vec::default() }
    }
}

impl LinkerInterface for LdLinker {
    fn add_obj(&mut self, path: &str) {
        self.args.push(path.into());
    }

    fn add_lib_path(&mut self, path: &str) {
        self.args.push(format!("-L{}", path));
    }

    fn add_lib(&mut self, path: &str) {
        self.args.push(format!("-l{}", path));
    }

    fn add_sysroot(&mut self, path: &str) {
        self.args.push(format!("--sysroot={}", path));
    }

    fn build_shared_object(&mut self, path: &str) {
        self.args.push("--shared".into());
        self.args.push("-o".into());
        self.args.push(path.into());
    }

    fn build_exectuable(&mut self, path: &str) {
        self.args.push("-o".into());
        self.args.push(path.into());
    }

    fn build_relocatable(&mut self, path: &str) {
        self.args.push("-relocatable".into());
        self.args.push("-o".into());
        self.args.push(path.into());
    }

    fn finalize(&mut self) -> Result<(), LinkerError> {
        #[cfg(feature = "debug")]
        println!("Linker arguments : {}", self.args.join(" "));

        lld_rs::link(lld_rs::LldFlavor::Elf, &self.args).ok().map_err(LinkerError::Link)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LinkerError {
    /// Error emitted by the linker
    Link(String),

    /// Invalid target
    Target(String),

    /// Error in path conversion
    Path(PathBuf),
}

impl From<LinkerError> for Diagnostic {
    fn from(error: LinkerError) -> Self {
        match error {
            LinkerError::Link(e) => Diagnostic::link_error(&e),
            LinkerError::Path(path) => {
                Diagnostic::link_error(&format!("path contains invalid UTF-8 characters: {}", path.display()))
            }
            LinkerError::Target(tgt) => {
                Diagnostic::link_error(&format!("linker not available for target platform: {}", tgt))
            }
        }
    }
}

impl<T: Error> From<T> for LinkerError {
    fn from(e: T) -> Self {
        LinkerError::Link(e.to_string())
    }
}

#[test]
fn windows_target_triple_should_result_in_error() {
    for target in vec![
        "x86_64-pc-windows-gnu",
        "x86_64-pc-win32-gnu",
        "aarch64-pc-windows-gnu",
        "aarch64-pc-win32-gnu",
        "i686-pc-windows-gnu",
        "i686-pc-win32-gnu",
    ] {
        assert!(Linker::new(target, None).is_err());
    }
}

#[test]
fn non_windows_target_triple_should_result_in_ok() {
    for target in vec!["x86_64-pc-linux-gnu", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin"] {
        assert!(Linker::new(target, None).is_ok());
    }
}
