fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &[
            "../../proto/skill.proto",
            "../../proto/spine.proto",
            "../../proto/citadel.proto",
            "../../proto/cass.proto",
        ],
        &["../../proto"],
    )?;
    Ok(())
}
