use std::sync::Arc;

use crate::types::Config;

#[inline]
pub fn generate(config: Arc<Config>) -> std::io::Result<()> {
    std::fs::create_dir_all(&config.output)?;
    let _ = std::fs::copy("src/helpers.ts", config.output.join("generated-helpers.ts"))?;
    Ok(())
}
