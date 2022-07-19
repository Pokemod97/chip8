use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Chip8 {
    memory: [u8; 0xFFF],
    register_i: u16,
    registers: [u8; 16],
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    screen: [[bool; WIDTH]; HEIGHT],
    dt: u8,
    st: u8,
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            memory: [0; 0xFFF],
            register_i: 0,
            registers: [0; 16],
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            screen: [[false; WIDTH]; HEIGHT],
            dt: 0,
            st: 0,
        }
    }
}

impl Chip8 {
    pub fn run_instruction(&mut self, key: &mut Option<u8>, time: &mut Instant) -> bool {
        if self.pc >= 0xFFE {
            return false;
        }

        let instruction = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[(self.pc + 1) as usize] as u16;
        //println!("{:x}, {:x}", instruction, self.pc);
        let kk = (0xFF & instruction) as u8;
        let adr = 0xFFF & instruction;
        let n = 0xF & instruction;
        let y = self.registers[((instruction & 0x00F0) >> 4) as usize];
        let v0 = self.registers[0];
        let x = &mut self.registers[((instruction & 0x0F00) >> 8) as usize];
        let mut screen_modified = false;

        match instruction & 0xF000 {
            0 => {
                if kk == 0xEE {
                    self.pc = self.stack[self.sp as usize];
                    self.sp -= 1;
                    return screen_modified;
                } else if kk == 0xE0 {
                    self.screen.copy_from_slice(&[[false; 64]; 32]);
                    screen_modified = true;
                }
            }
            0x1000 => {
                self.pc = adr;
                return false;
            }
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc + 2;
                self.pc = adr;
                return screen_modified;
            }
            0x3000 => {
                if self.registers[((instruction & 0x0F00) >> 8) as usize] == kk {
                    self.pc += 2;
                }
            }
            0x4000 => {
                if *x != kk {
                    self.pc += 2;
                }
            }
            0x5000 => {
                if *x == y {
                    self.pc += 2;
                }
            }
            0x6000 => *x = kk,
            0x7000 => *x = x.overflowing_add(kk).0,
            0x8000 => match n {
                0 => *x = y,
                1 => *x |= y,
                2 => *x &= y,
                3 => *x ^= y,
                4 => {
                    let result = x.overflowing_add(y);
                    *x = result.0;
                    self.registers[0xF] = result.1 as u8;
                }
                5 => {
                    let result = x.overflowing_sub(y);
                    *x = result.0;
                    self.registers[0xF] = !result.1 as u8;
                }
                0x6 => {
                    let end_bit = y & 1;
                    *x = y >> 1;
                    self.registers[0xF] = end_bit;
                }
                0x7 => {
                    let result = x.overflowing_sub(y);
                    *x = result.0;
                    self.registers[0xF] = result.1 as u8;
                }
                0xE => {
                    let end_bit = (y >> 7) & 1;
                    *x = y << 1;
                    println!("{:x}, {:x}, {:x}", y, *x, end_bit);
                    self.registers[0xF] = end_bit;
                }
                _ => (),
            },
            0x9000 => {
                if *x != y {
                    self.pc += 2;
                }
            }
            0xA000 => {
                self.register_i = adr;
            }
            0xB000 => {
                self.pc = adr + v0 as u16;
                return screen_modified;
            }
            0xC000 => *x = kk & fastrand::u8(0..=255),
            0xD000 => {
                let x = (*x % 64) as usize;
                let y = (y % 32) as usize;
                let mut overwrite = false;
                for i in 0..n as usize {
                    let byte = self.memory[self.register_i as usize + i];

                    for q in 0..8 as usize {
                        let bit = ((byte >> q.abs_diff(7)) & 1) != 0;

                        match self.screen.get_mut(y + i) {
                            Some(u) => match u.get_mut(x + q) {
                                Some(handle) => {
                                    overwrite |= *handle ^ bit;
                                    *handle ^= bit;
                                    //println!("{handle},{bit}, {}, {}", y+i,x+q);
                                }
                                None => (),
                            },
                            None => (),
                        };
                    }
                }
                screen_modified = true;
                self.registers[0xF] = overwrite as u8;
            }
            0xE000 => match kk {
                0x9E => {
                    if *key != None && *x == key.unwrap() {
                        self.pc += 2;
                        *key = None;
                    }
                }
                0xA1 => match *key {
                    Some(k) => {
                        if *x != k {
                            self.pc += 2;
                            *key = None;
                        }
                    }
                    None => self.pc += 2,
                },
                _ => (),
            },
            0xF000 => match kk {
                0x07 => *x = self.dt,
                0x0A => {
                    if None == *key {
                        return screen_modified;
                    } else {
                        *x = key.unwrap();
                        *key = None;
                    }
                }
                0x15 => self.dt = *x,
                0x18 => self.st = *x,
                0x1E => self.register_i += *x as u16,
                0x29 => self.register_i = 5 * *x as u16,
                0x33 => {
                    self.memory[(self.register_i) as usize] = *x / 100;
                    self.memory[(self.register_i + 1) as usize] = (*x % 100) / 10;
                    self.memory[(self.register_i + 2) as usize] = *x % 10;
                }
                0x55 => {
                    let section = &self.registers[0..=((instruction & 0x0F00) >> 8) as usize];
                    for (i, k) in section.iter().enumerate() {
                        self.memory[(self.register_i as usize) + i] = *k;
                    }
                }
                0x65 => {
                    let section = &mut self.registers[0..=((instruction & 0x0F00) >> 8) as usize];
                    for (i, k) in section.iter_mut().enumerate() {
                        *k = self.memory[(self.register_i as usize) + i];
                    }
                }
                _ => (),
            },

            _ => (),
        }
        if time.elapsed() >= Duration::new(0, 16666667) {
            if self.st > 0 {
                self.st -= 1;
            }
            if self.dt > 0 {
                self.dt -= 1;
            }
            *time = Instant::now();
        }

        self.pc += 2;
        screen_modified
    }
    pub fn draw(&self) -> Vec<u8> {
        let mut pixels: Vec<u8> = Vec::new();
        for y in self.screen {
            for x in y {
                if x {
                    pixels.push(0xFF);
                    pixels.push(0xFF);
                    pixels.push(0xFF);
                    pixels.push(0xFF);
                } else {
                    pixels.push(0x0);
                    pixels.push(0xF0);
                    pixels.push(0);
                    pixels.push(0xFF);
                }
            }
        }
        pixels
    }
    pub fn setup(path: &Path) -> Chip8 {
        let mut chip8: Chip8 = Chip8::default();
        let font: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];
        chip8.memory[..80].copy_from_slice(&font);
        let bytes = fs::read(path).expect("failed to read file");
        //let bytes = vec![0x0,0xE0, 0x60, 0x0F, 0xF0, 0x29, 0xD1, 0x15];
        chip8.memory[0x200..(0x200 + bytes.len())].copy_from_slice(&bytes);
        //chip8.memory[0x1FF] = 5;
        chip8
    }
}
