use std::env;
use std::fs::File;
use std::io;
use std::io::Read;

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

#[derive(Debug)]
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

fn compile(toks: &[Token]) -> Result<Vec<OpCode>, &str> {
    let mut brackets = vec![];
    let mut bytecode = vec![];

    for (idx, tok) in toks.iter().enumerate() {
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
    }

    if brackets.is_empty() {
        Ok(bytecode)
    } else {
        Err("Unmatched left bracket!")
    }
}

struct Tape {
    left: Vec<u8>,
    curr: u8,
    right: Vec<u8>,
}

impl Tape {
    fn new() -> Tape {
        Tape {
            left: vec![0, 0, 0, 0, 0, 0, 0, 0],
            curr: 0,
            right: vec![0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    fn move_left(&mut self) {
        self.right.push(self.curr);
        match self.left.pop() {
            Some(c) => self.curr = c,
            None => self.curr = 0,
        }
    }

    fn move_right(&mut self) {
        self.left.push(self.curr);
        match self.right.pop() {
            Some(c) => self.curr = c,
            None => self.curr = 0,
        }
    }
}

fn fill_buffer(buffer: &mut Vec<u8>) {
    while buffer.is_empty() {
        let mut s = String::new();
        io::stdin().read_line(&mut s).expect("Failed to read line");
        for c in s.as_bytes() {
            buffer.push(*c);
        }
        // buffer.pop(); // drop newline
        buffer.reverse();
    }
}

fn inc(x: &mut u8) {
    if *x == 255 {
        *x = 0;
    } else {
        *x += 1;
    }
}

fn dec(x: &mut u8) {
    if *x == 0 {
        *x = 255;
    } else {
        *x -= 1;
    }
}

fn interpret(bytecode: &[OpCode]) {
    let mut tape = Tape::new();
    let mut idx: usize = 0;

    let mut buffer = vec![];

    while idx < bytecode.len() {
        //println!("{}: {}, {:?}", idx, tape.curr, bytecode[idx]);
        match bytecode[idx] {
            OpCode::Right => tape.move_right(),
            OpCode::Left => tape.move_left(),
            OpCode::Inc => inc(&mut tape.curr),
            OpCode::Dec => dec(&mut tape.curr),
            OpCode::Print => print!("{}", tape.curr as char),
            OpCode::Read => match buffer.pop() {
                Some(c) => tape.curr = c,
                None => {
                    fill_buffer(&mut buffer);
                    tape.curr = buffer.pop().unwrap();
                }
            },
            OpCode::JumpIfZero(j) => {
                if tape.curr == 0 {
                    idx = j;
                    continue;
                }
            }
            OpCode::Jump(j) => {
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
            Ok(bytecode) => interpret(&bytecode),
            Err(e) => println!("{}", e),
        }
    } else {
        println!("please provide a file name");
    }
}
