pub mod dispatcher;

mod memory_mapper;

pub mod process;

pub mod thread;

mod registers_state;
pub use registers_state::RegistersState;

mod scheduler;
pub use scheduler::{add_process, get_scheduler, run_next_thread, run_processes};
