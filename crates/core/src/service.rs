use crate::{
    AppError, BindingAction, BindingProfile, BindingRepository, InputEvent, InputEventSource,
    InputPhase, OutputEmitter, Target,
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
    S: InputEventSource,
    E: OutputEmitter,
{
    pub fn new(repo: R, source: S, emitter: E) -> Self {
        Self { repo, source, emitter }
    }

    fn execute_target(
        &mut self,
        target: &Target,
        action: BindingAction,
        phase: InputPhase,
    ) -> Result<(), AppError> {
        match (target, action, phase) {
            (Target::Key(key), BindingAction::Tap, InputPhase::Press) => {
                self.emitter.key_tap(key)?;
            }
            (Target::Key(key), BindingAction::Hold, InputPhase::Press) => {
                self.emitter.key_press(key)?;
            }
            (Target::Key(key), BindingAction::Hold, InputPhase::Release) => {
                self.emitter.key_release(key)?;
            }

            (Target::Mouse(button), BindingAction::Tap, InputPhase::Press) => {
                self.emitter.mouse_click(*button)?;
            }
            (Target::Mouse(button), BindingAction::Hold, InputPhase::Press) => {
                self.emitter.mouse_press(*button)?;
            }
            (Target::Mouse(button), BindingAction::Hold, InputPhase::Release) => {
                self.emitter.mouse_release(*button)?;
            }

            (_, BindingAction::Tap, InputPhase::Release) => {}
        }

        Ok(())
    }

    fn dispatch_event(
        &mut self,
        profile: &BindingProfile,
        event: InputEvent,
    ) -> Result<(), AppError> {
        for binding in profile.bindings.iter().filter(|b| b.trigger == event.trigger) {
            self.execute_target(&binding.target, binding.action, event.phase)?;
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