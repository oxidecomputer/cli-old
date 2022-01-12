use anyhow::Result;

/// This trait describes a command.
pub trait Command {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()>;
}
