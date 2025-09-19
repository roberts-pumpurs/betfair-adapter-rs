use std::path::Path;
use std::process::Command;
use std::{fs, io};

use proc_macro2::TokenStream;

/// The generated output
pub struct GeneratedOutput {
    root: TokenStream,
    submodules: Vec<(String, TokenStream)>,
}

impl GeneratedOutput {
    pub(crate) fn new() -> Self {
        Self {
            root: quote::quote! {},
            submodules: vec![],
        }
    }

    /// Writes the generated output to a specified file path.
    ///
    /// This function creates a directory at the given path (if it doesn't exist),
    /// writes the root module to `mod.rs`, and formats the generated code using `rustfmt`.
    /// It also writes each submodule to its respective file and formats them.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a path where the output files will be written.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure of the file operations.
    ///
    /// # Errors
    ///
    /// This function will return an `std::io::Error` if:
    /// - The directory cannot be created.
    /// - Writing to the file fails.
    /// - The `rustfmt` command fails to execute.
    ///
    /// # Example
    ///
    /// ```
    /// let output = GeneratedOutput::new();
    /// let result = output.write_to_file("output_directory");
    /// match result {
    ///     Ok(_) => println!("Files written successfully!"),
    ///     Err(e) => eprintln!("Error writing files: {}", e),
    /// }
    /// ```
    /// Need Help to fix this error
    #[expect(clippy::pattern_type_mismatch)]
    #[tracing::instrument(skip_all)]
    pub fn write_to_file<T: AsRef<Path>>(&self, path: T) -> io::Result<()> {
        let path = path.as_ref();
        let root_path = path.join("mod.rs");
        let root = self.root.to_string();
        let bytes = root.as_bytes();
        let bytes_len = bytes.len();
        tracing::debug!(path = ?root_path, content_len = ?bytes_len, "writing mod.rs");

        fs::create_dir_all(path)?;
        fs::write(&root_path, bytes)?;

        for (module_name, submodule) in &self.submodules {
            // Write out the submodule
            tracing::info!(?module_name, "writing module to file");
            let submodule_file = path.join(format!("{module_name}.rs"));
            fs::write(&submodule_file, submodule.to_string().as_bytes())?;

            // Format the submodule file
            self.format_file(&submodule_file)?;
        }

        // Format the generated code for root
        self.format_file(&root_path)?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn format_file(&self, file_path: &Path) -> io::Result<()> {
        let status = Command::new("rustfmt")
            .arg("--emit")
            .arg("files")
            .arg(file_path)
            .status()?;

        if !status.success() {
            tracing::error!(?file_path, "cannot format file");
            return Err(io::Error::other(
                format!("rustfmt failed for {}", file_path.display()),
            ));
        }
        Ok(())
    }

    /// Returns the root module file as a `TokenStream`
    #[must_use]
    pub const fn root_mod(&self) -> &TokenStream {
        &self.root
    }

    /// Returns the submodules as a slice of tuples of the module name and the `TokenStream`
    #[must_use]
    pub fn submodules(&self) -> &[(String, TokenStream)] {
        self.submodules.as_ref()
    }

    /// Returns the root module file as a mutable `TokenStream`
    pub const fn submodules_mut(&mut self) -> &mut Vec<(String, TokenStream)> {
        &mut self.submodules
    }

    /// Returns the submodules as a mutable slice of tuples of the module name and the `TokenStream`
    pub const fn root_mut(&mut self) -> &mut TokenStream {
        &mut self.root
    }
}
