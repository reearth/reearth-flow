use std::borrow::Cow;

use crossbeam::channel::{Receiver, Select};

use super::name::Name;
use crate::{
    errors::ExecutionError,
    executor_operation::{ExecutorContext, ExecutorOperation, NodeContext},
};

pub trait ReceiverLoop: Name {
    /// Returns input channels to this node. Will be called exactly once in [`receiver_loop`].
    fn receivers(&mut self) -> Vec<Receiver<ExecutorOperation>>;
    /// Returns the name of the receiver at `index`. Used for logging.
    fn receiver_name(&self, index: usize) -> Cow<str>;
    /// Responds to `op` from the receiver at `index`.
    fn on_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError>;
    /// Responds to `terminate`.
    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError>;

    fn receiver_loop(mut self) -> Result<(), ExecutionError>
    where
        Self: Sized,
    {
        let receivers = self.receivers();
        let mut is_terminated = vec![false; receivers.len()];
        let mut sel = init_select(&receivers);

        loop {
            let index = sel.ready();
            let op = receivers[index]
                .recv()
                .map_err(|_| ExecutionError::CannotReceiveFromChannel)?;

            match op {
                ExecutorOperation::Op { ctx } => {
                    self.on_op(ctx)?;
                }
                ExecutorOperation::Terminate { ctx } => {
                    is_terminated[index] = true;
                    sel.remove(index);
                    if is_terminated.iter().all(|value| *value) {
                        self.on_terminate(ctx)?;
                        return Ok(());
                    }
                }
            }
        }
    }
}

pub(crate) fn init_select(receivers: &Vec<Receiver<ExecutorOperation>>) -> Select {
    let mut sel = Select::new();
    for r in receivers {
        sel.recv(r);
    }
    sel
}
