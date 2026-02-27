fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &[
        "../../proto/gppn/v1/payment_message.proto",
        "../../proto/gppn/v1/routing.proto",
        "../../proto/gppn/v1/settlement.proto",
        "../../proto/gppn/v1/identity.proto",
        "../../proto/gppn/v1/trust.proto",
        "../../proto/gppn/v1/node.proto",
    ];
    let includes = &["../../proto"];

    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(proto_files, includes)?;

    Ok(())
}
