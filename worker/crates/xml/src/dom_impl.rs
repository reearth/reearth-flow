use crate::node::RefNode;
use crate::traits::DOMImplementation;

#[doc(hidden)]
#[derive(Clone, Debug)]
pub(crate) struct Implementation;

const THIS_IMPLEMENTATION: &'static dyn DOMImplementation<NodeRef = RefNode> = &Implementation {};

pub fn get_implementation() -> &'static dyn DOMImplementation<NodeRef = RefNode> {
    THIS_IMPLEMENTATION
}
