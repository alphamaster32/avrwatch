/// Enum representing the registers available in AVR microcontrollers
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    R16,
    R17,
    R18,
    R19,
    R20,
    R21,
    R22,
    R23,
    R24,
    R25,
    R26,
    R27,
    R28,
    R29,
    R30,
    R31,
    X,
    Y,
    Z
}


impl From<u8> for Register {
    fn from(val: u8) -> Self {
        assert!(val < 0xFF);
        unsafe {
            // Avoid lenghty match statement
            core::ptr::read_unaligned(&(val as usize) as
                                      *const usize as *const Register)
        }
    }
}

/// List of the opcodes that are implemented in the AVR ISA
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Opcode {
    None,
    ADD,
    JMP,
}

impl Opcode {
    pub fn increment_pc(self) -> usize {
        match self {
            Opcode::JMP => 4,
            _ => panic!("dont know how much to increment: {:?}", self),
        }
    }
}

impl From<u32> for Opcode {
    fn from(val: u32) -> Self {
        // Divide the instruction
        let val = val.to_le_bytes();
        let inst: u16 = u16::from_le_bytes(val[..2]
            .try_into().unwrap());
        // We may not need the data according to the opcode
        let data: u16 = u16::from_le_bytes(val[2..]
            .try_into().unwrap());

        // Divide the prefix
        let prefix7 = inst >> (16 - 7);
        match prefix7 {
            0b1001010 => {
                Opcode::JMP
            },
            _ => {
                unimplemented!("unimplemented opcode {:#b}", prefix7);
            }
        }
    }
}

/// Struct representing an instruction
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Inst {
    /// Opcode of the `Inst`
    pub opcode: Opcode,
    /// Destination register
    pub rd: Option<Register>,
    /// Source register
    pub rr: Option<Register>,
    /// Immediate value
    pub imm: Option<u32>,
    /// Address of the instruction in the elf file or program counter 
    pub pc: usize,
}
impl From<u32> for Inst {
    fn from(val: u32) -> Self {
        let mut inst: Inst = Inst::new();
        inst.opcode = val.into();

        let val = val.to_le_bytes();
        let half1: u16 = u16::from_le_bytes(val[..2]
            .try_into().unwrap());
        let half2: u16 = u16::from_le_bytes(val[2..]
            .try_into().unwrap());
        match inst.opcode {
            Opcode::JMP => {
                let imm84 = (half1 << 7) >> 11;
                let imm0 = half1 & 0x01;
                // This is word address and we need byte address
                inst.imm = Some((half2 as u32 |
                        (((imm84 << 1) & imm0) as u32) << 16) << 1);
            },
            _ => {
                unimplemented!("don't know how to process instruction: {:?}",
                    inst.opcode);
            }
        }

        inst
    }
}

impl Inst {
    pub fn new() -> Self {
        Inst {
            opcode: Opcode::None,
            rd: None,
            rr: None,
            imm: None,
            pc: 0,
        }
    }
}
