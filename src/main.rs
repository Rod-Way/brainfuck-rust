use std::{
    env,
    fs::File,
    io::{stdin, Read},
    process::exit,
};

const OP_END: u16 = 0;
const OP_INC_DP: u16 = 1;
const OP_DEC_DP: u16 = 2;
const OP_INC_VAL: u16 = 3;
const OP_DEC_VAL: u16 = 4;
const OP_OUT: u16 = 5;
const OP_IN: u16 = 6;
const OP_JMP_FWD: u16 = 7;
const OP_JMP_BCK: u16 = 8;

const SUCCESS: i32 = 0;
const FAILURE: i32 = 1;

const PROGRAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 512;
const DATA_SIZE: usize = 65535;

fn stack_push(i: u16) {
    unsafe {
        STACK[SP] = i;
        SP += 1;
    }
}

fn stack_pop() -> u16 {
    unsafe {
        SP -= 1;
        STACK[SP]
    }
}

fn stack_empty() -> bool {
    unsafe {
        return SP == 0;
    }
}

fn stack_full() -> bool {
    unsafe { SP == STACK_SIZE }
}

#[derive(Copy, Clone)]
struct Instruction {
    operator: u16,
    operand: u16,
}

static mut PROGRAM: [Instruction; PROGRAM_SIZE] = [Instruction {
    operator: 0,
    operand: 0,
}; PROGRAM_SIZE];
static mut STACK: [u16; STACK_SIZE] = [0; STACK_SIZE];
static mut SP: usize = 0;

fn compile_bf(mut file: File) -> i32 {
    let mut pc: usize = 0;
    let mut jmp_pc: u16;
    let mut c = [0; 1];

    while let Ok(n) = file.read(&mut c) {
        if n == 0 || pc > PROGRAM_SIZE {
            break;
        }
        match c[0] as char {
            '>' => unsafe {
                PROGRAM[pc].operator = OP_INC_DP;
            },
            '<' => unsafe {
                PROGRAM[pc].operator = OP_DEC_DP;
            },
            '+' => unsafe {
                PROGRAM[pc].operator = OP_INC_VAL;
            },
            '-' => unsafe {
                PROGRAM[pc].operator = OP_DEC_VAL;
            },
            '.' => unsafe {
                PROGRAM[pc].operator = OP_OUT;
            },
            ',' => unsafe {
                PROGRAM[pc].operator = OP_IN;
            },
            '[' => unsafe {
                PROGRAM[pc].operator = OP_JMP_FWD;
                if stack_full() {
                    return FAILURE;
                }
                stack_push(pc as u16);
            },
            ']' => unsafe {
                if stack_empty() {
                    return FAILURE;
                }
                jmp_pc = stack_pop();
                PROGRAM[pc].operator = OP_JMP_BCK;
                PROGRAM[pc].operand = jmp_pc;
                PROGRAM[jmp_pc as usize].operand = pc as u16;
            },
            _default => {
                pc -= 1;
            }
        };
        pc += 1;
    }
    if !stack_empty() || pc == PROGRAM_SIZE {
        return FAILURE;
    }
    unsafe {
        PROGRAM[pc].operator = OP_END;
    }
    return SUCCESS;
}

fn execute_bf() -> i32 {
    let mut data = [0u8; DATA_SIZE];
    let mut pc: usize = 0;
    let mut ptr: usize = DATA_SIZE;

    while ptr > 0 {
        ptr -= 1;
        data[ptr] = 0;
    }

    while unsafe { PROGRAM[pc].operator != OP_END } && ptr < DATA_SIZE {
        match unsafe { PROGRAM[pc].operator } {
            OP_INC_DP => ptr += 1,
            OP_DEC_DP => ptr -= 1,
            OP_INC_VAL => data[ptr] += 1,
            OP_DEC_VAL => data[ptr] -= 1,
            OP_OUT => print!("{}", data[ptr] as char),
            OP_IN => {
                let mut input = [0; 1];
                match stdin().read_exact(&mut input) {
                    Ok(_) => {}
                    Err(_) => {
                        return FAILURE;
                    }
                };
                data[ptr] = input[0];
            }
            OP_JMP_FWD => {
                if data[ptr] == 0 {
                    pc = unsafe { PROGRAM[pc].operand as usize };
                }
            }
            OP_JMP_BCK => {
                if data[ptr] != 0 {
                    pc = unsafe { PROGRAM[pc].operand as usize };
                }
            }
            _default => return FAILURE,
        }
        pc += 1;
    }

    if ptr != DATA_SIZE {
        return SUCCESS;
    } else {
        return FAILURE;
    }
}

fn main() {
    let mut status: i32;

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprint!("Error:\n\tNeed args: 2\n\tGot: {}\n", args.len());
        exit(FAILURE);
    }

    let file: File = match File::open(&args[1]) {
        Ok(f) => f,
        Err(_) => {
            eprint!("Error:\n\tFailed to open file: {}\n", args[1]);
            exit(FAILURE);
        }
    };
    status = compile_bf(file);

    if status == SUCCESS {
        status = execute_bf();
    }
    if status == FAILURE {
        eprint!("Error: Can not compile\n",);
        exit(FAILURE);
    }

    eprint!("Status code: {}\n", status);
}
