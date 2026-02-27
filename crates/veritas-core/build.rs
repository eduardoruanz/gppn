fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &[
        "../../proto/veritas/v1/identity.proto",
        "../../proto/veritas/v1/credential.proto",
        "../../proto/veritas/v1/proof.proto",
        "../../proto/veritas/v1/trust.proto",
        "../../proto/veritas/v1/node.proto",
    ];
    let includes = &["../../proto"];

    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(proto_files, includes)?;

    Ok(())
}
