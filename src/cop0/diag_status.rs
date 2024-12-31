pub struct DiagStatus;

impl DiagStatus {
    pub const fn instruction_trace_support(val: bool) -> u32 {
        (val as u32) << 24
    }

    pub const fn bootstrap_exception_vectors(val: bool) -> u32 {
        (val as u32) << 22
    }

    pub const fn tlb_shutdown(val: bool) -> u32 {
        (val as u32) << 21
    }

    pub const fn soft_reset(val: bool) -> u32 {
        (val as u32) << 20
    }

    pub const fn cp0_condition(val: bool) -> u32 {
        (val as u32) << 18
    }
}
