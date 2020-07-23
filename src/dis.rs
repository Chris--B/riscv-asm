use std::path::Path;

use gumdrop::Options;

#[derive(Debug, Options)]
struct MyOptions {
    /// Print the help message and exit
    #[options()]
    help: bool,

    /// "Use equivilent psuedo instructions when possible"
    #[options(default = "true")]
    allow_psuedo: bool,

    /// Path to a RISC-V elf to disassemble
    #[options(free)]
    input: String,

    /// Path to write disassembled output into
    ///
    /// If unspecified, this is derived from the input file
    #[options()]
    output: Option<String>,
}

impl MyOptions {
    /// Parse args from argv, resolve extra steps, or exit trying.
    fn new() -> Self {
        let mut opts = MyOptions::parse_args_default_or_exit();
        // Some options have extra rules so we resolve them in a second pass.
        opts.resolve_extras();

        opts
    }

    /// Resolves extra options
    fn resolve_extras(&mut self) {
        // This path may optionally be specified directly.
        // When it's not, we need use the input file to derive an output.
        if self.output.is_none() {
            let input_path: &Path = &Path::new(&self.input);
            let file_stem: &str = input_path
                .file_stem()
                .expect("Failed to find file stem of input file")
                .to_str()
                .expect("file stem of input file wasn't valid utf8");

            self.output = Some(format!("./{}.s", file_stem));
        }
    }
}

fn main() {
    let opts = MyOptions::new();
    dbg!(&opts);
}
