pub fn disassemble(instruction: u16, immediate: u16) -> String {
    let source = format!("{:02}", (instruction & 0b1111100000000000) >> 11);
    let destination = format!("{:02}", (instruction & 0b0000011111000000) >> 6);

    match instruction & 0b0000000000111111 {
        0b000000 => {
            format!("ADD  r{}, r{}", destination, source)
        }
        0b000001 => {
            format!("SUB  r{}, r{}", destination, source)
        }
        0b000010 => {
            format!("AND  r{}, r{}", destination, source)
        }
        0b000011 => {
            format!("OR   r{}, r{}", destination, source)
        }
        0b000100 => {
            format!("XOR  r{}, r{}", destination, source)
        }
        0b000101 => {
            format!("SLL  r{}, r{}", destination, source)
        }
        0b000110 => {
            format!("SRL  r{}, r{}", destination, source)
        }
        0b000111 => {
            format!("SRA  r{}, r{}", destination, source)
        }
        0b001000 => {
            format!(
                "ADDI r{}, r{}, {:#06x}",
                destination, source, immediate as i16
            )
        }
        0b001010 => {
            format!("ANDI r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b001011 => {
            format!("ORI  r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b001100 => {
            format!("XORI r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b001101 => {
            format!(
                "SFTI r{}, r{}, {:#06x}",
                destination, source, immediate as i16
            )
        }
        0b001111 => {
            format!("SRAI r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b010000 => {
            format!("LD   r{}, r{}", destination, source)
        }
        0b010001 => {
            format!("ST   r{}, r{}", destination, source)
        }
        0b011000 => {
            format!("LDIO r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b011001 => {
            format!("STIO r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b101000 => {
            format!("JAL  r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b101001 => {
            let offset = (instruction & 0b1111111111000000) as i16 >> 6;
            format!("JSH            {:#06x}", offset)
        }
        0b101010 => {
            format!("BEQ  r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b101011 => {
            format!("BNE  r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b101100 => {
            format!("BLT  r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b101101 => {
            format!("BGE  r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b101110 => {
            format!("BLTU r{}, r{}, {:#06x}", destination, source, immediate)
        }
        0b101111 => {
            format!("BGEU r{}, r{}, {:#06x}", destination, source, immediate)
        }
        _ => String::new(),
    }
}
