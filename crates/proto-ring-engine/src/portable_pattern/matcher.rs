use num_bigint::BigUint;

use super::model::{CaptureSpan, ClassItem, Node, NodeId, Program};

#[derive(Clone)]
enum Task {
    Evaluate(NodeId),
    FinishCapture {
        index: usize,
        start: usize,
    },
    Repeat {
        child: NodeId,
        minimum: BigUint,
        maximum: Option<BigUint>,
        completed: BigUint,
    },
}

#[derive(Clone)]
struct ExecutionState {
    tasks: Vec<Task>,
    position: usize,
    captures: Vec<Option<CaptureSpan>>,
}

pub(super) fn full_match(program: &Program, input: &[char]) -> Option<Vec<Option<CaptureSpan>>> {
    let mut pending = vec![ExecutionState {
        tasks: vec![Task::Evaluate(program.root)],
        position: 0,
        captures: vec![None; program.capture_count],
    }];
    while let Some(mut state) = pending.pop() {
        'execute: loop {
            let Some(task) = state.tasks.pop() else {
                if state.position == input.len() {
                    return Some(state.captures);
                }
                break 'execute;
            };
            match task {
                Task::Evaluate(node_id) => match &program.nodes[node_id] {
                    Node::Empty => {}
                    Node::Literal(expected) => {
                        if input.get(state.position) != Some(expected) {
                            break 'execute;
                        }
                        state.position += 1;
                    }
                    Node::CharacterClass { negated, items } => {
                        let Some(&actual) = input.get(state.position) else {
                            break 'execute;
                        };
                        if class_contains(items, actual) == *negated {
                            break 'execute;
                        }
                        state.position += 1;
                    }
                    Node::StartAnchor => {
                        if state.position != 0 {
                            break 'execute;
                        }
                    }
                    Node::EndAnchor => {
                        if state.position != input.len() {
                            break 'execute;
                        }
                    }
                    Node::Sequence(children) => {
                        state
                            .tasks
                            .extend(children.iter().rev().copied().map(Task::Evaluate));
                    }
                    Node::Alternation(children) => {
                        schedule_alternatives(&mut state, &mut pending, children);
                    }
                    Node::Group { child, capture } => {
                        if let Some(index) = capture {
                            state.tasks.push(Task::FinishCapture {
                                index: *index,
                                start: state.position,
                            });
                        }
                        state.tasks.push(Task::Evaluate(*child));
                    }
                    Node::Repeat {
                        child,
                        minimum,
                        maximum,
                    } => state.tasks.push(Task::Repeat {
                        child: *child,
                        minimum: minimum.clone(),
                        maximum: maximum.clone(),
                        completed: BigUint::from(0_u8),
                    }),
                },
                Task::FinishCapture { index, start } => {
                    state.captures[index - 1] = Some((start, state.position));
                }
                Task::Repeat {
                    child,
                    minimum,
                    maximum,
                    completed,
                } => {
                    if !minimum_can_fit(program, input, state.position, child, &completed, &minimum)
                    {
                        break 'execute;
                    }
                    let may_stop = completed >= minimum;
                    let may_continue = maximum.as_ref().is_none_or(|maximum| completed < *maximum)
                        && child_can_fit(program, input, state.position, child);
                    if !may_continue {
                        if may_stop {
                            continue;
                        }
                        break 'execute;
                    }
                    if may_stop {
                        pending.push(state.clone());
                    }
                    state.tasks.push(Task::Repeat {
                        child,
                        minimum,
                        maximum,
                        completed: completed + BigUint::from(1_u8),
                    });
                    state.tasks.push(Task::Evaluate(child));
                }
            }
        }
    }
    None
}

fn schedule_alternatives(
    state: &mut ExecutionState,
    pending: &mut Vec<ExecutionState>,
    children: &[NodeId],
) {
    for child in children[1..].iter().rev() {
        let mut alternative = state.clone();
        alternative.tasks.push(Task::Evaluate(*child));
        pending.push(alternative);
    }
    state.tasks.push(Task::Evaluate(children[0]));
}

fn minimum_can_fit(
    program: &Program,
    input: &[char],
    position: usize,
    child: NodeId,
    completed: &BigUint,
    minimum: &BigUint,
) -> bool {
    if completed >= minimum {
        return true;
    }
    let required = minimum - completed;
    let consumption = &program.properties[child].minimum_consumption * required;
    consumption <= BigUint::from(input.len() - position)
}

fn child_can_fit(program: &Program, input: &[char], position: usize, child: NodeId) -> bool {
    program.properties[child].minimum_consumption <= BigUint::from(input.len() - position)
}

fn class_contains(items: &[ClassItem], scalar: char) -> bool {
    items.iter().any(|item| match item {
        ClassItem::Literal(expected) => scalar == *expected,
        ClassItem::Range(start, end) => *start <= scalar && scalar <= *end,
    })
}
