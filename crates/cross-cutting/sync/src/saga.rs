//! # Saga / compensating actions
//!
//! A saga is a sequence of typed steps, each with a forward
//! action (the work to do) and a compensating action (the work
//! to undo). If a step fails, the saga walks back through the
//! completed steps in reverse order and invokes each
//! compensation, restoring the system to a consistent state.
//!
//! ## Why
//!
//! Multi-step workflows in the sync engine often span several
//! resources: fetching a remote change, applying it locally,
//! committing to the audit log, and notifying the bus. If any
//! step fails partway through, the engine must undo the prior
//! steps to preserve the at-least-once semantics the engine
//! promises. The saga pattern is the standard tool for that.
//!
//! ## Shape
//!
//! - [`SagaStep<I, O>`] is a typed step with a `forward` action
//!   that takes an input of type `I` and produces an output of
//!   type `O`, plus a `compensate` action that takes the output
//!   and undoes the work. Each step has a unique name used in
//!   error messages and in the [`SagaResult`] report.
//! - [`Saga<S>`] is the state machine: it owns an initial state
//!   of type `S`, holds the sequence of steps, and tracks the
//!   compensations of the completed steps so they can be invoked
//!   in reverse order on failure.
//! - [`SagaResult`] is the outcome: [`SagaResult::Completed`],
//!   [`SagaResult::Compensated`], or [`SagaResult::Failed`].
//! - [`SagaError`] is the typed error variant returned by step
//!   actions.
//!
//! ## Compensating actions
//!
//! Compensation is invoked in reverse completion order. It is
//! idempotent: calling [`Saga::compensate`] twice is a no-op the
//! second time, even if individual steps have side effects.
//! Each stored compensation tracks its own `consumed` flag so
//! double-invocation cannot happen at the step level either.
//!
//! ## In practice
//!
//! The engine currently uses [`Saga`] from single-process sync
//! flows (where the in-process reference adapter handles
//! failures inline) and from future HTTP/IPC adapters that need
//! to undo a remote commit when a local side effect fails. The
//! wire protocol itself (`docs/ports/sync.md`) is transport-
//! agnostic and does not prescribe a saga layout; the saga is
//! a composition tool, not a transport detail.

use std::fmt;
use std::marker::PhantomData;

/// A boxed closure that performs a saga step's forward action.
type ForwardFn<I, O> = Box<dyn Fn(I) -> Result<O, SagaError> + Send + Sync>;

/// A boxed closure that performs a saga step's compensation.
type CompensateFn<O> = Box<dyn Fn(O) -> Result<(), SagaError> + Send + Sync>;

/// A typed error returned by saga steps and compensations.
///
/// Carries the name of the step that produced the error (when
/// known) and a human-readable message. Distinct from
/// `educore_core::DomainError` so the saga library stays
/// independent of the engine's domain error machinery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SagaError {
    step: Option<&'static str>,
    message: String,
}

impl SagaError {
    /// Creates a new error with no associated step.
    ///
    /// The caller can attach a step name later with
    /// [`SagaError::at_step`] when the error crosses a step
    /// boundary.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            step: None,
            message: message.into(),
        }
    }

    /// Attaches a step name to this error.
    ///
    /// Returns `self` so the call can be chained at the call
    /// site of a failing step.
    #[must_use]
    pub fn at_step(mut self, step: &'static str) -> Self {
        self.step = Some(step);
        self
    }

    /// Returns the name of the step that produced this error,
    /// if known.
    #[must_use]
    pub fn step(&self) -> Option<&'static str> {
        self.step
    }

    /// Returns the human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SagaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.step {
            Some(name) => write!(f, "saga step '{}' failed: {}", name, self.message),
            None => write!(f, "saga error: {}", self.message),
        }
    }
}

impl std::error::Error for SagaError {}

/// The outcome of running a saga.
///
/// Returned by [`Saga::run`] and inspected by the caller to
/// decide whether to commit, retry, or surface a failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaResult {
    /// Every step completed successfully.
    Completed,
    /// A step failed and every previously completed step had
    /// its compensating action invoked. The system is back to
    /// a consistent state.
    Compensated {
        /// The name of the step that triggered compensation.
        failed_at: String,
        /// The names of the steps that were compensated, in the
        /// order they were invoked (reverse of completion order).
        compensated: Vec<String>,
    },
    /// A step failed and at least one compensation failed.
    ///
    /// The system is in an inconsistent state; the caller should
    /// log the failure, alert operators, and decide whether to
    /// retry, surface, or quarantine the workflow.
    Failed {
        /// The name of the step that triggered compensation.
        failed_at: String,
        /// Human-readable message describing the failure.
        error: String,
        /// The names of the steps that were successfully
        /// compensated, in compensation order.
        compensated: Vec<String>,
    },
}

