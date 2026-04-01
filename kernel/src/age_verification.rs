use alloc::{collections::VecDeque, string::String};
use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;
use spin::Mutex;

use internal_utils::gpu_device::{
    BLACK, GPU_DEVICE, GPUDeviceCapabilityMut, GPUDeviceCapabilityRequest, GREEN, RED, VGAColor,
    WHITE,
};

use crate::hlt_loop;
use crate::interrupts::{
    InterruptIndex, disable_irq, enable_irq, irq_enabled, register_key_listener,
};

const INPUT_LIMIT: usize = 3;
const PROMPT_X: u16 = 0;
const PROMPT_Y: u16 = 0;
const PROMPT_WIDTH: u16 = 360;
const PROMPT_HEIGHT: u16 = 32;

lazy_static! {
    static ref INPUT_BUFFER: Mutex<VecDeque<char>> = Mutex::new(VecDeque::new());
}

fn on_key_event(event: DecodedKey) {
    if let DecodedKey::Unicode(character) = event {
        INPUT_BUFFER.lock().push_back(character);
    }
}

pub fn block_until_age_verified() {
    let interrupts_were_enabled = x86_64::instructions::interrupts::are_enabled();
    let timer_was_enabled = irq_enabled(InterruptIndex::Timer);

    register_key_listener(on_key_event);
    disable_irq(InterruptIndex::Timer);
    if !interrupts_were_enabled {
        x86_64::instructions::interrupts::enable();
    }

    self::verify_age();

    if timer_was_enabled {
        enable_irq(InterruptIndex::Timer);
    }
    if !interrupts_were_enabled {
        x86_64::instructions::interrupts::disable();
    }
}

pub fn verify_age() {
    let mut input = String::new();

    loop {
        render_prompt(WHITE, &alloc::format!("Age: {}_", input));

        let ch = wait_for_key();

        match ch {
            '\n' => {
                if let Some(age) = parse_age(&input) {
                    if age >= 18 {
                        render_prompt(GREEN, "Access granted. Starting rost...");
                        return;
                    }

                    render_prompt(
                        RED,
                        &alloc::format!(
                            "Access denied. Please try again in {} year(s).",
                            18u8.saturating_sub(age)
                        ),
                    );
                    hlt_loop();
                }
            }
            '\u{8}' => {
                input.pop();
            }
            '0'..='9' if input.len() < INPUT_LIMIT => {
                input.push(ch);
            }
            _ => {}
        }
    }
}

fn wait_for_key() -> char {
    loop {
        if let Some(ch) = INPUT_BUFFER.lock().pop_front() {
            return ch;
        }

        x86_64::instructions::hlt();
    }
}

fn parse_age(input: &str) -> Option<u8> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        trimmed.parse().ok()
    }
}

fn render_prompt(color: VGAColor<u8>, message: &str) {
    let Some(mut device) = GPU_DEVICE.lock() else {
        return;
    };

    if let Some(GPUDeviceCapabilityMut::Shape(shape)) =
        device.get_capability_mut(GPUDeviceCapabilityRequest::Shape)
    {
        shape.fill_rectangle(PROMPT_X, PROMPT_Y, PROMPT_WIDTH, PROMPT_HEIGHT, BLACK);
    }

    if let Some(GPUDeviceCapabilityMut::Text(text)) =
        device.get_capability_mut(GPUDeviceCapabilityRequest::Text)
    {
        text.draw_string(PROMPT_X, PROMPT_Y, color, message, PROMPT_X);
    }
}
