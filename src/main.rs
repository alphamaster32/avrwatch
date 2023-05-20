#![allow(warnings, unused)]

mod arch;
mod dwarf;

use elven_parse::Elf;
use elven_parse::file::ElfData;
use elven_parse::section::SectionType;

use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::str;

use crate::arch::avr::{*, Opcode};

// Corresponds to the precentage of the steps we need to take
const STEP: f64 = 0.01;

/// Error type for the emulator
#[derive(Debug)]
pub enum Error {
    InstReadError,
}

/// Wrapper type for the error result
type Result<T> = core::result::Result<T, Error>;

fn main() -> Result<()> {
    let projects = vec![
        //"test/gprs/gprs.ino.elf",
        "test/mqtt/mqtt.ino.elf",
        //"test/sdcard/sdcard.ino.elf",
        //"test/wifi/AdvancedChatServer/AdvancedChatServer.ino.elf",
        //"test/wifi/ConfigurationAP/ConfigurationAP.ino.elf",
        //"test/wifi/SDWebServer/SDWebServer.ino.elf",
        //"test/wifi/TelnetConsole/TelnetConsole.ino.elf",
        //"test/wifi/TemporaryWiFiConnect/TemporaryWiFiConnect.ino.elf",
        //"test/wifi/WebServerSSL/WebServerSSL.ino.elf",
        //"test/ble/LED/LED.ino.elf",
        //"test/ble/BatteryMonitor/BatteryMonitor.ino.elf",
        //"test/ble/ButtonLED/ButtonLED.ino.elf",
        //"test/ble/CallbackLED/CallbackLED.ino.elf",
        //"test/ble/EncryptedBatteryMonitor/EncryptedBatteryMonitor.ino.elf",
    ];
    let mut percent: [f64; 100] = [0.0; 100];
    // Get the file handle for the intended binary and parse the elf
    for f in &projects {
        let counter: AtomicUsize = AtomicUsize::new(0);
        let file = std::fs::read(f).unwrap();
        let elf = Elf::new(file.as_slice()).parse().unwrap();
        let mut sh: &[u8] = &[];

        // Find the section header strtab
        for section in elf.section_iter() {
            if section.sh_type == SectionType::ShtStrTab &&
                elf.file_header.e_shstrndx as usize == section.sh_ndx {
                    sh = file.as_slice().get(section.sh_offset..
                        (section.sh_offset + section.sh_size)).unwrap();
                    break;
                }
        }

        // Calculate the end of the text section from SectionHeader
        let mut text_addr: usize = 0;
        let mut text_size: usize = 0;
        let mut text_offset: usize = 0;

        for section in elf.section_iter() {
            let mut bits = vec![];
            // Skip the `sh_name` bytes to get to the required index
            let it = sh.iter().skip(section.sh_name as usize);
            // Parse the byte until null termination
            for &byte in it {
                if byte == b'\0' {
                    if bits.len() != 0 {
                        break;
                    }
                    continue;
                }
                bits.push(byte.clone());
            }
            // Parse the string from byte slice
            let name = String::from(str::from_utf8(&bits.as_slice()[1..])
                .unwrap());
            if name == "text" {
                text_addr = section.sh_addr;
                text_size = section.sh_size;
                text_offset = section.sh_offset;
            }
        }

        // Parse the debug symbols
        
        // Count all the instructions
        let mut size: usize = text_size;
        let mut inst_count: usize = 0;
        let mut io_count: usize = 0;
        let mut pc: usize = text_addr;
        loop {
            let mut inst: arch::avr::Inst = u32::from_le_bytes(
                file[((pc - text_addr) + text_offset)..((pc - text_addr) + text_offset + 4)]
                .try_into().map_err(|_err| Error::InstReadError)?)
                .into();
            inst.pc = pc;
            inst_count += 1;
            match inst.opcode {
                Opcode::IN => {
                    io_count += 1;
                },
                Opcode::OUT => {
                    io_count += 1;
                },
                _ => {},
            }
            if size <= 0 || pc >= (text_addr + text_size) {
                break;
            }
            pc = pc + inst.opcode.increment_pc();
            size = size.saturating_sub(inst.opcode.increment_pc());
        }
        println!("instruction count: {:?}, IO count: {:?}", inst_count,
            io_count);

        // Parse the assembly instructions
        for i in 0..=99 {
            let mut size: usize = (STEP * text_size as f64) as usize;
            let mut inst_count: usize = 0;
            // FIXME: Feels a little bad
            let mut pc: usize = text_addr + (size * i);
            let mut jmp_stack: Vec<usize> = Vec::new();
            // Alignment
            pc = (pc & 0b1) ^ pc;
            //println!("{:x?}", pc);
            if (pc > text_offset + text_size) {
                panic!();
            }
            counter.store(0, Ordering::SeqCst);
            loop {
                //println!("{:x?}", pc);
                let mut inst: arch::avr::Inst = u32::from_le_bytes(
                    file[((pc - text_addr) + text_offset)..((pc - text_addr) + text_offset + 4)]
                    .try_into().map_err(|_err| Error::InstReadError)?)
                    .into();
                inst_count += 1;
                inst.pc = pc;
                if inst.opcode == Opcode::JMP {
                    pc = inst.imm.unwrap() as usize;
                    size = size.saturating_sub(inst.opcode.increment_pc());
                    //println!("{:x?}", inst);
                    continue;
                }
                if inst.opcode == Opcode::RJMP {
                    let prev_pc = pc;
                    pc = (pc as i32 + ((1 + inst.imm.unwrap()) << 1)) as usize;
                    if prev_pc == pc {
                        pc = pc + inst.opcode.increment_pc();
                    }
                    size = size.saturating_sub(inst.opcode.increment_pc());
                    //println!("{:x?}", inst);
                    continue;
                }
                if inst.opcode == Opcode::RCALL {
                    jmp_stack.push(pc + inst.opcode.increment_pc());
                    pc = pc + ((1 + (inst.imm.unwrap())) * 2) as usize;
                    size = size.saturating_sub(inst.opcode.increment_pc());
                    //println!("{:x?}", inst);
                    continue;
                }
                if inst.opcode == Opcode::CALL {
                    jmp_stack.push(pc + inst.opcode.increment_pc());
                    pc = inst.imm.unwrap() as usize;
                    size = size.saturating_sub(inst.opcode.increment_pc());
                    //println!("{:x?}", inst);
                    continue;
                }
                if inst.opcode == Opcode::RET {
                    if jmp_stack.len() != 0 {
                        pc = jmp_stack.pop().unwrap();;
                    } else {
                        pc = pc + inst.opcode.increment_pc();
                    }
                    size = size.saturating_sub(inst.opcode.increment_pc());
                    //println!("{:x?}", inst);
                    continue;
                }
                match inst.opcode {
                    Opcode::IN => {
                        counter.fetch_add(1, Ordering::SeqCst);
                    },
                    Opcode::OUT => {
                        counter.fetch_add(1, Ordering::SeqCst);
                    },
                    _ => {},
                }
                //println!("{:x?}", inst);
                if size <= 0 || (pc - 2) > text_addr + text_size {
                    break;
                }
                pc = pc + inst.opcode.increment_pc();
                size = size.saturating_sub(inst.opcode.increment_pc());
            }

            percent[i] = percent[i] + counter.load(Ordering::SeqCst) as f64;

            println!("it: {}, IO: {:?}, count: {:?}", i, counter, inst_count);
        }

    }
    //for (i, el) in percent.iter().enumerate() {
        //println!("Percentage {:?}%:    {:?}", i+1, el / projects.len() as f64);
    //}
    Ok(())
}
