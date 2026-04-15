use crate::{
    AppError, BindingAction, BindingProfile, InputPhase, KeyEmitter, MouseEventSource,
    MouseInputEvent, BindingRepository,
};
use std::sync::mpsc;

pub struct BindingRuntime<R, S, E> {
    repo: R,
    source: S,
    emitter: E,
}

impl<R, S, E> BindingRuntime<R, S, E>
where
    R: BindingRepository,
    S: MouseEventSource,
    E: KeyEmitter,
{
    pub fn new (repo: R, source: S, emitter: E) -> Self {
        Self { repo, source, emitter }
    }

    fn dispatch_event(
        &mut self,
        profile: &BindingProfile,
        event: MouseInputEvent,
    ) -> Result<(), AppError> {
        for binding in profile
            .bindings
            .iter()
            .filter(|b| b.mouse_button == event.button)
        {
            match (binding.action, event.phase) {
                (BindingAction::Tap, InputPhase::Press) => {
                    self.emitter.tap(&binding.key)?;
                }
                (BindingAction::Hold, InputPhase::Press) => {
                    self.emitter.press(&binding.key)?;
                }
                (BindingAction::Hold, InputPhase::Release) => {
                    self.emitter.release(&binding.key)?;
                }
                (BindingAction::Tap, InputPhase::Release) => {
                    // Bỏ qua
                }
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        let profile = self.repo.load()?;

        let (tx, rx) = mpsc::channel();
        self.source.spawn(tx)?;

        while let Ok(event) = rx.recv() {
            self.dispatch_event(&profile, event)?;
        }

        Ok(())
    }
}