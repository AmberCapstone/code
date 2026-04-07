use regex::Regex;
use std::{
    io::{BufRead, BufReader, Result, Write},
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let proto_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../proto")
        .canonicalize()
        .unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let protos = [
        "sensor.proto",
        "sensor/alerts.proto",
        "sensor/backscatter.proto",
        "sensor/camera.proto",
        "sensor/fpga.proto",
        "sensor/fpga/flash.proto",
        "sensor/fpga/image.proto",
        "sensor/measure.proto",
        "sensor/nvm.proto",
        "base_station.proto",
    ];

    for proto_file in &protos {
        println!("cargo:rerun-if-changed={}", proto_root.join(proto_file).display());
    }

    let descriptor_path = out_dir.join("proto_descriptor.bin");
    let include_file = "proto_include.rs";

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .include_file(include_file)
        .compile_well_known_types()
        .compile_protos(&protos.map(|p| proto_root.join(p)), &[proto_root])?;

    let descriptor_set = std::fs::read(descriptor_path)?;
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .build(&["."])?;

    add_serde_to_include(&out_dir, &out_dir.join(include_file))?;

    Ok(())
}

/// prost_build generates a file including its generated files.
/// pbjson_build does not.
///
/// Search for prost_build include_file lines like
/// ```ignore
///      include!(concat!(env!("OUT_DIR"), "/myfile.rs"));
/// ```
/// Then if the corresponding pbjson_file exists, include it.
/// ```ignore
///     include!(concat!(env!("OUT_DIR"), "/myfile.rs"));
///     include!(concat!(env!("OUT_DIR"), "/myfile.serde.rs"));
/// ```
fn add_serde_to_include(out_dir: &Path, include_file: &Path) -> Result<()> {
    // Read the original file
    let file = std::fs::File::open(include_file)?;
    let lines: Vec<String> = BufReader::new(file).lines().collect::<Result<Vec<String>>>()?;

    let re = Regex::new(r#"^\s*include!\(concat!\(env!\("OUT_DIR"\), "/(?<name>[\w\.]+\.rs)"\)\);$"#).unwrap();

    let mut out_file = std::fs::File::create(include_file)?;
    for line in lines {
        writeln!(&mut out_file, "{}", line)?; // Copy the original line

        if let Some(captures) = re.captures(&line) {
            let prost_file = &captures["name"];
            let pbjson_file = prost_file.replace(".rs", ".serde.rs");

            if std::fs::exists(out_dir.join(&pbjson_file))? {
                writeln!(&mut out_file, "{}", line.replace(prost_file, &pbjson_file))?;
            }
        }
    }

    Ok(())
}
