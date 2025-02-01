use crate::executor_operation::ExecutorContext;

pub trait ProcessorChannelForwarder {
    fn node_id(&self) -> String;
    fn send(&mut self, ctx: ExecutorContext);
}
