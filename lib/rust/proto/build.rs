use std::{io::Result, path::Path};

fn main() -> Result<()> {
    let proto_path = Path::new("../../../proto");

    let protos = ["sensor/flash.proto", "sensor.proto"];

    prost_build::Config::new()
        .include_file("_proto_include.rs")
        .compile_protos(&protos.map(|p| proto_path.join(p)), &[proto_path])?;

    Ok(())
}
