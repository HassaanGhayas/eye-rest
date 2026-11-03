use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};

pub struct MemoryOptimizer;

impl MemoryOptimizer {
    /// Trims application working set to minimize RAM usage when idling
    pub fn trim_working_set() {
        unsafe {
            let process = GetCurrentProcess();
            // Passing (usize::MAX, usize::MAX) instructs Windows to trim all working pages
            let _ = SetProcessWorkingSetSize(process, usize::MAX, usize::MAX);
        }
    }
}
