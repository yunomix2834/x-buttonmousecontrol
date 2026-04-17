pub mod emitter;
pub mod source;

pub(crate) const SYNTHETIC_TAG: usize = 0x5842_4D43;

pub use emitter::WinHookEmitter;
pub use source::WinHookInputInterceptor;