impl SagaResult {
    /// Returns `true` if the saga completed cleanly.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self, SagaResult::Completed)
    }

    /// Returns `true` if the saga failed but was fully
    /// compensated.
    #[must_use]
    pub fn is_compensated(&self) -> bool {
        matches!(self, SagaResult::Compensated { .. })
    }

    /// Returns `true` if the saga failed and at least one
    /// compensation failed.
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self, SagaResult::Failed { .. })
    }
}

/// A typed step in a saga.
///
/// `SagaStep<I, O>` declares a forward action that takes an
/// input of type `I` and produces an output of type `O`, plus a
/// compensating action that takes the output and undoes the
/// work. Each step has a unique name used in error messages
/// and in the [`SagaResult`] report.
///
/// The input type `I` is generic for symmetry with `O`; in
/// practice the saga library passes `()` as the input, so
/// steps that do not need an argument should declare `I = ()`.
/// The forward closure's argument type must match this generic
/// parameter; the compiler enforces it at the closure
/// signature.
///
/// The output type `O` is what the step produces on success and
/// what the compensation receives on failure.
pub struct SagaStep<I, O> {
    name: &'static str,
    forward: Option<ForwardFn<I, O>>,
    compensate: Option<CompensateFn<O>>,
    _phantom: PhantomData<fn(I) -> O>,
}

impl<I, O> SagaStep<I, O> {
    /// Creates a new typed step from forward and compensate
    /// closures.
    ///
    /// Both closures must be `Send + Sync + 'static` so the step
    /// can be stored in the saga's type-erased step vector and
    /// shared across threads if the saga is driven from a
    /// multi-threaded runtime.
    #[must_use]
    pub fn new(
        name: &'static str,
        forward: impl Fn(I) -> Result<O, SagaError> + Send + Sync + 'static,
        compensate: impl Fn(O) -> Result<(), SagaError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name,
            forward: Some(Box::new(forward)),
            compensate: Some(Box::new(compensate)),
            _phantom: PhantomData,
        }
    }

    /// Returns the step's name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// Type-erased execution step used inside [`Saga`].
///
/// Each step in the saga's storage is an `ErasedExecutionStep`
/// so the saga can hold steps of different `I` and `O` types in
/// a single `Vec`. The step's `execute` method runs the forward
/// action and produces a type-erased compensation holding the
/// captured output.
trait ErasedExecutionStep: Send + Sync {
    /// Returns the step's name.
    fn name(&self) -> &'static str;

    /// Runs the step's forward action and returns a
    /// type-erased compensation that can undo the work later.
    fn execute(&mut self) -> Result<Box<dyn ErasedCompensation>, SagaError>;
}

/// Type-erased compensation stored after a step succeeds.
///
/// Each `ErasedCompensation` carries the output produced by the
/// step's forward action plus the compensating closure. The
/// compensation is idempotent: calling [`compensate`] a second
/// time is a no-op.
///
/// [`compensate`]: ErasedCompensation::compensate
trait ErasedCompensation: Send + Sync {
    /// Returns the step's name.
    fn name(&self) -> &str;

    /// Invokes the compensating action with the captured
    /// output. Returns `Ok(())` if already consumed.
    fn compensate(&mut self) -> Result<(), SagaError>;

    /// Returns `true` if the compensation has already been
    /// invoked.
    fn is_compensated(&self) -> bool;
}

/// A stored compensation for a single completed step.
///
/// Holds the output produced by the forward action and the
/// compensate closure. The `invoked` flag ensures idempotent
/// compensation (the closure runs at most once) even if the
/// [`Saga`]'s outer idempotency guard is bypassed. The
/// `succeeded` flag tracks whether the compensation succeeded
/// and is what [`ErasedCompensation::is_compensated`] reports.
struct StoredCompensation<O> {
    name: &'static str,
    output: Option<O>,
    compensate: CompensateFn<O>,
    invoked: bool,
    succeeded: bool,
}

impl<O: Send + Sync + 'static> ErasedCompensation for StoredCompensation<O> {
    fn name(&self) -> &str {
        self.name
    }

    fn compensate(&mut self) -> Result<(), SagaError> {
        if self.invoked {
            return if self.succeeded {
                Ok(())
            } else {
                Err(SagaError::new("compensation previously failed"))
            };
        }
        let output = self
            .output
            .take()
            .ok_or_else(|| SagaError::new("compensation output already consumed"))?;
        self.invoked = true;
        let result = (self.compensate)(output);
        self.succeeded = result.is_ok();
        result
    }

    fn is_compensated(&self) -> bool {
        self.succeeded
    }
}

