use rand::{self, random};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_LENGTH: usize = 32;

const RAM_SIZE: usize = 4096;
const REGISTERS_AMOUNT: usize = 16;
const STACK_SIZE: usize = 16;
const KEYS_AMOUNT: usize = 16;

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
    keys: [bool; KEYS_AMOUNT],
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
            keys: [false; KEYS_AMOUNT],
        };

        emulator.ram[FONT_START..FONT_END].copy_from_slice(&FONT);

        emulator
    }

    pub fn load(&mut self, rom_data: Vec<u8>) {
        self.ram[PROGRAM_START as usize..PROGRAM_START as usize + rom_data.len()]
            .copy_from_slice(&rom_data);
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

    // Fetch, Decode, Execute loop running per tick
    pub fn fde_loop(&mut self) {
        let instruction = self.fetch();
        self.decode_and_execute(instruction);
    }

    pub fn get_screen(&self) -> &[bool] {
        &self.screen
    }

    pub fn timer_tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn set_key_state(&mut self, key_index: usize, is_pressed: bool) {
        self.keys[key_index] = is_pressed;
    }

    fn fetch(&mut self) -> u16 {
        let first_bit = self.ram[self.program_counter as usize] as u16;
        let second_bit = self.ram[self.program_counter as usize + 1] as u16;

        self.program_counter += 2;

        (first_bit << 8) | second_bit // Moves 8 bits to the left and combines with the second bit as a u16
    }

    fn decode_and_execute(&mut self, instruction: u16) {
        // binary AND masking
        let first_nibble = (instruction & 0xF000) >> 12;
        let second_nibble = (instruction & 0x0F00) >> 8;
        let third_nibble = (instruction & 0x00F0) >> 4;
        let fourth_nibble = instruction & 0x000F;

        match (first_nibble, second_nibble, third_nibble, fourth_nibble) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_LENGTH],
            (0, 0, 0xE, 0xE) => self.program_counter = self.pop(),
            (1, _, _, _) => self.program_counter = instruction & 0x0FFF,
            (2, _, _, _) => {
                self.push(self.program_counter);
                self.program_counter = instruction & 0x0FFF;
            }
            (3, _, _, _) => {
                if self.v_registers[second_nibble as usize] == (instruction & 0x00FF) as u8 {
                    self.program_counter += 2;
                }
            }
            (4, _, _, _) => {
                if self.v_registers[second_nibble as usize] != (instruction & 0x00FF) as u8 {
                    self.program_counter += 2;
                }
            }
            (5, _, _, 0) => {
                if self.v_registers[second_nibble as usize]
                    == self.v_registers[third_nibble as usize]
                {
                    self.program_counter += 2;
                }
            }
            (6, _, _, _) => self.v_registers[second_nibble as usize] = (instruction & 0x00FF) as u8,
            (7, _, _, _) => {
                self.v_registers[second_nibble as usize] = self.v_registers[second_nibble as usize]
                    .wrapping_add(instruction as u8 & 0x00FF)
            }
            (8, _, _, 0) => {
                self.v_registers[second_nibble as usize] = self.v_registers[third_nibble as usize]
            }
            (8, _, _, 1) => {
                self.v_registers[second_nibble as usize] |= self.v_registers[third_nibble as usize]
            }
            (8, _, _, 2) => {
                self.v_registers[second_nibble as usize] &= self.v_registers[third_nibble as usize]
            }
            (8, _, _, 3) => {
                self.v_registers[second_nibble as usize] ^= self.v_registers[third_nibble as usize]
            }
            (8, _, _, 4) => {
                let vx = self.v_registers[second_nibble as usize];
                let vy = self.v_registers[third_nibble as usize];

                let (result, did_overflow) = vx.overflowing_add(vy);

                self.v_registers[second_nibble as usize] = result;
                self.v_registers[0xF as usize] = if did_overflow { 1 } else { 0 }
            }
            (8, _, _, 5) => {
                let vx = self.v_registers[second_nibble as usize];
                let vy = self.v_registers[third_nibble as usize];

                let (result, did_underflow) = vx.overflowing_sub(vy);

                self.v_registers[second_nibble as usize] = result;
                self.v_registers[0xF as usize] = if did_underflow { 0 } else { 1 }
            }
            (8, _, _, 6) => {
                // should be configurable
                // self.v_registers[second_nibble as usize] = self.v_registers[third_nibble as usize];

                let shifted_bit = (self.v_registers[second_nibble as usize] >> 1) & 1;
                self.v_registers[second_nibble as usize] >>= 1;

                self.v_registers[0xF as usize] = if shifted_bit == 1 { 1 } else { 0 }
            }
            (8, _, _, 7) => {
                let vx = self.v_registers[second_nibble as usize];
                let vy = self.v_registers[third_nibble as usize];

                let (result, did_underflow) = vy.overflowing_sub(vx);

                self.v_registers[second_nibble as usize] = result;
                self.v_registers[0xF as usize] = if did_underflow { 0 } else { 1 }
            }
            (8, _, _, 0xE) => {
                // should be configurable
                self.v_registers[second_nibble as usize] = self.v_registers[third_nibble as usize];

                let shifted_bit = (self.v_registers[second_nibble as usize] >> 7) & 1;
                self.v_registers[second_nibble as usize] <<= 1;

                self.v_registers[0xF as usize] = if shifted_bit == 1 { 1 } else { 0 }
            }
            (9, _, _, 0) => {
                if self.v_registers[second_nibble as usize]
                    != self.v_registers[third_nibble as usize]
                {
                    self.program_counter += 2;
                }
            }
            (0xA, _, _, _) => {
                self.i_register = (second_nibble << 8) | (third_nibble << 4) | fourth_nibble
            }
            (0xB, _, _, _) => {
                self.program_counter = (self.v_registers[0] as u16) + (instruction & 0x0FFF)
            }
            (0xC, _, _, _) => {
                let random: u8 = random();
                self.v_registers[second_nibble as usize] = random & ((instruction & 0x00FF) as u8)
            }
            (0xD, _, _, _) => {
                let x_coordinate = self.v_registers[second_nibble as usize];
                let y_coordinate = self.v_registers[third_nibble as usize];
                let height = fourth_nibble as u8;
                let mut flipped = false;

                for sprite_y in 0..height {
                    let sprite_row = self.i_register + sprite_y as u16;
                    let sprite = self.ram[sprite_row as usize];
                    let y = y_coordinate + sprite_y;

                    if y >= SCREEN_LENGTH as u8 {
                        break;
                    }

                    for sprite_x in 0..8 {
                        let x = x_coordinate + sprite_x;

                        if x >= SCREEN_WIDTH as u8 {
                            break;
                        }

                        let screen_point = y as usize * SCREEN_WIDTH + x as usize;

                        if (sprite & (0b10000000 >> sprite_x)) != 0 {
                            if self.screen[screen_point] {
                                self.screen[screen_point] = false;
                                flipped = true;
                            } else {
                                self.screen[screen_point] = true;
                            }
                        }
                    }
                }

                self.v_registers[0xF] = if flipped { 1 } else { 0 };
            }
            (0xE, _, 9, 0xE) => {
                if self.keys[self.v_registers[second_nibble as usize] as usize] {
                    self.program_counter += 2
                }
            }
            (0xE, _, 0xA, 1) => {
                if !self.keys[self.v_registers[second_nibble as usize] as usize] {
                    self.program_counter += 2
                }
            }
            (0xF, _, 0, 7) => self.v_registers[second_nibble as usize] = self.delay_timer,
            (0xF, _, 0, 0xA) => match self.keys.iter().position(|is_pressed| *is_pressed) {
                Some(key_pos) => self.v_registers[second_nibble as usize] = key_pos as u8,
                None => self.program_counter -= 2,
            },
            (0xF, _, 1, 5) => self.delay_timer = self.v_registers[second_nibble as usize],
            (0xF, _, 1, 8) => self.sound_timer = self.v_registers[second_nibble as usize],
            (0xF, _, 1, 0xE) => self.i_register += self.v_registers[second_nibble as usize] as u16,
            (0xF, _, 2, 9) => self.i_register = self.v_registers[second_nibble as usize] as u16,
            (0xF, _, 3, 3) => {
                let number = self.v_registers[second_nibble as usize] as f32;

                self.ram[self.i_register as usize] = (number / 100.0).floor() as u8;
                self.ram[self.i_register as usize + 1] = ((number / 10.0) % 10.0).floor() as u8;
                self.ram[self.i_register as usize + 2] = (number % 10.0).floor() as u8;
            }
            (0xF, _, 5, 5) => {
                for i in 0..=second_nibble as usize {
                    self.ram[self.i_register as usize + i] = self.v_registers[i];
                }
            }
            (0xF, _, 6, 5) => {
                for i in 0..=second_nibble as usize {
                    self.v_registers[i] = self.ram[self.i_register as usize + i];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented code: {}", instruction),
        }
    }
}
