use anyhow::Result;

/// This trait describes a command.
#[async_trait::async_trait]
pub trait Command {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()>;
}
