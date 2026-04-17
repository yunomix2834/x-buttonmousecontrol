use crate::{
    AppError, BindingProfile, InputEvent, InterceptDecision, KeySpec, MouseButton,
};

pub trait BindingRepository {
    fn load(&self) -> Result<BindingProfile, AppError>;
}

pub trait InputInterceptor {
    fn run<H>(&self, handler: H) -> Result<(), AppError>
    where
        H: FnMut(InputEvent) -> Result<InterceptDecision, AppError> + Send + 'static;
}

pub trait OutputEmitter {
    fn key_press(&mut self, key: &KeySpec) -> Result<(), AppError>;
    fn key_release(&mut self, key: &KeySpec) -> Result<(), AppError>;
    fn key_tap(&mut self, key: &KeySpec) -> Result<(), AppError>;

    fn mouse_press(&mut self, button: MouseButton) -> Result<(), AppError>;
    fn mouse_release(&mut self, button: MouseButton) -> Result<(), AppError>;
    fn mouse_click(&mut self, button: MouseButton) -> Result<(), AppError>;
}