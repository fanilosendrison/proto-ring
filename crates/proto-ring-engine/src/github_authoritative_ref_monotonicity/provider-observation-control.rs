//! Calling-thread-only conformance evidence, not universal network interception.

use std::cell::Cell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlFailure {
    NestedScope,
    CounterOverflow,
    MissingState,
    StateUnavailable,
    InvocationsObserved,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ControlError {
    kind: ControlFailure,
    invocations: Option<u64>,
}

impl ControlError {
    pub fn kind(&self) -> ControlFailure {
        self.kind
    }

    pub fn invocations(&self) -> Option<u64> {
        self.invocations
    }
}

impl Display for ControlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "ARM observation control {:?}: {:?} invocations",
            self.kind, self.invocations
        )
    }
}

impl Error for ControlError {}

/// Constructed only after a valid scope completes. No public constructor.
#[derive(Debug)]
pub struct ObservationReceipt {
    invocations: u64,
}

impl ObservationReceipt {
    pub fn invocations(&self) -> u64 {
        self.invocations
    }
}

#[derive(Clone, Copy)]
struct Counter {
    invocations: u64,
    failure: Option<ControlFailure>,
}

#[derive(Clone, Copy)]
struct State {
    armed: bool,
    counter: Option<Counter>,
}

const IDLE: State = State {
    armed: false,
    counter: None,
};

thread_local! {
    static STATE: Cell<State> = const { Cell::new(IDLE) };
}

struct ScopeOwner(PhantomData<Rc<()>>);

impl Drop for ScopeOwner {
    fn drop(&mut self) {
        // TLS destruction cannot yield a receipt. Unwinding still clears a live slot.
        let _ = STATE.try_with(|slot| slot.set(IDLE));
    }
}

fn error(kind: ControlFailure, invocations: Option<u64>) -> ControlError {
    ControlError { kind, invocations }
}

fn begin() -> Result<ScopeOwner, ControlError> {
    STATE
        .try_with(|slot| {
            let mut state = slot.get();
            if state.armed || state.counter.is_some() {
                let count = state.counter.map(|counter| counter.invocations);
                if let Some(counter) = state.counter.as_mut() {
                    counter.failure.get_or_insert(ControlFailure::NestedScope);
                }
                state.armed = true;
                slot.set(state);
                return Err(error(ControlFailure::NestedScope, count));
            }
            slot.set(State {
                armed: true,
                counter: Some(Counter {
                    invocations: 0,
                    failure: None,
                }),
            });
            Ok(ScopeOwner(PhantomData))
        })
        .map_err(|_| error(ControlFailure::StateUnavailable, None))?
}

fn complete(state: State) -> Result<ObservationReceipt, ControlError> {
    let counter = state
        .counter
        .filter(|_| state.armed)
        .ok_or_else(|| error(ControlFailure::MissingState, None))?;
    if let Some(failure) = counter.failure {
        return Err(error(failure, Some(counter.invocations)));
    }
    if counter.invocations != 0 {
        return Err(error(
            ControlFailure::InvocationsObserved,
            Some(counter.invocations),
        ));
    }
    Ok(ObservationReceipt {
        invocations: counter.invocations,
    })
}

/// Runs synchronously on this thread; nested scopes invalidate the outer evidence.
/// The TLS cell is never borrowed across the closure. The owner cannot cross threads.
pub fn with_provider_observation_forbidden<T>(
    operation: impl FnOnce() -> T,
) -> Result<(T, ObservationReceipt), ControlError> {
    let owner = begin()?;
    let result = operation();
    let state = STATE
        .try_with(|slot| slot.replace(IDLE))
        .map_err(|_| error(ControlFailure::StateUnavailable, None))?;
    let receipt = complete(state)?;
    drop(owner);
    Ok((result, receipt))
}

/// True prohibits entry, including when active evidence is damaged or TLS unavailable.
pub(super) fn entry_forbidden() -> bool {
    STATE
        .try_with(|slot| {
            let mut state = slot.get();
            if !state.armed && state.counter.is_none() {
                return false;
            }
            state.armed = true;
            if let Some(counter) = state.counter.as_mut() {
                match counter.invocations.checked_add(1) {
                    Some(count) => counter.invocations = count,
                    None => counter.failure = Some(ControlFailure::CounterOverflow),
                }
            }
            slot.set(state);
            true
        })
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_scope_invalidates_outer_without_running_nested_closure() {
        let mut nested_ran = false;
        let outer = with_provider_observation_forbidden(|| {
            assert!(entry_forbidden());
            let nested = with_provider_observation_forbidden(|| nested_ran = true);
            assert_eq!(nested.unwrap_err().kind(), ControlFailure::NestedScope);
        })
        .unwrap_err();
        assert!(!nested_ran);
        assert_eq!(outer.kind(), ControlFailure::NestedScope);
        assert_eq!(outer.invocations(), Some(1));
        assert!(with_provider_observation_forbidden(|| ()).is_ok());
    }

    #[test]
    fn overflow_invalidates_evidence_without_wrapping() {
        let outcome = with_provider_observation_forbidden(|| {
            STATE.with(|slot| {
                let mut state = slot.get();
                state.counter.as_mut().unwrap().invocations = u64::MAX;
                slot.set(state);
            });
            assert!(entry_forbidden());
        })
        .unwrap_err();
        assert_eq!(outcome.kind(), ControlFailure::CounterOverflow);
        assert_eq!(outcome.invocations(), Some(u64::MAX));
    }

    #[test]
    fn missing_counter_blocks_entry_and_cannot_complete_zero() {
        let outcome = with_provider_observation_forbidden(|| {
            STATE.with(|slot| {
                slot.set(State {
                    armed: true,
                    counter: None,
                })
            });
            assert!(entry_forbidden());
        })
        .unwrap_err();
        assert_eq!(outcome.kind(), ControlFailure::MissingState);
        assert_eq!(outcome.invocations(), None);
        assert!(complete(IDLE).is_err());
        assert!(with_provider_observation_forbidden(|| ()).is_ok());
    }

    #[test]
    fn borrowed_tls_storage_is_not_held_across_operation() {
        let (_, receipt) = with_provider_observation_forbidden(|| {
            STATE.with(|slot| assert!(slot.get().armed));
        })
        .unwrap();
        assert_eq!(receipt.invocations(), 0);
    }
}
