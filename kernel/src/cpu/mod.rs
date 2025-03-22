pub mod sched;


#[macro_export]
macro_rules! read_csr {
    ($reg:expr, $type:tt) => {
        unsafe {
            let value: $type;

            core::arch::asm!(
                concat!("csrr ", "{value}, ", $reg),
                value = out(reg) value,
            );

            value
        }
    };
}

#[macro_export]
macro_rules! write_csr {
    ($reg:expr, $value:expr) => {
        unsafe {
            core::arch::asm!(
                concat!("csrw ", $reg, ", {value}"),
                value = in(reg) $value,
            );
        }
    };
}


