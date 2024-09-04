use std::{
    env,
    fs::{read, write},
    path::Path,
};

fn main() {
    if env::var_os("CARGO_FEATURE_IPL3").is_some() {
        let Some(libdragon) = env::var_os("N64_INST") else {
            panic!("Cannot find libdragon path (required for feature `ipl3'); ensure N64_INST is set correctly");
        };

        let out_dir = env::var_os("OUT_DIR").unwrap();

        let dest_path = Path::new(&out_dir).join("ipl3.rs");

        let ipl3_file = Path::new(&libdragon).join("boot/bin/ipl3_compat.z64");

        let Ok(ipl3) = read(&ipl3_file) else {
            panic!("Failed to read IPL3 file from {}; ensure libdragon is checked out on the `preview' branch", ipl3_file.display());
        };

        if let Err(e) = write(&dest_path, format!("{:?}", &ipl3[0x40..0x1000])) {
            panic!(
                "Failed to write IPL3 file to {}; error: {e}",
                dest_path.display()
            )
        };

        println!("cargo:rerun-if-changed={}", ipl3_file.display());
    }
}
