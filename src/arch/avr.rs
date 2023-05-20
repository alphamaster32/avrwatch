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
    JMP,
    NOP,
    SBC,
    MULS,
    WORD,
    MULSU,
    CPC,
    AND,
    FMULSU,
    LD,
    EOR,
    OUT,
    LDI,
    RJMP,
    LPM,
    ST,
    CPI,
    BRNE,
    SBIW,
    MOVW,
    PUSH,
    ADD,
    ADC,
    SUB,
    CP,
    BREQ,
    MOV,
    POP,
    RET,
    LDD,
    STS,
    BRCS,
    ADIW,
    SUBI,
    ANDI,
    STD,
    SBCI,
    OR,
    CPSE,
    ORI,
    SBRS,
    IN,
    SBRC,
    LDS,
    STS16,
    BRCC,
    SBIS,
    MUL,
    RCALL,
    BRLT,
    BST,
    CLI,
    IJMP,
    SEI,
    ICALL,
    DEC,
    NEG,
    BRTC,
    BRPL,
    BRGE,
    COM,
    ASR,
    ROR,
    INC,
    SET,
    LSR,
    RETI,
    CALL,
    CLT,
    CLV,
    SWAP,
    BRMI,
    BLD,
}

impl Opcode {
    pub fn increment_pc(self) -> usize {
        match self {
            Opcode::JMP | Opcode::STS | Opcode::LDS | Opcode::CALL => 4,
            Opcode::NOP | Opcode::SBC | Opcode::MULS | Opcode::WORD |
                Opcode::MULSU | Opcode::CPC | Opcode::AND |
                Opcode::FMULSU | Opcode::LD | Opcode::EOR |
                Opcode::OUT | Opcode::LDI | Opcode::RJMP |
                Opcode::LPM | Opcode::ST | Opcode::CPI |
                Opcode::BRNE | Opcode::SBIW | Opcode::MOVW |
                Opcode::PUSH | Opcode::ADD | Opcode::ADC |
                Opcode::SUB | Opcode::CP | Opcode::BREQ |
                Opcode::MOV | Opcode::POP | Opcode::RET |
                Opcode::LDD | Opcode::BRCS | Opcode::ADIW |
                Opcode::SUBI | Opcode::ANDI | Opcode::STD |
                Opcode::SBCI | Opcode::OR | Opcode::CPSE |
                Opcode::ORI | Opcode::SBRS | Opcode::IN |
                Opcode::SBRC | Opcode::STS16 | Opcode::BRCC |
                Opcode::SBIS | Opcode::MUL | Opcode::RCALL |
                Opcode::BRLT | Opcode::BST | Opcode::CLI |
                Opcode::IJMP | Opcode::SEI | Opcode::ICALL |
                Opcode::DEC | Opcode::NEG | Opcode::BRTC |
                Opcode::BRPL | Opcode::BRGE | Opcode::COM |
                Opcode::ASR | Opcode::ROR | Opcode::INC |
                Opcode::SET | Opcode::LSR | Opcode::RETI |
                Opcode::CLT | Opcode::CLV | Opcode::SWAP |
                Opcode::BRMI | Opcode::BLD |
                Opcode::None => 2,
            _ => panic!("dont know how much to increment: {:?}", self),
        }
    }
}