impl<O> ErasedExecutionStep for SagaStep<(), O>
where
    O: Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn execute(&mut self) -> Result<Box<dyn ErasedCompensation>, SagaError> {
        let forward_fn = self
            .forward
            .take()
            .ok_or_else(|| SagaError::new("forward already executed"))?;
        let compensate_fn = self
            .compensate
            .take()
            .ok_or_else(|| SagaError::new("compensate already taken"))?;
        // The saga library passes `()` as the input to each
        // step's forward action. Steps that need an argument
        // can capture it from the closure environment.
        let output = forward_fn(()).map_err(|e| match e.step() {
            Some(_) => e,
            None => e.at_step(self.name),
        })?;
        Ok(Box::new(StoredCompensation {
            name: self.name,
            output: Some(output),
            compensate: compensate_fn,
            invoked: false,
            succeeded: false,
        }))
    }
}

/// A saga state machine.
///
/// Owns an initial state of type `S`, holds a sequence of
/// [`SagaStep`]s, and tracks the compensations of the completed
/// steps so they can be invoked in reverse order on failure.
///
/// The state `S` is stored alongside the saga so callers can
/// keep workflow-scoped data (a session id, a remote change id,
/// etc.) without juggling it as a separate variable. Steps do
/// not currently receive the state as input; steps that need
/// workflow-scoped data should capture it from their closure
/// environment.
pub struct Saga<S> {
    name: String,
    state: S,
    steps: Vec<Box<dyn ErasedExecutionStep>>,
    completed: Vec<Box<dyn ErasedCompensation>>,
    has_compensated: bool,
    last_result: Option<SagaResult>,
}

impl<S> Saga<S> {
    /// Creates a new saga with the given name and initial state.
    #[must_use]
    pub fn new(name: impl Into<String>, state: S) -> Self {
        Self {
            name: name.into(),
            state,
            steps: Vec::new(),
            completed: Vec::new(),
            has_compensated: false,
            last_result: None,
        }
    }

    /// Adds a step to the saga.
    ///
    /// Steps are executed in the order they are added. The
    /// `SagaStep<(), O>` is moved into the saga; after being
    /// added it can only be invoked through the saga's
    /// [`run`](Self::run) method. The saga library passes `()`
    /// as the input to each step's forward action; steps that
    /// need workflow-scoped data should capture it from their
    /// closure environment.
    pub fn add<O>(&mut self, step: SagaStep<(), O>)
    where
        O: Send + Sync + 'static,
    {
        self.steps.push(Box::new(step));
    }

    /// Returns the saga's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the saga's initial state.
    #[must_use]
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Returns the number of steps registered with the saga.
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Runs every step in order.
    ///
    /// On the first failure, invokes compensation in reverse
    /// order on the previously completed steps and returns
    /// either [`SagaResult::Compensated`] (if every compensation
    /// succeeded) or [`SagaResult::Failed`] (if at least one
    /// compensation failed).
    ///
    /// If the saga has already been compensated (via
    /// [`compensate`](Self::compensate) or a previous failure),
    /// returns the previous result without re-running.
    #[must_use]
    pub fn run(&mut self) -> SagaResult {
        if let Some(prev) = &self.last_result {
            return prev.clone();
        }

        let step_count = self.steps.len();
        for i in 0..step_count {
            let step_name = self.steps[i].name().to_string();
            match self.steps[i].execute() {
                Ok(compensation) => {
                    self.completed.push(compensation);
                }
                Err(err) => {
                    let failed_at = step_name;
                    let error_msg = err.message().to_string();
                    match self.compensate() {
                        Ok(()) => {
                            let result = SagaResult::Compensated {
                                failed_at,
                                compensated: self.compensated_step_names(),
                            };
                            self.last_result = Some(result.clone());
                            return result;
                        }
                        Err(_) => {
                            let result = SagaResult::Failed {
                                failed_at,
                                error: error_msg,
                                compensated: self.compensated_step_names(),
                            };
                            self.last_result = Some(result.clone());
                            return result;
                        }
                    }
                }
            }
        }

        let result = SagaResult::Completed;
        self.last_result = Some(result.clone());
        result
    }

