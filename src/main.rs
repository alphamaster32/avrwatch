#![allow(warnings, unused)]

mod arch;

use elven_parse::Elf;
use elven_parse::section::SectionType;

use core::arch::x86_64::_rdtsc;
use std::str;

use crate::arch::avr;

/// Error type for the emulator
#[derive(Debug)]
pub enum Error {
    InstReadError,
}

/// Wrapper type for the error result
type Result<T> = core::result::Result<T, Error>;

fn parse_shstrtab(file: &[u8]) -> Vec<String> {
    let mut res = vec![];

    let mut bits = vec![];
    for &byte in file {
        if byte == b'\0' {
            if bits.len() != 0 {
                // Remove null termination and '.' dot.
                let s = String::from(str::from_utf8(&bits.as_slice()[2..])
                    .unwrap());
                res.push(s);
                bits.clear();
            }
        }
        bits.push(byte.clone());
    }

    res
}

fn main() -> Result<()> {
    // Get the file handle for the intended binary and parse the elf
    let file = std::fs::read("test/gprs/gprs.ino.elf").unwrap();
    let elf = Elf::new(file.as_slice()).parse().unwrap();
    //let mut shstrtab = vec![];
    let mut sh: &[u8] = &[];

    // Check all the sections and load only the load section with the required
    // permissions
    for section in elf.section_iter() {
        if section.sh_type == SectionType::ShtStrTab &&
            elf.file_header.e_shstrndx as usize == section.sh_ndx {
            let str_tab = file.as_slice().get(section.sh_offset..
                (section.sh_offset + section.sh_size)).unwrap();
            sh = file.as_slice().get(section.sh_offset..
                (section.sh_offset + section.sh_size)).unwrap();
            let s = match str::from_utf8(str_tab) {
                Ok(v) => v,
                Err(e) => panic!("Could no parse the shstrtab"),
            };
            //shstrtab = parse_shstrtab(str_tab);
            break;
        }
    }

    // Calculate the end of the text section from SectionHeader
    let mut text_addr: usize = 0;
    let mut text_size: usize = 0;

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
            text_addr = section.sh_offset;
            text_size = section.sh_size;
        }
    }

    let mut idx: usize = text_addr;
    loop {
        let inst: arch::avr::Inst = u32::from_le_bytes(file[idx..(idx + 4)]
            .try_into().map_err(|_err| Error::InstReadError)?)
            .into();
        println!("{:x?}", inst);
        idx = idx + inst.opcode.increment_pc();
    }

    Ok(())
}
