#[cfg(feature = "tui")]
use crate::tui;
#[cfg(feature = "tui")]
use tokio::runtime::Runtime;

pub fn run() -> anyhow::Result<()> {
    #[cfg(feature = "tui")]
    {
        let rt = Runtime::new()?;
        return rt.block_on(tui::run_tui());
    }
    #[allow(unreachable_code)]
    Ok(())
}
