use core::arch::x86_64::{_mm_lfence, _rdtsc};

#[inline(always)]
/// Returns the current CPU tick.
pub fn get_current_tick() -> u64 {
    let value = unsafe { _rdtsc() };
    unsafe { _mm_lfence() };
    value
}
