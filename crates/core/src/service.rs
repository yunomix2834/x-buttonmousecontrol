use crate::{
    AppError, BindingAction, BindingMode, BindingProfile, BindingRepository, InputEvent,
    InputInterceptor, InputPhase, InterceptDecision, OutputEmitter, Target,
};

pub struct BindingRuntime<R, S, E> {
    repo: R,
    source: S,
    emitter: E,
}

impl<R, S, E> BindingRuntime<R, S, E>
where
    R: BindingRepository,
    S: InputInterceptor,
    E: OutputEmitter + Send + 'static,
{
    pub fn new(repo: R, source: S, emitter: E) -> Self {
        Self { repo, source, emitter }
    }

    fn execute_target(
        emitter: &mut E,
        target: &Target,
        action: BindingAction,
        phase: InputPhase,
    ) -> Result<(), AppError> {
        match (target, action, phase) {
            (Target::Key(key), BindingAction::Tap, InputPhase::Press) => emitter.key_tap(key)?,
            (Target::Key(key), BindingAction::Hold, InputPhase::Press) => emitter.key_press(key)?,
            (Target::Key(key), BindingAction::Hold, InputPhase::Release) => emitter.key_release(key)?,

            (Target::Mouse(button), BindingAction::Tap, InputPhase::Press) => {
                emitter.mouse_click(*button)?
            }
            (Target::Mouse(button), BindingAction::Hold, InputPhase::Press) => {
                emitter.mouse_press(*button)?
            }
            (Target::Mouse(button), BindingAction::Hold, InputPhase::Release) => {
                emitter.mouse_release(*button)?
            }

            (_, BindingAction::Tap, InputPhase::Release) => {}
        }

        Ok(())
    }

    fn handle_event(
        profile: &BindingProfile,
        emitter: &mut E,
        event: InputEvent,
    ) -> Result<InterceptDecision, AppError> {
        let mut suppress = false;

        for binding in profile.bindings.iter().filter(|b| b.trigger == event.trigger) {
            Self::execute_target(emitter, &binding.target, binding.action, event.phase)?;
            if binding.mode == BindingMode::Replace {
                suppress = true;
            }
        }

        Ok(if suppress {
            InterceptDecision::Suppress
        } else {
            InterceptDecision::PassThrough
        })
    }

    pub fn run(self) -> Result<(), AppError> {
        let profile = self.repo.load()?;
        let mut emitter = self.emitter;

        self.source.run(move |event| Self::handle_event(&profile, &mut emitter, event))
    }
}