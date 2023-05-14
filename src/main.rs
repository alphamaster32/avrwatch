#![allow(warnings, unused)]
extern crate capstone;

use elven_parse::Elf;
use elven_parse::section::SectionType;
use capstone::prelude::*;

use core::arch::x86_64::_rdtsc;
use std::str;

//mod arch;

/// Error type for the emulator
#[derive(Debug)]
pub enum Error {
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
    let file = std::fs::read("test/nrf9160/obj/nrf9160.elf").unwrap();
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

    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .syntax(arch::arm::ArchSyntax::NoRegName)
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

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
            let insns = cs.disasm_all(
                &file[section.sh_offset..(section.sh_offset + section.sh_size)],
                section.sh_addr as u64)
                .expect("Failed to disassemble");

            println!("Found {} instructions", insns.len());
            for i in insns.as_ref() {
                println!();
                println!("{}", i);

                let detail: InsnDetail = cs.insn_detail(&i).expect("Failed to get insn detail");
                let arch_detail: ArchDetail = detail.arch_detail();
                let ops = arch_detail.operands();

                let output: &[(&str, String)] = &[
                    ("insn id:", format!("{:?}", i.id().0)),
                    ("bytes:", format!("{:?}", i.bytes())),
                ];

                for &(ref name, ref message) in output.iter() {
                    println!("{:4}{:12} {}", "", name, message);
                }

                println!("{:4}operands: {}", "", ops.len());
                for op in ops {
                    println!("{:8}{:?}", "", op);
                }
            }
        }
    }



    // Calculate the end of the text section from SectionHeader
    let mut text_addr: usize = 0;
    let mut text_size: usize = 0;
    for section in elf.section_iter() {
        if section.sh_flags.is_exec() && section.sh_flags.is_alloc() {
            text_addr = section.sh_addr;
            text_size = section.sh_size;
        }
    }

    Ok(())
}
