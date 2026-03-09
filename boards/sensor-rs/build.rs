use std::{io::Result, path::Path};

fn generate_proto() -> Result<()> {
    let path = Path::new("proto");

    let mut g = micropb_gen::Generator::new();
    g.use_container_heapless()
        .single_oneof_msg_as_enum(true)
        .add_protoc_arg(format!("-I{}", path.display()));

    // generator doesn't follow -I argument path for config files
    g.parse_config_file(&path.join("sensor/flash.toml"), "sensor.flash")
        .map_err(std::io::Error::other)?;

    g.compile_protos(
        &["sensor.proto", "sensor/flash.proto"],
        std::env::var("OUT_DIR").unwrap() + "/generated_proto.rs",
    )
    .expect("micropb failed");

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    generate_proto()?;

    Ok(())
}
