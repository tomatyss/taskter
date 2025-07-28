#[cfg(feature = "tui")]
use crate::tui;

pub fn run() -> anyhow::Result<()> {
    #[cfg(feature = "tui")]
    {
        return tui::run_tui();
    }
    #[allow(unreachable_code)]
    Ok(())
}
