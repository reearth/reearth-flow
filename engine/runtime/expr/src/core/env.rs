use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::value::Value;

/// A single scope frame in the environment chain.
pub struct Frame {
    pub bindings: HashMap<String, Value>,
    /// Parent frame. Root frames are immutable.
    pub parent: Option<Env>,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("bindings", &self.bindings.keys().collect::<Vec<_>>())
            .field("has_parent", &self.parent.is_some())
            .finish()
    }
}

/// A reference-counted environment frame. Cloning shares the same frame.
pub type Env = Rc<RefCell<Frame>>;

/// Create a new frame with an optional parent link.
pub fn new_frame(parent: Option<Env>) -> Env {
    Rc::new(RefCell::new(Frame {
        bindings: HashMap::new(),
        parent,
    }))
}
