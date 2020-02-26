use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let linker = match (
        cfg!(feature = "xC-package"),
        cfg!(feature = "xE-package"),
        cfg!(feature = "xG-package"),
    ) {
        | (false, false, false)
        | (true, true, true)
        | (false, true, true)
        | (true, false, true)
        | (true, true, false) => {
            panic!("\n\nMust select exactly one package for linker script generation!\nChoices: 'xC-package', 'xE-package' or 'xG-package'\n\n");
        }

        (true, false, false) => include_bytes!("memory_xC.x"),
        (false, true, false) => include_bytes!("memory_xE.x"),
        (false, false, true) => include_bytes!("memory_xG.x"),
    };

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(linker)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=memory_xC.x");
    println!("cargo:rerun-if-changed=memory_xE.x");
    println!("cargo:rerun-if-changed=memory_xG.x");
}
