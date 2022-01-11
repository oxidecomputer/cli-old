/// This trait describes a command.
pub trait Command {
    fn run(&self, ctx: crate::context::Context);
}
