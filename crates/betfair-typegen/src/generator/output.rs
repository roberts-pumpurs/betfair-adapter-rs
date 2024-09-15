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

    /// # Write the generated output to a file
    /// TODO: add example usage code
    pub fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let root_name = path.as_ref().join("mod.rs");
        std::fs::create_dir(path.as_ref()).unwrap_or_default();
        std::fs::write(&root_name, self.root.to_string().as_bytes()).unwrap();

        // format the generated code
        let mut cmd = std::process::Command::new("rustfmt");
        cmd.arg("--emit")
            .arg("files")
            .arg(root_name)
            .spawn()
            .unwrap();

        for (module_name, submodule) in &self.submodules {
            // Write out the submodule
            let submodule_file = path.as_ref().join(format!("{module_name}.rs"));
            std::fs::write(&submodule_file, submodule.to_string().as_bytes()).unwrap();

            let mut cmd = std::process::Command::new("rustfmt");
            cmd.arg("--emit")
                .arg("files")
                .arg(submodule_file)
                .spawn()
                .unwrap();
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
    pub fn submodules_mut(&mut self) -> &mut Vec<(String, TokenStream)> {
        &mut self.submodules
    }

    /// Returns the submodules as a mutable slice of tuples of the module name and the `TokenStream`
    pub fn root_mut(&mut self) -> &mut TokenStream {
        &mut self.root
    }
}
