use crate::Result;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ServiceMode {
    Console,
    WindowsService,
}

pub fn run_indexer_service(mode: ServiceMode) -> Result<()> {
    let _ = mode;
    super::unsupported("Windows Service host")?;
    unreachable!()
}
