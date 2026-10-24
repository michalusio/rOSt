use internal_utils::ilogln;
use lazy_static::lazy_static;
use x86_64::{VirtAddr, structures::idt::InterruptDescriptorTable};

use crate::interrupts::{
    cpu_handlers::{
        alignment_check_handler, breakpoint_handler, debug_handler, device_not_available_handler,
        divide_error_handler, double_fault_handler, general_protection_fault_handler,
        invalid_opcode_handler, invalid_tss_handler, nmi_handler, page_fault_handler,
        segment_not_present_handler, stack_segment_fault_handler, syscall_interrupt_handler,
    },
    pic::InterruptIndex,
    pic_handlers::{
        ata_primary_interrupt_handler, ata_secondary_interrupt_handler, keyboard_interrupt_handler,
        timer_interrupt_handler,
    },
};

lazy_static! {
    /// The IDT used by the OS.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // ##################
        // # CPU interrupts #
        // ##################
        unsafe {
            idt.divide_error.set_handler_fn(divide_error_handler);

            idt.debug
                .set_handler_fn(debug_handler)
                .set_stack_index(crate::interrupts::gdt::DEBUG_IST_INDEX);

            idt.non_maskable_interrupt
                .set_handler_fn(nmi_handler)
                .set_stack_index(crate::interrupts::gdt::NMI_IST_INDEX);

            idt.breakpoint
                .set_handler_fn(breakpoint_handler)
                .set_stack_index(crate::interrupts::gdt::DEBUG_IST_INDEX);

            idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);

            idt.device_not_available.set_handler_fn(device_not_available_handler);

            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(crate::interrupts::gdt::DOUBLE_FAULT_IST_INDEX);

            idt.invalid_tss.set_handler_fn(invalid_tss_handler);

            idt.segment_not_present.set_handler_fn(segment_not_present_handler);

            idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);

            idt.general_protection_fault
                .set_handler_fn(general_protection_fault_handler)
                .set_stack_index(crate::interrupts::gdt::GPF_IST_INDEX);

            idt.page_fault.set_handler_fn(page_fault_handler);

            idt.alignment_check.set_handler_fn(alignment_check_handler);
        }

        // ##################
        // # PIC interrupts #
        // ##################
        unsafe {
            idt[InterruptIndex::Timer.as_u8()]
                .set_handler_addr(VirtAddr::from_ptr(timer_interrupt_handler as *const u8));
        }

        idt[InterruptIndex::Keyboard.as_u8()]
            .set_handler_fn(keyboard_interrupt_handler);

        idt[InterruptIndex::AtaPrimary.as_u8()]
            .set_handler_fn(ata_primary_interrupt_handler);

        idt[InterruptIndex::AtaSecondary.as_u8()]
            .set_handler_fn(ata_secondary_interrupt_handler);

        // ###########
        // # Syscall #
        // ###########
        unsafe {
            idt[0x80]
                .set_handler_addr(VirtAddr::from_ptr(syscall_interrupt_handler as *const u8));
        }

        idt
    };
}

/// Loads the IDT.
pub fn init_idt() {
    IDT.load();
    ilogln!("IDT loaded");
}
