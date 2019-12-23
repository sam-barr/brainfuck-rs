use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::num::Wrapping;

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum Token {
    T_GT,
    T_LT,
    T_PLUS,
    T_MINUS,
    T_DOT,
    T_COMMA,
    T_LBRACKET,
    T_RBRACKET,
}

fn parse(code: &str) -> Vec<Token> {
    let mut tokens = vec![];

    for c in code.chars() {
        match c {
            '>' => tokens.push(Token::T_GT),
            '<' => tokens.push(Token::T_LT),
            '+' => tokens.push(Token::T_PLUS),
            '-' => tokens.push(Token::T_MINUS),
            '.' => tokens.push(Token::T_DOT),
            ',' => tokens.push(Token::T_COMMA),
            '[' => tokens.push(Token::T_LBRACKET),
            ']' => tokens.push(Token::T_RBRACKET),
            _ => {}
        }
    }

    tokens
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum OpCode {
    Right,
    Left,
    Inc,
    Dec,
    Print,
    Read,
    JumpIfZero(usize),
    Jump(usize),
}

#[derive(Debug)]
enum CondensedOpCode {
    Right(u8),
    Left(u8),
    Inc(Wrapping<u8>),
    Dec(Wrapping<u8>),
    Print,
    Read,
    JumpIfZero(usize),
    Jump(usize),
}

fn compile(toks: &Vec<Token>) -> Result<Vec<OpCode>, &str> {
    let mut idx: usize = 0;
    let mut brackets = vec![];
    let mut bytecode = vec![];

    for tok in toks {
        match tok {
            Token::T_GT => bytecode.push(OpCode::Right),
            Token::T_LT => bytecode.push(OpCode::Left),
            Token::T_PLUS => bytecode.push(OpCode::Inc),
            Token::T_MINUS => bytecode.push(OpCode::Dec),
            Token::T_DOT => bytecode.push(OpCode::Print),
            Token::T_COMMA => bytecode.push(OpCode::Read),
            Token::T_LBRACKET => {
                brackets.push(idx);
                bytecode.push(OpCode::JumpIfZero(0));
            }
            Token::T_RBRACKET => match brackets.pop() {
                None => return Err("Unmatched right bracket!"),
                Some(j) => {
                    bytecode.push(OpCode::Jump(j));
                    bytecode[j] = OpCode::JumpIfZero(idx + 1);
                }
            },
        }

        idx += 1;
    }

    if brackets.len() == 0 {
        Ok(bytecode)
    } else {
        Err("Unmatched left bracket!")
    }
}

fn condense(opcodes: &Vec<OpCode>) -> Vec<CondensedOpCode> {
    let mut grouped: Vec<(OpCode, u8)> = Vec::new();

    for &opcode in opcodes {
        if grouped.is_empty() {
            grouped.push((opcode, 1));
        } else {
            let idx = grouped.len() - 1;
            let (last_code, count) = grouped[idx];
            if last_code == opcode {
                grouped[idx] = (last_code, count + 1);
            } else {
                grouped.push((opcode, 1));
            }
        }
    }

    let mut condensed: Vec<CondensedOpCode> = Vec::new();

    for (code, n) in grouped {
        match code {
            OpCode::Right => condensed.push(CondensedOpCode::Right(n)),
            OpCode::Left => condensed.push(CondensedOpCode::Left(n)),
            OpCode::Inc => condensed.push(CondensedOpCode::Inc(Wrapping(n))),
            OpCode::Dec => condensed.push(CondensedOpCode::Dec(Wrapping(n))),
            OpCode::Print => {
                for _ in 0..n {
                    condensed.push(CondensedOpCode::Print);
                }
            }
            OpCode::Read => {
                for _ in 0..n {
                    condensed.push(CondensedOpCode::Read);
                }
            }
            OpCode::JumpIfZero(j) => {
                for _ in 0..n {
                    condensed.push(CondensedOpCode::JumpIfZero(j));
                }
            }
            OpCode::Jump(j) => {
                for _ in 0..n {
                    condensed.push(CondensedOpCode::Jump(j));
                }
            }
        }
    }

    condensed
}

struct Tape {
    left: Vec<Wrapping<u8>>,
    curr: Wrapping<u8>,
    right: Vec<Wrapping<u8>>,
}

impl Tape {
    fn new() -> Tape {
        let zero = Wrapping(0);
        Tape {
            left: vec![zero, zero, zero, zero, zero, zero, zero, zero],
            curr: zero,
            right: vec![zero, zero, zero, zero, zero, zero, zero, zero],
        }
    }

    fn move_left(&mut self) {
        self.right.push(self.curr);
        match self.left.pop() {
            Some(c) => self.curr = c,
            None => self.curr = Wrapping(0),
        }
    }

    fn move_right(&mut self) {
        self.left.push(self.curr);
        match self.right.pop() {
            Some(c) => self.curr = c,
            None => self.curr = Wrapping(0),
        }
    }

    // TODO: optimize the hell out of this
    fn move_leftn(&mut self, n: u8) {
        for _ in 0..n {
            self.move_left();
        }
    }

    fn move_rightn(&mut self, n: u8) {
        for _ in 0..n {
            self.move_right();
        }
    }
}

fn fill_buffer(buffer: &mut Vec<u8>) {
    while buffer.len() == 0 {
        let mut s = String::new();
        io::stdin().read_line(&mut s).expect("Failed to read line");
        for c in s.as_bytes() {
            buffer.push(*c);
        }
        // buffer.pop(); // drop newline
        buffer.reverse();
    }
}

fn interpret(bytecode: &Vec<CondensedOpCode>) {
    let mut tape = Tape::new();
    let mut idx: usize = 0;

    let mut buffer: Vec<u8> = Vec::new();

    while idx < bytecode.len() {
        match bytecode[idx] {
            CondensedOpCode::Right(n) => tape.move_rightn(n),
            CondensedOpCode::Left(n) => tape.move_leftn(n),
            CondensedOpCode::Inc(n) => tape.curr += n,
            CondensedOpCode::Dec(n) => tape.curr -= n,
            CondensedOpCode::Print => print!("{}", tape.curr.0 as char),
            CondensedOpCode::Read => match buffer.pop() {
                Some(c) => tape.curr = Wrapping(c),
                None => {
                    fill_buffer(&mut buffer);
                    tape.curr = Wrapping(buffer.pop().unwrap());
                }
            },
            CondensedOpCode::JumpIfZero(j) => match tape.curr.0 {
                0 => {
                    idx = j;
                    continue;
                }
                _ => {}
            },
            CondensedOpCode::Jump(j) => {
                idx = j;
                continue;
            }
        }

        idx += 1;
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let file_name = &args[1];
        let mut file = File::open(file_name).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        let toks = parse(&code);
        match compile(&toks) {
            Ok(bytecode) => {
                let bytecode = condense(&bytecode);
                println!("{:?}", bytecode);
                interpret(&bytecode);
            }
            Err(e) => println!("{}", e),
        }
    } else {
        println!("please provide a file name");
    }
}
