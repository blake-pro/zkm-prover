fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto/src/proto/include/v1/includes.proto");
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .type_attribute(
            ".",
            "#[derive(serde_derive::Serialize, serde_derive::Deserialize)]",
        )
        .compile(
            &["../proto/src/proto/include/v1/includes.proto"],
            &["../proto/src/proto"],
        )?;
    Ok(())
}
