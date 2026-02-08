use internal_utils::logln;
use lazy_static::lazy_static;
use x86_64::registers::segmentation::{DS, ES, SS, SegmentSelector};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

/// the interrupt stack table index of the stack used for double faults
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const NMI_IST_INDEX: u16 = 1;

lazy_static! {
    /// The TSS of the OS.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        const STACK_SIZE: usize = 4096;
        #[repr(C, align(16))]
        struct Stack([u8; STACK_SIZE]);

        // Stack used when an exception happens in user mode
        tss.privilege_stack_table[0] = {

            static mut STACK: Stack = Stack([0; STACK_SIZE]);

            let stack_start = VirtAddr::from_ptr(&raw const STACK);

            // returns the highest address of the stack because the stack grows downwards
            stack_start + STACK_SIZE as u64
        };

        // set the interrupt stack table to the appropriate address
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {

            static mut STACK: Stack = Stack([0; STACK_SIZE]);

            let stack_start = VirtAddr::from_ptr(&raw const STACK);

            // returns the highest address of the stack because the stack grows downwards
            stack_start + STACK_SIZE as u64
        };

        tss.interrupt_stack_table[NMI_IST_INDEX as usize] = {

            static mut STACK: Stack = Stack([0; STACK_SIZE]);

            let stack_start = VirtAddr::from_ptr(&raw const STACK);

            // returns the highest address of the stack because the stack grows downwards
            stack_start + STACK_SIZE as u64
        };

        tss
    };

    /// The GDT used by the OS.
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());

        let user_data_selector = gdt.append(Descriptor::user_data_segment());
        let user_code_selector = gdt.append(Descriptor::user_code_segment());

        let mut tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        tss_selector.set_rpl(PrivilegeLevel::Ring0);

        (
            gdt,
            Selectors {
                kernel_code_selector,
                kernel_data_selector,
                user_code_selector,
                user_data_selector,
                tss_selector
            },
        )
    };
}

pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Initialises the GDT and TSS.
pub fn reload_gdt() {
    use x86_64::instructions::segmentation::{CS, Segment};
    use x86_64::instructions::tables::load_tss;
    logln!("[   ---{:^15}---   ]", "INTERRUPTS");
    logln!("Loading GDT and segment registers");
    GDT.0.load();
    logln!("GDT loaded");
    let selector = &GDT.1;
    unsafe {
        CS::set_reg(selector.kernel_code_selector);
        load_tss(selector.tss_selector);
        SS::set_reg(selector.kernel_data_selector);
        DS::set_reg(selector.kernel_data_selector);
        ES::set_reg(selector.kernel_data_selector);
    }
    logln!("Segment registers loaded");
}
