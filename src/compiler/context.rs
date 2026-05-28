use std::any::Any;

use super::TimeZone;

use super::{Target, state::RuntimeState};

pub struct Context<'a> {
    target: &'a mut dyn Target,
    state: &'a mut RuntimeState,
    timezone: &'a TimeZone,
    dynamic_state: Option<&'a mut dyn Any>,
}

impl<'a> Context<'a> {
    /// Create a new [`Context`].
    pub fn new(
        target: &'a mut dyn Target,
        state: &'a mut RuntimeState,
        timezone: &'a TimeZone,
    ) -> Self {
        Self {
            target,
            state,
            timezone,
            dynamic_state: None,
        }
    }

    #[must_use]
    pub fn with_dynamic_state(mut self, state: &'a mut dyn Any) -> Self {
        self.dynamic_state = Some(state);
        self
    }

    pub fn dynamic_state(&mut self) -> Option<&mut dyn Any> {
        self.dynamic_state.as_deref_mut()
    }

    /// Get a reference to the [`Target`].
    #[must_use]
    pub fn target(&self) -> &dyn Target {
        self.target
    }

    /// Get a mutable reference to the [`Target`].
    pub fn target_mut(&mut self) -> &mut dyn Target {
        self.target
    }

    /// Get a reference to the [`runtime state`](RuntimeState).
    #[must_use]
    pub fn state(&self) -> &RuntimeState {
        self.state
    }

    /// Get a mutable reference to the [`runtime state`](RuntimeState).
    pub fn state_mut(&mut self) -> &mut RuntimeState {
        self.state
    }

    /// Get a reference to the [`TimeZone`]
    #[must_use]
    pub fn timezone(&self) -> &TimeZone {
        self.timezone
    }
}
