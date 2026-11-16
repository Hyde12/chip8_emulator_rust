const SCREEN_WIDTH: usize = 64;
const SCREEN_LENGTH: usize = 32;

const RAM_SIZE: usize = 4096;
const REGISTERS_AMOUNT: usize = 16;
const STACK_SIZE: usize = 16;

const PROGRAM_START: u16 = 0x200;

const FONT_START: usize = 0x050;
const FONT_SIZE: usize = 80;
const FONT_END: usize = FONT_START + FONT_SIZE;

const FONT: [u8; FONT_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct emulator {
    program_counter: u16, // points at current instruction in memory
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_LENGTH],
    stack: [u16; STACK_SIZE],
    v_registers: [u8; REGISTERS_AMOUNT], // variable registers (V0 to VF)
    i_register: u16,                     // points at locations in memory
    stack_pointer: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl emulator {
    pub fn new() -> Self {
        let mut emulator = emulator {
            program_counter: PROGRAM_START,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_LENGTH],
            stack: [0; STACK_SIZE],
            v_registers: [0; REGISTERS_AMOUNT],
            i_register: 0,
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
        };

        emulator.ram[FONT_START..=FONT_END].copy_from_slice(&FONT);

        emulator
    }

    fn push(&mut self, val: u16) {
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;

        // can lwk js return orig val but it looks cooler this way and sets the memory back to 0
        let stack_ref = &mut self.stack[self.stack_pointer as usize];
        let val = *stack_ref;
        *stack_ref = 0;

        val
    }
}
