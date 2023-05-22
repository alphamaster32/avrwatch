#![allow(warnings, unused)]

mod arch;

use elven_parse::Elf;
use elven_parse::file::ElfData;
use elven_parse::section::SectionType;
use ego_tree::Tree;
use ego_tree::NodeRef;
use rand::Rng;

use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::str;

use std::io::Write;
use std::io::LineWriter;

use crate::arch::avr::{*, Opcode};

// Corresponds to the precentage of the steps we need to take
const STEP: f64 = 0.01;
const PATHS: usize = 4;

/// Error type for the emulator
#[derive(Debug)]
pub enum Error {
    InstReadError,
}

#[derive(Debug, Clone)]
pub struct ExecutionNode {
    /// Inclusive start pc
    pub start: Option<usize>,
    /// Inclusive end pc
    pub end: Option<usize>,
    /// Number of instruction in that node
    pub inst_count: usize,
    /// Number of IO commands in that node
    pub io_count: usize,
    /// States that the node is processed
    pub processed: bool,
    /// Node jump stack
    pub jmp_stack: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct WriteInfo {
    pub inst_count: f64,
    pub io_count: f64,
    pub percentage: usize,
}

impl ExecutionNode {
    pub fn new() -> Self {
        ExecutionNode {
            start: None,
            end: None,
            inst_count: 0,
            io_count: 0,
            processed: false,
            jmp_stack: vec![],
        }
    }
}

impl WriteInfo {
    pub fn new(inst_count: f64, io_count: f64, percentage: usize) -> Self {
        WriteInfo {
            inst_count,
            io_count,
            percentage,
        }
    }
}

/// Wrapper type for the error result
type Result<T> = core::result::Result<T, Error>;

fn main() -> Result<()> {
    // An tuple representing the elf file and the end of the loop
    let projects = vec![
        //("test/gprs/gprs.ino.elf", 0x50d6 ,"gprs"),
        //("test/mqtt/mqtt.ino.elf", 0x43fc, "mqtt"),
        //("test/sdcard/sdcard.ino.elf", 0x5420, "sdcard")
        //("test/wifi/AdvancedChatServer/AdvancedChatServer.ino.elf", 0x28b8, "chat"),
        //"test/wifi/ConfigurationAP/ConfigurationAP.ino.elf",
        //"test/wifi/SDWebServer/SDWebServer.ino.elf",
        //"test/wifi/TelnetConsole/TelnetConsole.ino.elf",
        //"test/wifi/TemporaryWiFiConnect/TemporaryWiFiConnect.ino.elf",
        //"test/wifi/WebServerSSL/WebServerSSL.ino.elf",
        //"test/ble/LED/LED.ino.elf",
        //("test/ble/BatteryMonitor/BatteryMonitor.ino.elf", 0xa056 ,"battery_io"),
        //"test/ble/ButtonLED/ButtonLED.ino.elf",
        //("test/ble/CallbackLED/CallbackLED.ino.elf", 0xa01e, "callback_led"),
        //"test/ble/EncryptedBatteryMonitor/EncryptedBatteryMonitor.ino.elf",
        //("test/circuitdigest/voice/voice.ino.elf", [0x6180, 0xffff] , "voice")
        //("test/circuitdigest/gprs_gsm/gprs_gsm.ino.elf", [0x23b0, 0x359c] , "gprs_gsm")
        //("test/circuitdigest/rf24/rf24.ino.elf", [0x164e, 0xffff] , "rf24"),
        //("test/circuitdigest/sdcard/sdcard.ino.elf", [0x27c4, 0x2614] , "sdcard"),
        //("test/circuitdigest/lte/lte.ino.elf", [0x218c, 0xffff] , "lte"),
        //("test/circuitdigest/wifi/wifi.ino.elf", [0x17f0, 0xffff] , "wifi"),
        //("test/circuitdigest/gps/gps.ino.elf", [0x3b4a, 0x4242] , "gps"),
        //("test/circuitdigest/gsm/gsm.ino.elf", [0xd8c, 0xffff] , "gsm"),
        ("test/circuitdigest/bluetooth/bluetooth.ino.elf", [0xcb4, 0xd84] , "bluetooth"),
    ];

    // Setup path tree
    let mut path_tree: Tree<ExecutionNode> = Tree::new(ExecutionNode::new());

    // Setup the percentage array
    let mut percent: [f64; 100] = [0.0; 100];

    // Get the file handle for the intended binary and parse the elf
    for f in &projects {
        let counter: AtomicUsize = AtomicUsize::new(0);
        let file = std::fs::read(f.0).unwrap();
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

        let mut root = path_tree.root_mut();
        root.value().start = Some(elf.file_header.e_entry);
        let mut pc = elf.file_header.e_entry;
        let mut rng = rand::thread_rng();
        let mut taken_node = ExecutionNode::new();
        let mut not_taken_node = ExecutionNode::new();
        let mut copy_tree = path_tree.clone();
        for i in 1..=PATHS {
            for c in copy_tree.nodes() {
                let node_id = c.id();
                let mut node_mut = path_tree.get_mut(node_id).unwrap();
                if node_mut.value().processed == true {
                    continue;
                }
                let mut taken_node = ExecutionNode::new();
                let mut not_taken_node = ExecutionNode::new();
                pc = node_mut.value().start.unwrap();
                //println!("{:?}", node_mut.id());
                loop {
                    let mut inst: arch::avr::Inst = u32::from_le_bytes(
                        file[((pc - text_addr) + text_offset)..((pc - text_addr) +
                            text_offset + 4)]
                        .try_into().map_err(|_err| Error::InstReadError)?)
                        .into();
                    inst.pc = pc;
                    //println!("{:x?}", inst);
                    match inst.opcode {
                        Opcode::IN => {
                            node_mut.value().io_count += 1;
                            node_mut.value().inst_count += 1;
                        },
                        Opcode::OUT => {
                            node_mut.value().io_count += 1;
                            node_mut.value().inst_count += 1;
                        },
                        Opcode::JMP => {
                            pc = (inst.imm.unwrap() - inst.opcode.increment_pc()
                                as i32) as usize;
                            node_mut.value().inst_count += 1;
                        },
                        Opcode::BRNE | Opcode::BREQ | Opcode::BRCS |
                            Opcode::BRTC | Opcode::BRCC | Opcode::BRPL |
                            Opcode::BRGE | Opcode::BRLT | Opcode::BRMI => {
                                let prev_pc = pc - inst.opcode.increment_pc() as usize;
                                node_mut.value().end = Some(prev_pc +
                                    inst.opcode.increment_pc());
                                node_mut.value().inst_count += 1;
                                node_mut.value().processed = true;
                                pc = (((pc as i32 + ((1 + inst.imm.unwrap()) << 1))) -
                                    inst.opcode.increment_pc() as i32) as usize;
                                // Avoid getting in dead loop FIXME: END?
                                if prev_pc == pc {
                                    panic!("dead loop");
                                }
                                // We are done with the tree;
                                //println!("{:x?}", inst);
                                taken_node.start = Some(pc + 2);
                                taken_node.jmp_stack = node_mut.value().jmp_stack.clone();
                                not_taken_node.start = Some(prev_pc +
                                    inst.opcode.increment_pc() + 2);
                                not_taken_node.jmp_stack = node_mut.value().jmp_stack.clone();
                                break;
                            },
                        Opcode::RJMP => {
                            let prev_pc = pc - inst.opcode.increment_pc() as usize;
                            pc = (((pc as i32 + ((1 + inst.imm.unwrap()) << 1))) -
                                inst.opcode.increment_pc() as i32) as usize;
                            node_mut.value().inst_count += 1;
                            // Avoid getting inside a dead loop
                            // FIXME: END?
                            if prev_pc == pc {
                                break;
                                //panic!("dead loop");
                            }
                        },
                        Opcode::CALL => {
                            node_mut.value().jmp_stack.push(
                                pc + inst.opcode.increment_pc());
                            pc = inst.imm.unwrap() as usize;
                            node_mut.value().inst_count += 1;
                        },
                        Opcode::RET => {
                            pc = node_mut.value().jmp_stack.pop().unwrap();
                        },
                        _ => {
                            node_mut.value().inst_count += 1;
                        }
                    }
                    //println!("{:x?}", inst);
                    pc += inst.opcode.increment_pc();
                }
                node_mut.append(taken_node);
                node_mut.append(not_taken_node);
                if pc == f.1[0] {
                    panic!("END OF THE PROGRAM");
                }
            }
            copy_tree = path_tree.clone();
        }

        // Now we have enough paths
        let mut leaf_nodes = vec![];
        for mut c in path_tree.nodes() {
            if !c.has_children() {
                leaf_nodes.push(c);
            }
        }

        let mut path_vec: Vec<Vec<ExecutionNode>> = vec![];
        let mut write_vec: Vec<Vec<WriteInfo>> = vec![];

        for l in &leaf_nodes {
            let mut path = vec![];
            path.push(l.value().clone());
            for n in l.ancestors() {
                path.push(n.value().clone());
            }
            path_vec.push(path);
        }

        // Reconstruct the path and continue
        let mut inst_len = 0;
        for (id, mut p) in path_vec.iter_mut().enumerate() {
            let mut inst_vec: Vec<Inst> = vec![];
            //let mut path = path_vec.pop().unwrap();
            let mut jmp_stack = vec![];
            p.reverse();

            // Create the start of the path according to the nodes
            for mut n in p {
                // add all the path registers to the system
                if n.processed == true {
                    let mut pc = n.start.unwrap();
                    loop {
                        let mut inst: arch::avr::Inst = u32::from_le_bytes(
                            file[
                            ((pc - text_addr) + text_offset)..((pc - text_addr) +
                            text_offset + 4)].try_into()
                            .map_err(|_err| Error::InstReadError)?)
                            .into();
                        inst.pc = pc;
                        inst_vec.push(inst);
                        match inst.opcode {
                            Opcode::IN => {
                            },
                            Opcode::OUT => {
                            },
                            Opcode::JMP => {
                                pc = (inst.imm.unwrap() - inst.opcode.increment_pc()
                                    as i32) as usize;
                            },
                            Opcode::BRNE | Opcode::BREQ | Opcode::BRCS |
                                Opcode::BRTC | Opcode::BRCC | Opcode::BRPL |
                                Opcode::BRGE | Opcode::BRLT | Opcode::BRMI => {
                                    let prev_pc = pc - inst.opcode.increment_pc() as usize;
                                    pc = (((pc as i32 + ((1 + inst.imm.unwrap()) << 1))) -
                                        inst.opcode.increment_pc() as i32) as usize;
                                    // Avoid getting in dead loop FIXME: END?
                                    if prev_pc == pc {
                                        panic!("dead loop");
                                    }
                                    // We are done with the tree;
                                    //println!("{:x?}", inst);
                                    break;
                                },
                            Opcode::RJMP => {
                                let prev_pc = pc - inst.opcode.increment_pc() as usize;
                                pc = (((pc as i32 + ((1 + inst.imm.unwrap()) << 1))) -
                                    inst.opcode.increment_pc() as i32) as usize;
                                // Avoid getting inside a dead loop
                                // FIXME: END?
                                if prev_pc == pc {
                                    panic!("dead loop");
                                }
                            },
                            Opcode::CALL => {
                                jmp_stack.push(
                                    pc + inst.opcode.increment_pc());
                                pc = inst.imm.unwrap() as usize;
                            },
                            Opcode::RET => {
                                pc = jmp_stack.pop().unwrap();
                            },
                            _ => {},
                        }
                        if n.end.unwrap() == pc {
                            break;
                        }
                        pc += inst.opcode.increment_pc();
                    }
                }
            }

            let mut pc: usize = inst_vec.clone().pop().unwrap().pc;
            let mut rng = rand::thread_rng();

            // Now continue
            loop {
                //println!("{:x?}, {:x?}, {:x?}", pc, text_addr, text_offset);
                //println!("{:x?}",((pc - text_addr) + text_offset)..((pc - text_addr) +
                    //text_offset + 4));
                let mut inst: arch::avr::Inst = u32::from_le_bytes(
                    file[((pc - text_addr) + text_offset)..((pc - text_addr) +
                        text_offset + 4)]
                    .try_into().map_err(|_err| Error::InstReadError)?)
                    .into();
                inst.pc = pc;
                inst_vec.push(inst);
                match inst.opcode {
                    Opcode::IN => {
                    },
                    Opcode::OUT => {
                    },
                    Opcode::JMP => {
                        pc = (inst.imm.unwrap() - inst.opcode.increment_pc()
                            as i32) as usize;
                    },
                    Opcode::BRNE | Opcode::BREQ | Opcode::BRCS |
                        Opcode::BRTC | Opcode::BRCC | Opcode::BRPL |
                        Opcode::BRGE | Opcode::BRLT | Opcode::BRMI => {
                            if rng.gen_bool(0.6) {
                                pc = (((pc as i32 + ((1 + inst.imm.unwrap()) << 1))) -
                                    inst.opcode.increment_pc() as i32) as usize;
                            }
                            //let prev_pc = pc - inst.opcode.increment_pc() as usize;
                            //pc = (((pc as i32 + ((1 + inst.imm.unwrap()) << 1))) -
                            //inst.opcode.increment_pc() as i32) as usize;
                            // Avoid getting in dead loop FIXME: END?
                            //if prev_pc == pc {
                            //panic!("dead loop");
                            //}
                            // We are done with the tree;
                            //println!("{:x?}", inst);
                        },
                    Opcode::SBRS | Opcode::SBRC | Opcode::SBIS |
                        Opcode::CPSE => {
                        if rng.gen_bool(0.3) {
                            let mut next_inst: arch::avr::Inst = u32::from_le_bytes(
                                file[((pc - text_addr + inst.opcode.increment_pc()) +
                                    text_offset)..((pc - text_addr + inst.opcode.increment_pc()) +
                                    text_offset + 4)]
                                .try_into().map_err(|_err| Error::InstReadError)?)
                                .into();
                            pc += inst.opcode.increment_pc() +
                                next_inst.opcode.increment_pc();
                        }
                    },
                    Opcode::RJMP => {
                        let prev_pc = pc - inst.opcode.increment_pc() as usize;
                        pc = (((pc as i32 + ((1 + inst.imm.unwrap()) << 1))) -
                            inst.opcode.increment_pc() as i32) as usize;
                        // Avoid getting inside a dead loop
                        // FIXME: END?
                        if prev_pc == pc {
                            //panic!("dead loop");
                            //break;
                        }
                    },
                    Opcode::IJMP => {
                        // Bypass the println
                        if (pc == 0x660) {
                            pc = jmp_stack.pop().unwrap() - inst.opcode.increment_pc();
                        }
                    }
                    Opcode::CALL => {
                        //if inst.pc == 0x59d8 || inst.pc == 0x3fcc || inst.pc == 0x3c0 {
                        if inst.pc == 0x3c0 {
                            pc += inst.opcode.increment_pc();
                            continue
                        }
                        jmp_stack.push(
                            pc + inst.opcode.increment_pc());
                        pc = inst.imm.unwrap() as usize;
                        continue
                    },
                    Opcode::RCALL => {
                        jmp_stack.push(
                            pc + inst.opcode.increment_pc());
                        let prev_pc = pc - inst.opcode.increment_pc() as usize;
                        pc = (((pc as i32 + ((1 + inst.imm.unwrap()) << 1))) -
                            inst.opcode.increment_pc() as i32) as usize;
                        // Avoid getting inside a dead loop
                        // FIXME: END?
                        if prev_pc == pc {
                            panic!("dead loop");
                        }
                    },
                    //Opcode::IJMP => {
                        //pc = jmp_stack.pop().unwrap() - inst.opcode.increment_pc();
                    //},
                    Opcode::RET => {
                        pc = jmp_stack.pop().unwrap() - inst.opcode.increment_pc();
                        //println!("{:x?}", inst);
                        //continue;
                    },
                    _ => {
                    }
                }
                println!("{:x?}", inst);
                pc += inst.opcode.increment_pc();
                //if (pc <= 0x4f6a) && (pc >= 0x4f5a) {
                if (pc == f.1[0]) || (pc == f.1[1]) {
                //if (pc <= 0xa04e) && (pc >= 0xa05a) {
                    break
                }
                //}
            }
            
            let mut temp_write = vec![];
            for i in 1..=100 {
                let lndx = (i - 1) * (inst_vec.len() / 100);
                let mut ndx = i * (inst_vec.len() / 100);
                //println!("{:?}", inst_vec[lndx..ndx].len());
                //println!("{:?}", inst_vec.len());
                let mut io_count = 0;
                if (i == 100) {
                    ndx = inst_vec.len()
                }
                for (id, inst) in inst_vec[lndx..ndx].iter().enumerate() {
                    match inst.opcode {
                        Opcode::IN | Opcode::OUT => {
                            io_count += 1;
                        }
                        _ => {},
                    }
                }
                temp_write.push(WriteInfo::new((inst_vec[lndx..ndx].len() as f64), (io_count as f64), i));
            }
            io_count = 0;
            for i in &inst_vec {
                match i.opcode {
                    Opcode::IN | Opcode::OUT => {
                        io_count += 1;
                    }
                    _ => {},
                }
            }
            temp_write.push(WriteInfo::new((inst_vec.len() as f64), (io_count as f64), 101));

            write_vec.push(temp_write);
            temp_write = vec![];
            inst_len = inst_vec.len();
        }
        let file = std::fs::File::create(format!("results/{}_path.csv", f.2)).expect("failed");
        let mut file = LineWriter::new(file);
        file.write_all(format!("Percentage, Path 1, Path 2, Path 3, Path 4, Path 5, Path 6, Path 7, Path 8, Path 9, Path 10, Path 11, Path 12, Path 13, Path 14, Path 15, Path 16\n")
            .as_bytes()).expect("failed to write the csv header");
        for i in 0..=99 {
            file.write_all(format!("{}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?},\n", (i+1),
                    (write_vec[0][i].io_count / write_vec[0][100].io_count), (write_vec[1][i].io_count / write_vec[1][100].io_count), (write_vec[2][i].io_count / write_vec[2][100].io_count), (write_vec[3][i].io_count / write_vec[3][100].io_count),
                    (write_vec[4][i].io_count / write_vec[4][100].io_count), (write_vec[5][i].io_count / write_vec[5][100].io_count), (write_vec[6][i].io_count / write_vec[6][100].io_count), (write_vec[7][i].io_count / write_vec[7][100].io_count),
                    (write_vec[8][i].io_count / write_vec[8][100].io_count), (write_vec[9][i].io_count / write_vec[9][100].io_count), (write_vec[10][i].io_count / write_vec[10][100].io_count), (write_vec[11][i].io_count / write_vec[11][100].io_count),
                    (write_vec[12][i].io_count / write_vec[12][100].io_count), (write_vec[13][i].io_count / write_vec[13][100].io_count), (write_vec[14][i].io_count / write_vec[14][100].io_count), (write_vec[15][i].io_count / write_vec[15][100].io_count))
                .as_bytes()).expect("failed to write row");
        }
        file.write_all(format!("101, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {},\n",
                write_vec[0][100].inst_count, write_vec[1][100].inst_count, write_vec[2][100].inst_count, write_vec[3][100].inst_count,
                write_vec[4][100].inst_count, write_vec[5][100].inst_count, write_vec[6][100].inst_count, write_vec[7][100].inst_count,
                write_vec[8][100].inst_count, write_vec[9][100].inst_count, write_vec[10][100].inst_count, write_vec[11][100].inst_count,
                write_vec[12][100].inst_count, write_vec[13][100].inst_count, write_vec[14][100].inst_count, write_vec[15][100].inst_count).as_bytes())
            .expect("failed to write the density");
        file.write_all(format!("102, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {},\n",
                write_vec[0][100].io_count, write_vec[1][100].io_count, write_vec[2][100].io_count, write_vec[3][100].io_count,
                write_vec[4][100].io_count, write_vec[5][100].io_count, write_vec[6][100].io_count, write_vec[7][100].io_count,
                write_vec[8][100].io_count, write_vec[9][100].io_count, write_vec[10][100].io_count, write_vec[11][100].io_count,
                write_vec[12][100].io_count, write_vec[13][100].io_count, write_vec[14][100].io_count, write_vec[15][100].io_count).as_bytes())
            .expect("failed to write the density");
    }


    //for (i, el) in percent.iter().enumerate() {
        //println!("Percentage {:?}%:    {:?}", i+1, el / projects.len() as f64);
    //}
    Ok(())
}