    /// Walks back through the completed steps and invokes each
    /// compensating action in reverse order.
    ///
    /// Idempotent: a second invocation returns `Ok(())` without
    /// re-invoking any compensation. Continues past
    /// compensation failures so a single failed compensation
    /// does not leave the rest of the saga unrolled; the last
    /// encountered error (if any) is returned.
    pub fn compensate(&mut self) -> Result<(), SagaError> {
        if self.has_compensated {
            return Ok(());
        }
        let mut last_err: Option<SagaError> = None;
        for compensation in self.completed.iter_mut().rev() {
            if let Err(e) = compensation.compensate() {
                last_err = Some(e);
            }
        }
        self.has_compensated = true;
        match last_err {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }

    /// Collects the names of the steps that have been
    /// compensated, in compensation order (reverse of
    /// completion order).
    fn compensated_step_names(&self) -> Vec<String> {
        self.completed
            .iter()
            .rev()
            .filter(|c| c.is_compensated())
            .map(|c| c.name().to_string())
            .collect()
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[test]
    fn happy_path_runs_all_steps() {
        let mut saga: Saga<()> = Saga::new("happy", ());
        saga.add(SagaStep::<(), u32>::new(
            "step1",
            |()| Ok::<_, SagaError>(1u32),
            |_| Ok::<_, SagaError>(()),
        ));
        saga.add(SagaStep::<(), u32>::new(
            "step2",
            |()| Ok::<_, SagaError>(2u32),
            |_| Ok::<_, SagaError>(()),
        ));
        saga.add(SagaStep::<(), u32>::new(
            "step3",
            |()| Ok::<_, SagaError>(3u32),
            |_| Ok::<_, SagaError>(()),
        ));

        let result = saga.run();
        assert_eq!(result, SagaResult::Completed);
        assert!(result.is_completed());
        assert_eq!(saga.step_count(), 3);
    }

    #[test]
    fn single_step_failure_triggers_compensation() {
        let mut saga: Saga<()> = Saga::new("single_fail", ());
        saga.add(SagaStep::<(), u32>::new(
            "ok1",
            |()| Ok::<_, SagaError>(1u32),
            |_| Ok::<_, SagaError>(()),
        ));
        saga.add(SagaStep::<(), u32>::new(
            "boom",
            |()| Err(SagaError::new("kaboom")),
            |_| Ok::<_, SagaError>(()),
        ));
        saga.add(SagaStep::<(), u32>::new(
            "ok2",
            |()| Ok::<_, SagaError>(2u32),
            |_| Ok::<_, SagaError>(()),
        ));

        let result = saga.run();
        match result {
            SagaResult::Compensated {
                failed_at,
                compensated,
            } => {
                assert_eq!(failed_at, "boom");
                assert_eq!(compensated, vec!["ok1".to_string()]);
            }
            other => panic!("expected Compensated, got {:?}", other),
        }
    }

    #[test]
    fn multi_step_failure_compensates_in_reverse() {
        let order: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));

        let mut saga: Saga<()> = Saga::new("multi_fail", ());
        saga.add(SagaStep::<(), u32>::new(
            "s1",
            |()| Ok::<_, SagaError>(1u32),
            {
                let order = order.clone();
                move |_| {
                    order.lock().unwrap().push("c1");
                    Ok::<_, SagaError>(())
                }
            },
        ));
        saga.add(SagaStep::<(), u32>::new(
            "s2",
            |()| Ok::<_, SagaError>(2u32),
            {
                let order = order.clone();
                move |_| {
                    order.lock().unwrap().push("c2");
                    Ok::<_, SagaError>(())
                }
            },
        ));
        saga.add(SagaStep::<(), u32>::new(
            "s3",
            |()| Err(SagaError::new("boom")),
            |_| Ok::<_, SagaError>(()),
        ));

        let result = saga.run();
        match result {
            SagaResult::Compensated {
                failed_at,
                compensated,
            } => {
                assert_eq!(failed_at, "s3");
                assert_eq!(compensated, vec!["s2".to_string(), "s1".to_string()]);
                assert_eq!(
                    *order.lock().unwrap(),
                    vec!["c2".to_string(), "c1".to_string()]
                );
            }
            other => panic!("expected Compensated, got {:?}", other),
        }
    }

    #[test]
    fn compensate_is_idempotent() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut saga: Saga<()> = Saga::new("idempotent", ());
        saga.add(SagaStep::<(), u32>::new(
            "step",
            |()| Ok::<_, SagaError>(1u32),
            {
                let counter = counter.clone();
                move |_| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, SagaError>(())
                }
            },
        ));

        // Run, then compensate. The counter should reach 1.
        let _ = saga.run();
        assert!(saga.compensate().is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Two more calls must be no-ops at both the saga and
        // step level.
        assert!(saga.compensate().is_ok());
        assert!(saga.compensate().is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn compensation_failure_returns_failed_result() {
        let mut saga: Saga<()> = Saga::new("comp_fail", ());
        saga.add(SagaStep::<(), u32>::new(
            "ok_step",
            |()| Ok::<_, SagaError>(1u32),
            |_| Err(SagaError::new("compensation error")),
        ));
        saga.add(SagaStep::<(), u32>::new(
            "boom",
            |()| Err(SagaError::new("forward error")),
            |_| Ok::<_, SagaError>(()),
        ));

        let result = saga.run();
        match result {
            SagaResult::Failed {
                failed_at,
                error,
                compensated,
            } => {
                assert_eq!(failed_at, "boom");
                assert!(error.contains("forward error"));
                assert!(
                    compensated.is_empty(),
                    "compensation of ok_step should not be marked successful"
                );
            }
            other => panic!("expected Failed, got {:?}", other),
        }
    }
}
