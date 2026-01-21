//! @ai:module:intent Task corpus definitions and loading
//! @ai:module:layer domain
//! @ai:module:public_api Task, TaskCategory, Language, Difficulty, CorpusLoader

pub mod loader;
pub mod task;

pub use loader::{CorpusLoader, CorpusLoaderTrait};
pub use task::{Difficulty, Language, Task, TaskCategory};
