use std::sync::Arc;

use anathema_values::{BucketRef, ScopeId};

use crate::expression::{ControlFlow, FromContext, EvaluationContext};
use crate::{Node, Nodes};

pub struct ControlFlows<Output: FromContext> {
    flows: Arc<[ControlFlow<Output>]>,
    scope: Option<ScopeId>,
    pub(crate) nodes: Nodes<Output>,
    selected_flow: Option<usize>,
    node_index: usize,
}

impl<Output: FromContext> ControlFlows<Output> {
    pub fn new(flows: Arc<[ControlFlow<Output>]>, scope: Option<ScopeId>) -> Self {
        Self {
            flows,
            scope,
            nodes: Nodes::empty(),
            selected_flow: None,
            node_index: 0,
        }
    }

    pub(super) fn next(
        &mut self,
        bucket: &BucketRef<'_, Output::Value>,
    ) -> Option<Result<&mut Output, Output::Err>> {
        match self.selected_flow {
            None => {
                let flow_index = self.eval(bucket, self.scope)?;
                self.selected_flow = Some(flow_index);
                for expr in &*self.flows[flow_index].body {
                    match expr.to_node(&EvaluationContext::new(bucket, self.scope)) {
                        Ok(node) => self.nodes.push(node),
                        Err(e) => return Some(Err(e))
                    }
                }
                return self.next(bucket);
            }
            Some(index) => {
                for node in self.nodes.inner[self.node_index..].iter_mut() {
                    match node {
                        Node::Single(output, _) => {
                            self.node_index += 1;
                            return Some(Ok(output));
                        }
                        Node::Collection(nodes) => match nodes.next(bucket) {
                            last @ Some(_) => return last,
                            None => self.node_index += 1,
                        }
                        Node::ControlFlow(flows) => match flows.next(bucket) {
                            last @ Some(_) => return last,
                            None => self.node_index += 1,
                        }
                    }
                }
            }
        }
        None
    }

    fn eval(&mut self, bucket: &BucketRef<'_, Output::Value>, scope: Option<ScopeId>) -> Option<usize> {
        for (index, flow) in self.flows.iter().enumerate() {
            if flow.cond.eval(bucket, scope) {
                return Some(index);
            }
        }
        None
    }
}
