use crate::executor_operation::ExecutorContext;

pub trait ProcessorChannelForwarder {
    fn send(&mut self, ctx: ExecutorContext);
}
