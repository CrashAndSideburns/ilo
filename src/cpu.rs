pub struct Cpu {
    // The CPU has 32 registers. The zero register, or `registers[0]`, always outputs a value of
    // 0x0000 when it is read. The only reason that we allocate space for 32 registers here, rather
    // than 31, is just so that we can read from and write to registers with slightly more reckless
    // abandon.
    pub registers: [u16; 0x20],
    // The program counter points to the address in ram containing the next instruction to be
    // executed.
    pub program_counter: u16,
    // The RAM is somewhat unusual, in that its word size is 16 bits, rather than the more typical
    // 8 bits. Consequently, an address refers to a 16-bit value in RAM, rather than an 8-bit one.
    pub ram: [u16; 0x10000],
}

impl Cpu {
    // Construct a new CPU, initialized to a default state.
    pub fn new() -> Self {
        Self {
            // NOTE: The contents of all registers and RAM are initialized to 0x0000. Actual
            // hardware is unlikely to offer such a guarantee, so software should not rely on
            // these values.
            registers: [0x0000; 0x20],
            ram: [0x0000; 0x10000],

            // The program counter is guaranteed to always be initialized to 0x0000. Hardware must
            // also offer this guarantee.
            program_counter: 0x0000,
        }
    }

    // Stepping the CPU has the effect of executing the instruction to which the program counter
    // currently points, and advancing the program counter as appropriate to refer to the next
    // instruction.
    pub fn step(&mut self) {
        // The instruction to be executed is held at the address indicated by the program counter.
        // Not all instructions take immediate operands, but if they do, it will be stored in the
        // next address. Technically, the JSH instruction takes an immediate operand which is held
        // at the address indicated by the program counter, but this is a special case which we may
        // treat separately.
        let instruction = self.ram[usize::from(self.program_counter)];
        let immediate = self.ram[usize::from(self.program_counter.wrapping_add(1))];

        // Break up the instruction into its constituent parts for ease of access. Observe that it
        // is valuable to have a mutable reference to the destination register, but an instruction
        // will never write to its source register, so we only need the value. r0 is hardwired to
        // contain 0x0000.
        let source = if instruction & 0b1111100000000000 == 0x0000 {
            0x0000
        } else {
            self.registers[usize::from((instruction & 0b1111100000000000) >> 11)]
        };
        let destination = &mut self.registers[usize::from((instruction & 0b0000011111000000) >> 6)];
        let opcode = instruction & 0b0000000000111111;

        // At the end of the day, what is an emulator but socially acceptable trappings on a
        // massive switch statement?
        match opcode {
            0b000000 => {
                // ADD
                *destination = destination.wrapping_add(source);
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b000001 => {
                // SUB
                *destination = destination.wrapping_sub(source);
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b000010 => {
                // AND
                *destination &= source;
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b000011 => {
                // OR
                *destination |= source;
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b000100 => {
                // XOR
                *destination ^= source;
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b000101 => {
                // SLL
                *destination = if (source as i16).is_positive() {
                    *destination << source
                } else {
                    *destination >> -(source as i16)
                };
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b000110 => {
                // SRL
                *destination = if (source as i16).is_positive() {
                    *destination >> source
                } else {
                    *destination << -(source as i16)
                };
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b000111 => {
                // SRA
                *destination = if (source as i16).is_positive() {
                    ((*destination as i16) >> source) as u16
                } else {
                    *destination << -(source as i16)
                };
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b001000 => {
                // ADDI
                *destination = source.wrapping_add(immediate);
                self.program_counter = self.program_counter.wrapping_add(2);
            }
            0b001010 => {
                // ANDI
                *destination = source & immediate;
                self.program_counter = self.program_counter.wrapping_add(2);
            }
            0b001011 => {
                // ORI
                *destination = source | immediate;
                self.program_counter = self.program_counter.wrapping_add(2);
            }
            0b001100 => {
                // XORI
                *destination = source ^ immediate;
                self.program_counter = self.program_counter.wrapping_add(2);
            }
            0b001101 => {
                // SFTI
                *destination = if (immediate as i16).is_positive() {
                    source << immediate
                } else {
                    source >> -(immediate as i16)
                };
                self.program_counter = self.program_counter.wrapping_add(2);
            }
            0b001111 => {
                // SRAI
                *destination = if (immediate as i16).is_positive() {
                    ((source as i16) >> immediate) as u16
                } else {
                    source << -(immediate as i16)
                };
                self.program_counter = self.program_counter.wrapping_add(2);
            }
            0b010000 => {
                // LD
                *destination = self.ram[usize::from(source)];
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b010001 => {
                // ST
                self.ram[usize::from(source)] = *destination;
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            0b011000 => {
                // LDIO
                *destination = self.ram[usize::from(source.wrapping_add(immediate))];
                self.program_counter = self.program_counter.wrapping_add(2);
            }
            0b011001 => {
                // STIO
                self.ram[usize::from(source.wrapping_add(immediate))] = *destination;
                self.program_counter = self.program_counter.wrapping_add(2);
            }
            0b101000 => {
                // JAL
                *destination = self.program_counter.wrapping_add(2);
                self.program_counter = source.wrapping_add(immediate);
            }
            0b101001 => {
                // JSH
                let offset = (instruction & 0b1111111111000000) as i16 >> 6;
                self.program_counter = self.program_counter.wrapping_add_signed(offset);
            }
            0b101010 => {
                // BEQ
                if *destination == source {
                    self.program_counter = immediate;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
            }
            0b101011 => {
                // BNE
                if *destination != source {
                    self.program_counter = immediate;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
            }
            0b101100 => {
                // BLT
                if (*destination as i16) < (source as i16) {
                    self.program_counter = immediate;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
            }
            0b101101 => {
                // BGE
                if (*destination as i16) >= (source as i16) {
                    self.program_counter = immediate;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
            }
            0b101110 => {
                // BLTU
                if *destination < source {
                    self.program_counter = immediate;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
            }
            0b101111 => {
                // BGEU
                if *destination >= source {
                    self.program_counter = immediate;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
            }
            _ => {
                // NOTE: Here, all instructions which are not explicitly encoded are treated as
                // NOPs. A hardware implementation need not offer this guarantee, so code should
                // not rely on unused instrustions behaving as NOPs.
            }
        }
    }
}
