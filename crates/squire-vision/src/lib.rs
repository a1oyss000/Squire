pub mod capture;
pub mod matcher;
pub mod ocr;

pub fn check_dependencies() -> squire_error::Result<()> {
    tracing::info!("Vision dependencies check passed");
    Ok(())
}
