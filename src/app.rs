use anyhow::{anyhow, Result};

use regex::RegexBuilder;

use std::collections::VecDeque;
use std::fs;

use crate::cpu::Cpu;

pub struct App {
    pub cpu: Cpu,
    pub command_buffer: String,
    pub command_result: Result<String>,
    pub instruction_history: VecDeque<(u16, u16)>,
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            command_buffer: String::new(),
            command_result: Ok(String::new()),
            instruction_history: VecDeque::new(),
            running: false,
        }
    }

    pub fn step(&mut self) {
        // Update the instruction history. Make sure that it doesn't grow too large in a rather
        // lazy way.
        let instruction = self.cpu.ram[usize::from(self.cpu.program_counter)];
        let immediate = self.cpu.ram[usize::from(self.cpu.program_counter.wrapping_add(1))];
        self.instruction_history.push_back((instruction, immediate));
        if self.instruction_history.len() > 0xff {
            self.instruction_history.pop_front();
        }

        // Step the CPU.
        self.cpu.step();
    }

    pub fn execute_command(&mut self) {
        self.command_result = self.execute_command_with_result();
        self.command_buffer.clear();
    }

    pub fn execute_command_with_result(&mut self) -> Result<String> {
        // Build a whole bunch of regexes which are used to match commands.
        let run_regex = RegexBuilder::new(r"^\s*run\s*$")
            .case_insensitive(true)
            .build()
            .unwrap();
        let halt_regex = RegexBuilder::new(r"^\s*halt\s*$")
            .case_insensitive(true)
            .build()
            .unwrap();
        let step_regex = RegexBuilder::new(r"^\s*step(?:\s+(?:(?<decimal_literal>[0-9]+)|0x(?<hex_literal>[0-9a-f]+)|0b(?<binary_literal>[01]+)))?\s*$").case_insensitive(true).build().unwrap();
        let load_regex = RegexBuilder::new(r"^\s*load(?:\s+(?:(?<decimal_literal>[0-9]+)|0x(?<hex_literal>[0-9a-f]+)|0b(?<binary_literal>[01]+)))?\s+(?<filename>.+)\s*$")
            .case_insensitive(true)
            .build()
            .unwrap();

        // HACK: There's a tonne of repeated code in here for parsing numeric literals. A lot of
        // the error messages also offer... questionable levels of clarity.
        if run_regex.is_match(&self.command_buffer) {
            self.running = true;
            Ok("Running simulation.".into())
        } else if halt_regex.is_match(&self.command_buffer) {
            self.running = false;
            Ok(format!(
                "Simulation halted at {:#06x}.",
                self.cpu.program_counter
            ))
        } else if let Some(caps) = step_regex.captures(&self.command_buffer) {
            let step_size = if let Some(decimal_literal) = caps.name("decimal_literal") {
                u16::from_str_radix(decimal_literal.as_str(), 10)?
            } else if let Some(hex_literal) = caps.name("hex_literal") {
                u16::from_str_radix(hex_literal.as_str(), 16)?
            } else if let Some(binary_literal) = caps.name("binary_literal") {
                u16::from_str_radix(binary_literal.as_str(), 2)?
            } else {
                1
            };

            for _ in 0..step_size {
                self.step()
            }

            Ok(format!("Stepping simulation {:#06x} times.", step_size))
        } else if let Some(caps) = load_regex.captures(&self.command_buffer) {
            let address = if let Some(decimal_literal) = caps.name("decimal_literal") {
                u16::from_str_radix(decimal_literal.as_str(), 10)?
            } else if let Some(hex_literal) = caps.name("hex_literal") {
                u16::from_str_radix(hex_literal.as_str(), 16)?
            } else if let Some(binary_literal) = caps.name("binary_literal") {
                u16::from_str_radix(binary_literal.as_str(), 2)?
            } else {
                0
            };

            let bytes = fs::read(&caps["filename"])?;
            let words = bytes
                .chunks_exact(2)
                .map(|c| u16::from_ne_bytes([c[1], c[0]]))
                .collect::<Vec<_>>();
            self.cpu.ram[usize::from(address)..(usize::from(address) + bytes.len() / 2)]
                .copy_from_slice(&words);

            Ok(format!(
                "Loaded {:#06x} words from {} into RAM at address {:#06x}.",
                words.len(),
                &caps["filename"],
                address
            ))
        } else {
            Err(anyhow!(
                "\"{}\" is not a valid command. Supported commands are RUN, HALT, STEP, SET, and LOAD.",
                self.command_buffer.trim()
            ))
        }
    }
}
