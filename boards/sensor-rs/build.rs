use std::{io::Result, path::Path};

fn generate_proto() -> Result<()> {
    let path = Path::new("proto");

    let mut g = micropb_gen::Generator::new();
    g.use_container_heapless()
        .single_oneof_msg_as_enum(true)
        .add_protoc_arg(format!("-I{}", path.display()));

    let protos = [
        "sensor.proto",
        "sensor/alerts.proto",
        "sensor/camera.proto",
        "sensor/fpga.proto",
        "sensor/fpga/flash.proto",
        "sensor/fpga/image.proto",
        "sensor/measure.proto",
        "sensor/nvm.proto",
    ];

    // Check for config files
    for proto in protos.map(Path::new) {
        let config = path.join(proto.with_extension("toml"));
        if std::fs::exists(&config)? {
            // Assumes "x/y.[proto,toml]" is package "x.y"
            let package = proto.with_extension("").to_str().unwrap().replace('/', ".");
            g.parse_config_file(&config, &package).map_err(std::io::Error::other)?;
        }
    }

    g.compile_protos(&protos, std::env::var("OUT_DIR").unwrap() + "/generated_proto.rs")
        .expect("micropb failed");

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rustc-link=search=.");
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    generate_proto()?;

    Ok(())
}
