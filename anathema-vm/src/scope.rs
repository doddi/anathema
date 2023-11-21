use anathema_compiler::{Constants, Instruction, StringId};
use anathema_values::{Attributes, ValueExpr};
use anathema_widget_core::generator::{
    ControlFlow, ElseExpr, Expression, IfExpr, LoopExpr, SingleNode,
};

use crate::error::Result;

pub(crate) struct Scope<'vm> {
    instructions: Vec<Instruction>,
    consts: &'vm Constants,
}

impl<'vm> Scope<'vm> {
    pub fn new(instructions: Vec<Instruction>, consts: &'vm Constants) -> Self {
        Self {
            instructions,
            consts,
        }
    }

    pub fn exec(&mut self) -> Result<Vec<Expression>> {
        let mut nodes = vec![];

        if self.instructions.is_empty() {
            return Ok(nodes);
        }

        loop {
            let instruction = self.instructions.remove(0);
            match instruction {
                Instruction::View(ident) => {
                    let ident = self.consts.lookup_value(ident).clone();

                    let ident = match self.instructions.remove(0) {
                        Instruction::LoadValue(ident) => {
                            self.consts.lookup_value(ident).clone()
                        }
                        _ => unreachable!(
                            "a view can not be compiled without being followed by a value"
                        ),
                    };

                    let state = match self.instructions.get(0) {
                        Some(Instruction::LoadValue(i)) => {
                            let state = Some(self.consts.lookup_value(*i).clone());
                            self.instructions.remove(0);
                            state
                        }
                        _ => None,
                    };

                    nodes.push(Expression::View { ident, state });
                }
                Instruction::Node { ident, scope_size } => {
                    nodes.push(self.node(ident, scope_size)?)
                }
                Instruction::For {
                    binding,
                    data,
                    size,
                } => {
                    let binding = self.consts.lookup_string(binding);

                    let collection = self.consts.lookup_value(data).clone();

                    let body = self.instructions.drain(..size).collect();
                    let body = Scope::new(body, &self.consts).exec()?;
                    let template = Expression::Loop(LoopExpr {
                        binding: binding.into(),
                        collection,
                        body,
                    });

                    nodes.push(template);
                }
                Instruction::If { cond, size } => {
                    let cond = self.consts.lookup_value(cond);

                    let body = self.instructions.drain(..size).collect::<Vec<_>>();
                    let body = Scope::new(body, &self.consts).exec()?;

                    let mut control_flow = ControlFlow {
                        if_expr: IfExpr {
                            cond,
                            expressions: body,
                        },
                        elses: vec![],
                    };

                    loop {
                        let Some(&Instruction::Else { cond, size }) = self.instructions.get(0)
                        else {
                            break;
                        };
                        self.instructions.remove(0);
                        let cond = cond.map(|cond| self.consts.lookup_value(cond));

                        let body = self.instructions.drain(..size).collect();
                        let body = Scope::new(body, &self.consts).exec()?;

                        control_flow.elses.push(ElseExpr {
                            cond,
                            expressions: body,
                        });
                    }

                    let template = Expression::ControlFlow(control_flow.into());
                    nodes.push(template);
                }
                Instruction::Else { .. } => {
                    unreachable!("the `Else` instructions are consumed inside the `If` instruction")
                }
                Instruction::LoadAttribute { .. } | Instruction::LoadValue(_) => {
                    unreachable!("these instructions are only executed in the `node` function")
                }
            }

            if self.instructions.is_empty() {
                break;
            }
        }

        Ok(nodes)
    }

    fn node(&mut self, ident: StringId, scope_size: usize) -> Result<Expression> {
        let ident = self.consts.lookup_string(ident);

        let mut attributes = Attributes::new();
        let mut text = None::<ValueExpr>;
        let mut ip = 0;

        loop {
            match self.instructions.get(ip) {
                Some(Instruction::LoadAttribute { key, value }) => {
                    let key = self.consts.lookup_string(*key);
                    let value = self.consts.lookup_value(*value);
                    attributes.insert(key.to_string(), value.clone());
                }
                Some(Instruction::LoadValue(i)) => {
                    text = Some(self.consts.lookup_value(*i).clone())
                }
                _ => break,
            }
            ip += 1;
        }

        // Remove processed attribute and text instructions
        self.instructions.drain(..ip);

        let scope = self.instructions.drain(..scope_size).collect();
        let children = Scope::new(scope, &self.consts).exec()?;

        let node = Expression::Node(SingleNode {
            ident: ident.to_string(),
            text,
            attributes,
            children,
        });

        Ok(node)
    }
}
