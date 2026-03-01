fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile_protos(
            &["../../proto/kernel.proto", "../../proto/skill.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