impl From<u32> for Opcode {
    fn from(val: u32) -> Self {
        // Divide the instruction
        let val_arr = val.to_le_bytes();
        let inst: u16 = u16::from_le_bytes(val_arr[..=1]
            .try_into().unwrap());
        // We may not need the data according to the opcode
        let data: u16 = u16::from_le_bytes(val_arr[2..]
            .try_into().unwrap());
        let word_data: u8 = val_arr[1];

        // Divide the prefix
        let prefix9 = inst >> (16 - 9);
        let prefix8 = inst >> (16 - 8);
        let prefix7 = inst >> (16 - 7);
        let prefix6 = inst >> (16 - 6);
        let prefix5 = inst >> (16 - 5);
        let prefix4 = inst >> (16 - 4);

        let suffix30 = inst & 0b1111;
        let suffix20 = inst & 0b111;

        match inst {
            0b0 => {
                return Opcode::NOP;
            },
            0b1001_0101_1100_1000 => {
                return Opcode::LPM;
            },
            0b1001_0101_0000_1000 => {
                return Opcode::RET;
            },
            0b1001_0100_1111_1000 => {
                return Opcode::CLI;
            },
            0b1001_0100_0000_1001 => {
                return Opcode::IJMP;
            },
            0b1001_0100_0111_1000 => {
                return Opcode::SEI;
            },
            0b1001_0101_0000_1001 => {
                return Opcode::ICALL;
            },
            0b1001_0100_0110_1000 => {
                return Opcode::SET;
            },
            0b1001_0101_0001_1000 => {
                return Opcode::RETI;
            }
            0b1001_0100_1110_1000 => {
                return Opcode::CLT;
            },
            0b1001_0100_1011_1000 => {
                return Opcode::CLV;
            },
            _ => {},
        }

        match prefix4 {
            0b1110 => {
                return Opcode::LDI;
            },
            0b1100 => {
                return Opcode::RJMP
            },
            0b011 => {
                return Opcode::CPI;
            },
            0b0101 => {
                return Opcode::SUBI;
            },
            0b0111 => {
                return Opcode::ANDI;
            },
            0b0100 => {
                return Opcode::SBCI;
            },
            0b0110 => {
                return Opcode::ORI;
            },
            0b1101 => {
                return Opcode::RCALL;
            }
            _ => {},
        }

        match prefix5 {
            0b1011_1 => {
                return Opcode::OUT;
            },
            0b1011_0 => {
                return Opcode::IN;
            },
            0b1010_1 => {
                return Opcode::STS16;
            },
            _ => {},
        }

        match (prefix6, suffix20) {
            (0b0000_10, _) => {
                return Opcode::SBC;
            },
            (0b0000_01, _) => {
                return Opcode::CPC;
            },
            (0b0010_00, _) => {
                return Opcode::AND;
            },
            (0b0010_01, _) => {
                return Opcode::EOR
            },
            (0b1111_01, 0b001) => {
                return Opcode::BRNE;
            },
            (0b0000_11, _) => {
                return Opcode::ADD;
            },
            (0b0001_11, _) => {
                return Opcode::ADC;
            },
            (0b0001_10, _) => {
                return Opcode::SUB;
            },
            (0b0001_01, _) => {
                return Opcode::CP;
            },
            (0b1111_00, 0b001) => {
                return Opcode::BREQ;
            },
            (0b0010_11, _) => {
                return Opcode::MOV;
            },
            (0b1111_00, 0b000) => {
                return Opcode::BRCS;
            },
            (0b0010_10, _) => {
                return Opcode::OR;
            },
            (0b0001_00, _) => {
                return Opcode::CPSE;
            },
            (0b1111_01, 0b110) => {
                return Opcode::BRTC;
            }
            (0b1111_01, 0b000) => {
                return Opcode::BRCC;
            },
            (0b1111_01, 0b010) => {
                return Opcode::BRPL;
            },
            (0b1111_01, 0b100) => {
                return Opcode::BRGE;
            },
            (0b1001_11, _) => {
                return Opcode::MUL;
            },
            (0b1111_00, 0b100) => {
                return Opcode::BRLT;
            },
            (0b1111_00, 0b010) => {
                return Opcode::BRMI;
            },
            _ => {},
        }

        // FIXME: now that we get suffix30 clean it up
        match (prefix7, suffix30) {
            (0b1001_010, 0b1010) => {
                return Opcode::DEC;
            },
            (0b1001_010, 0b0001) => {
                return Opcode::NEG;
            },
            (0b1001_010, 0b0000) => {
                return Opcode::COM;
            },
            (0b1001_010, 0b0101) => {
                return Opcode::ASR;
            },
            (0b1001_010, 0b0111) => {
                return Opcode::ROR;
            },
            (0b1001_010, 0b0011) => {
                return Opcode::INC;
            },
            (0b1001_010, 0b0110) => {
                return Opcode::LSR;
            },
            (0b1001_010, 0b0010) => {
                return Opcode::SWAP;
            },
            (0b1001_010, _) => {
                match (suffix30 & 0b1110) {
                    0b1100 => {
                        return Opcode::JMP;
                    },
                    0b1110 => {
                        return Opcode::CALL
                    },
                    _ => {},
                }
            },
            (0b1001_000, 0b1111) => {
                return Opcode::POP;
            },
            (0b1001_000, 0b0000) => {
                return Opcode::LDS;
            },
            (0b1001_000, 0b0100) | (0b1001_000, 0b101)
                => {
                return Opcode::LPM;
            },
            (0b1001_001, 0b0000) => {
                return Opcode::STS;
            },
            (0b1000_000, 0b0000) | (0b1001_000, 0b0001) |
                (0b1001_000, 0b0010) | (0b1001_000, 0b1100) |
                (0b1001_000, 0b1101) | (0b1001_000, 0b1110) |
                (0b1000_000, 0b1000) | (0b1001_000, 0b1001) |
                (0b1001_000, 0b1010) => {
                return Opcode::LD;
            },
            (0b1001_001, 0b1100) | (0b1001_001, 0b1101) |
                (0b1001_001, 0b1110) | (0b1000_001, 0b1000) |
                (0b1001_001, 0b1001) | (0b1001_001, 0b1010) |
                (0b1000_001, 0b0000) | (0b1001_001, 0b0001) |
                (0b1001_001, 0b0010) => {
                return Opcode::ST;
            },
            (0b1111_111, _) => {
                return Opcode::SBRS;
            },
            (0b1111_110, _) => {
                return Opcode::SBRC;
            },
            (0b1001_001, 0b1111) => {
                return Opcode::PUSH;
            },
            (0b1111_101, _) => {
                return Opcode::BST;
            },
            (0b1111_100, _) => {
                return Opcode::BLD;
            },
            _ => {},
        }

        match prefix8 {
            0b0000_0010 => {
                return Opcode::MULS;
            },
            0b1001_0111 => {
                return Opcode::SBIW;
            },
            0b0000_0001 => {
                return Opcode::MOVW;
            },
            0b1001_0110 => {
                return Opcode::ADIW;
            },
            0b1001_1011 => {
                return Opcode::SBIS;
            },
            _ => {},
        }

        match prefix9 {
            0b0000_0011_0 => {
                return Opcode::MULSU;
            },
            0b0000_0011_1 => {
                return Opcode::FMULSU;
            },
            _ => {},
        }

        match word_data {
            0b0 => {
                return Opcode::WORD;
            },
            _ => {}
        }

        // Some obsecure ldd
        match inst & 0b1101_0010_0000_1000 {
            0b1000_0000_0000_0000 | 0b1000_0000_0000_1000 => {
                return Opcode::LDD;
            },
            0b1000_0010_0000_1000 | 0b1000_0010_0000_0000 => {
                return Opcode::STD;
            },
            _ => {},
        }

        //unimplemented!("can't understand inst: 0b{:016b}, data: 0b{:016b}",
            //inst, data);
        return Opcode::None;
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
    pub imm: Option<i32>,
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
                inst.imm = Some(((half2 as u32 |
                        (((imm84 << 1) & imm0) as u32) << 16) << 1) as i32);
            },
            Opcode::NOP => {
                // NOP :^)
            },
            Opcode::SBC => {
                let dest84 = (half1 << 7) >> 11;
                let source10 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source10 << 4)) as u8).into());
            },
            Opcode::MULS => {
                let dest74 = (half1 << 8) >> 12;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some(((dest74 as u8) + 16).into());
                inst.rr = Some(((source30 as u8) + 16).into());
            },
            Opcode::WORD => {
                // NOP :^)
            },
            Opcode::MULSU => {
                let dest64 = (half1 << 9) >> 13;
                let source20 = (half1 << 13) >> 13;
                inst.rd = Some(((dest64 as u8) + 16).into());
                inst.rr = Some(((source20 as u8) + 16).into());
            },
            Opcode::CPC => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::AND => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::FMULSU => {
                let dest64 = (half1 << 9) >> 13;
                let source20 = (half1 << 13) >> 13;
                inst.rd = Some(((dest64 as u8) + 16).into());
                inst.rr = Some(((source20 as u8) + 16).into());
            },
            Opcode::LD => {
                // This instruction has a indirect address mode but
                // for the simplicity we would not parse and provide
                // information for that
                // FIXME: May bite us back :^)
                let prefix7 = half1 >> (16 - 7);
                let suffix30 = half1 & 0b1111;
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some(((dest84 as u8)).into());
                match (prefix7, suffix30) {
                    (0b1001_000, 0b1100) | (0b1001_000, 0b1101) |
                        (0b1001_000, 0b1110) => {
                        inst.rr = Some(32_u8.into());
                    },
                    (0b1000_000, 0b1000) | (0b1001_000, 0b1001) |
                        (0b1001_000, 0b1010) => {
                        inst.rr = Some(33_u8.into());
                    },
                    (0b1000_000, 0b0000) | (0b1001_000, 0b0001) |
                        (0b1001_000, 0b0010) => {
                        inst.rr = Some(34_u8.into());
                    },
                    (0b1001_000, 0b0100) | (0b1001_000, 0b0101) => {
                        inst.opcode = Opcode::LPM;
                        inst.rr = Some(34_u8.into());
                    }
                    _ => {
                        unimplemented!("Undefined LD match: {:#b}", half1);
                    },
                }
            },
            Opcode::EOR => {
                let source30 = (half1 << 12) >> 12;
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());

            },
            Opcode::OUT => {
                // FIXME: This is the port number we should process it with
                // another way
                let dest30 = (half1 << 12) >> 12;
                let dest109 = (half1 & 0b0000_0110_0000_0000) >> 9;
                let source84= (half1 << 7) >> 11;

                inst.rr = Some((source84 as u8).into());
                //inst.rd = Some(((dest30 | (dest109 << 4)) as u8).into());
            },
            Opcode::LDI => {
                let dest74 = (half1 << 8) >> 12;
                let imm30 = (half1 << 12) >> 12;
                let imm118 = (half1 << 4) >> 12;
                inst.imm = Some((imm30 | (imm118 << 4)) as i32);
                inst.rd = Some((dest74 as u8 + 16).into());
            },
            Opcode::RJMP => {
                inst.imm = Some((((half1 << 4) as i16) >> 4) as i32);
            },
            Opcode::ST => {
                // This instruction has a indirect address mode but
                // for the simplicity we would not parse and provide
                // information for that
                // FIXME: May bite us back :^)
                let prefix7 = half1 >> (16 - 7);
                let suffix30 = half1 & 0b1111;
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some(((dest84 as u8)).into());
                match (prefix7, suffix30) {
                    (0b1001_001, 0b1100) | (0b1001_001, 0b1101) |
                        (0b1001_001, 0b1110) => {
                        inst.rr = Some(32_u8.into());
                    },
                    (0b1000_001, 0b1000) | (0b1001_001, 0b1001) |
                        (0b1001_001, 0b1010) => {
                        inst.rr = Some(33_u8.into());
                    },
                    (0b1000_001, 0b0000) | (0b1001_001, 0b0001) |
                        (0b1001_001, 0b0010) => {
                        inst.rr = Some(34_u8.into());
                    },
                    (0b1001_001, 0b1111) => {
                        inst.rr = Some((dest84 as u8).into());
                    },
                    _ => {
                        unimplemented!("Undefined ST match: {:#b}", half1);
                    },
                }
            },
            Opcode::CPI => {
                let dest74 = (half1 << 8) >> 12;
                let imm30 = (half1 << 12) >> 12;
                let imm118 = (half1 << 4) >> 12;
                inst.imm = Some((imm30 | (imm118 << 4)) as i32);
                inst.rd = Some((dest74 as u8 + 16).into());
            },
            Opcode::BRNE => {
                inst.imm = Some(((half1 << 6) >> 9) as i32);
            },
            Opcode::SBIW => {
                let imm30 = (half1 << 12) >> 12;
                let imm76 = (half1 << 8) >> 14;
                inst.imm = Some((imm30 | (imm76 << 4)) as i32);
                inst.rd = Some((((((half1 & 0b0000_0000_0011_0000) + 1) >> 4)
                            * 2 + 24) as u8).into());

            },
            Opcode::MOVW => {
                let dest74 = (half1 << 8) >> 12;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some(((dest74 * 2) as u8).into());
                inst.rr = Some(((source30 * 2) as u8).into());
            },
            Opcode::ADD => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::ADC => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::SUB => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::CP => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::BREQ => {
                inst.imm = Some(((half1 << 6) >> 9) as i32);
            },
            Opcode::MOV => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::POP => {
                inst.rd = Some((((half1 << 7) >> 11) as u8).into());
            },
            Opcode::RET => {
                // NOP :^)
            },
            Opcode::LDD => {
                // FIXME: add the immeadiate
                let dest84 = (half1 << 7) >> 11;
                inst.rr = Some((dest84 as u8).into());
            },
            Opcode::BRCS => {
                inst.imm = Some(((half1 << 6) >> 9) as i32);
            },
            Opcode::ADIW => {
                let imm30 = (half1 << 12) >> 12;
                let imm76 = (half1 << 8) >> 14;
                inst.imm = Some((imm30 | (imm76 << 4)) as i32);
                inst.rd = Some((((((half1 & 0b0000_0000_0011_0000) + 1) >> 4)
                            * 2 + 24) as u8).into());
            },
            Opcode::SUBI => {
                let dest74 = (half1 << 8) >> 12;
                let imm30 = (half1 << 12) >> 12;
                let imm118 = (half1 << 4) >> 12;
                inst.imm = Some((imm30 | (imm118 << 4)) as i32);
                inst.rd = Some((dest74 as u8 + 16).into());
            },
            Opcode::ANDI => {
                let dest74 = (half1 << 8) >> 12;
                let imm30 = (half1 << 12) >> 12;
                let imm118 = (half1 << 4) >> 12;
                inst.imm = Some((imm30 | (imm118 << 4)) as i32);
                inst.rd = Some((dest74 as u8 + 16).into());
            },
            Opcode::STD => {
                // FIXME: add the immeadiate
                let dest84 = (half1 << 7) >> 11;
                inst.rr = Some((dest84 as u8).into());
            },
            Opcode::SBCI => {
                let dest74 = (half1 << 8) >> 12;
                let imm30 = (half1 << 12) >> 12;
                let imm118 = (half1 << 4) >> 12;
                inst.imm = Some((imm30 | (imm118 << 4)) as i32);
                inst.rd = Some((dest74 as u8 + 16).into());
            },
            Opcode::OR => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::CPSE => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::ORI => {
                let dest74 = (half1 << 8) >> 12;
                let imm30 = (half1 << 12) >> 12;
                let imm118 = (half1 << 4) >> 12;
                inst.imm = Some((imm30 | (imm118 << 4)) as i32);
                inst.rd = Some((dest74 as u8 + 16).into());
            },
            Opcode::SBRS => {
                inst.imm = Some(((half1 & 0b111) as u8).into());
                inst.rr = Some((((half1 << 7) >> 11) as u8).into());
            },
            Opcode::IN => {
                // FIXME: This is the port number we should process it with
                // another way
                let imm30 = (half1 << 12) >> 12;
                let imm109 = (half1 & 0b0000_0110_0000_0000) >> 9;
                let dest84 = (half1 << 7) >> 11;
                inst.rr = Some((dest84 as u8).into());
                inst.imm = Some(((imm30 | (imm109 << 4))) as i32);
            },
            Opcode::SBRC => {
                inst.imm = Some(((half1 & 0b111) as u8).into());
                inst.rr = Some((((half1 << 7) >> 11) as u8).into());
            },
            Opcode::LPM => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some(((dest84 as u8)).into());
                inst.rr = Some(34_u8.into());
            },
            Opcode::LDS => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some(((dest84 as u8)).into());
                inst.imm  = Some(half2 as i32);
            },
            Opcode::STS => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some(((dest84 as u8)).into());
                inst.imm  = Some(half2 as i32);
            },
            Opcode::BRCC => {
                inst.imm = Some(((half1 << 6) >> 9) as i32);
            },
            Opcode::SBIS => {
                // FIXME: has too many register bits needs inst refactor
            },
            Opcode::MUL => {
                let dest84 = (half1 << 7) >> 11;
                let source9 = (half1 & 0b0000_0010_0000_0000) >> 9;
                let source30 = (half1 << 12) >> 12;
                inst.rd = Some((dest84 as u8).into());
                inst.rr = Some(((source30 | (source9 << 4)) as u8).into());
            },
            Opcode::PUSH => {
                inst.rd = Some((((half1 << 7) >> 11) as u8).into());
            },
            Opcode::RCALL => {
                inst.imm = Some((half1 & 0b0000_1111_1111_1111) as i32);
            },
            Opcode::BRLT => {
                inst.imm = Some(((half1 & 0b0000_0011_1111_1000) >> 3) as i32);
            },
            Opcode::BST => {
                let dest84 = (half1 << 7) >> 11;
                let imm20 = (half1 << 13) >> 13;
                inst.rd = Some((dest84 as u8).into());
                inst.imm = Some(imm20 as i32);
            },
            Opcode::CLI => {},
            Opcode::IJMP => {},
            Opcode::SEI => {},
            Opcode::ICALL => {},
            Opcode::DEC => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some((dest84 as u8).into());
            },
            Opcode::NEG => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some((dest84 as u8).into());
            },
            Opcode::BRTC => {
                inst.imm = Some(((half1 & 0b0000_0011_1111_1000) >> 3) as i32);
            },
            Opcode::BRPL => {
                inst.imm = Some(((half1 & 0b0000_0011_1111_1000) >> 3) as i32);
            },
            Opcode::BRGE => {
                inst.imm = Some(((half1 & 0b0000_0011_1111_1000) >> 3) as i32);
            },
            Opcode::COM => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some((dest84 as u8).into());
            },
            Opcode::ASR => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some((dest84 as u8).into());
            },
            Opcode::ROR => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some((dest84 as u8).into());
            },
            Opcode::INC => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some((dest84 as u8).into());
            },
            Opcode::SET => {},
            Opcode::LSR => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some((dest84 as u8).into());
            },
            Opcode::RETI => {},
            Opcode::CALL => {
                let imm84 = (half1 << 7) >> 11;
                let imm0 = half1 & 0x01;
                // This is word address and we need byte address
                inst.imm = Some(((half2 as u32 |
                        (((imm84 << 1) & imm0) as u32) << 16) << 1) as i32);
            },
            Opcode::CLT => {},
            Opcode::CLV => {},
            Opcode::STS16 => {
                let dest74 = (half1 << 8) >> 12;
                let imm30 = (half1 << 12) >> 12;
                let imm108 = (half1 & 0b0000_0111_0000_0000);
                inst.imm = Some((imm30 | (imm108 >> 4)) as i32);
                inst.rd = Some((dest74 as u8).into());
            },
            Opcode::SWAP => {
                let dest84 = (half1 << 7) >> 11;
                inst.rd = Some((dest84 as u8).into());
            },
            Opcode::BRMI => {
                inst.imm = Some(((half1 & 0b0000_0011_1111_1000) >> 3) as i32);
            },
            Opcode::BLD => {
                let dest84 = (half1 << 7) >> 11;
                let imm20 = (half1 << 13) >> 13;
                inst.rd = Some((dest84 as u8).into());
                inst.imm = Some(imm20 as i32);
            },
            _ => {
                //unimplemented!("don't know how to process instruction: {:?}",
                    //inst.opcode);
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
