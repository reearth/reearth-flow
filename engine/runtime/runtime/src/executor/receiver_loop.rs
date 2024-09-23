use std::borrow::Cow;

use crossbeam::channel::{Receiver, Select};

use crate::{
    errors::ExecutionError,
    executor_operation::{ExecutorContext, ExecutorOperation, NodeContext},
};

pub trait ReceiverLoop {
    /// Returns input channels to this node. Will be called exactly once in [`receiver_loop`].
    fn receivers(&mut self) -> Vec<Receiver<ExecutorOperation>>;
    /// Returns the name of the receiver at `index`. Used for logging.
    fn receiver_name(&self, index: usize) -> Cow<str>;
    /// Responds to `op` from the receiver at `index`.
    fn on_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError>;
    /// Responds to `terminate`.
    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError>;

    fn receiver_loop(self) -> Result<(), ExecutionError>
    where
        Self: Sized;
}

pub(crate) fn init_select(receivers: &Vec<Receiver<ExecutorOperation>>) -> Select {
    let mut sel = Select::new();
    for r in receivers {
        sel.recv(r);
    }
    sel
}
