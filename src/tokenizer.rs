use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    IncrementData(u8),       // +
    DecrementData(u8),       // -
    IncrementPointer(usize), // >
    DecrementPointer(usize), // <
    Input,                   // ,
    Output,                  // .
    LoopStart(u32),          // [
    LoopEnd(u32),            // ]
}

#[derive(Debug, thiserror::Error)]
pub enum TokenizerErrorKind {
    #[error("Unclose left bracket")]
    UncloseLeftBracket,

    #[error("Unclose right bracket")]
    UncloseRightBracket,
}

#[derive(Debug)]
pub struct TokenizerError {
    line: i32,
    col: i32,
    kind: TokenizerErrorKind,
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}:{}", self.kind, self.line, self.col)
    }
}
impl std::error::Error for TokenizerError {}

pub fn tokenizer(src: &str) -> Result<Vec<Token>, TokenizerError> {
    let mut ir: Vec<Token> = vec![];
    let mut stk: Vec<(u32, i32, i32)> = vec![];
    let mut line: i32 = 1;
    let mut col: i32 = 0;
    let mut pc = 0;

    for chr in src.chars() {
        match chr {
            '\n' => {
                // new line
                line += 1;
                col = 0;
            }
            '+' => ir.push(Token::IncrementData(1)),
            '-' => ir.push(Token::DecrementData(1)),
            '>' => ir.push(Token::IncrementPointer(1)),
            '<' => ir.push(Token::DecrementPointer(1)),
            ',' => ir.push(Token::Input),
            '.' => ir.push(Token::Output),
            '[' => {
                stk.push((pc, line, col));
                ir.push(Token::LoopStart(0));
            }
            ']' => {
                let (org, _, _) = stk.pop().ok_or(TokenizerError {
                    line,
                    col,
                    kind: TokenizerErrorKind::UncloseLeftBracket,
                })?;
                ir.push(Token::LoopEnd(org));
                if ir.get(org as usize) == Some(&Token::LoopStart(0)) {
                    ir[org as usize] = Token::LoopStart(pc);
                }
            }

            _ => {}
        }
        pc = ir.len() as u32;
    }

    if let Some((_, line, col)) = stk.pop() {
        return Err(TokenizerError {
            line,
            col,
            kind: TokenizerErrorKind::UncloseRightBracket,
        });
    }
    Ok(ir)
}

pub fn optimize(tokens: &mut Vec<Token>) {
    let mut observer = 0;
    let mut writer = 0;
    let len = tokens.len();

    let mut stk: Vec<usize> = vec![];

    macro_rules! _flod_ir {
        ($var:ident, $x:ident) => {{
            let mut j = observer + 1;
            while j < len {
                if let $var(d) = tokens[j] {
                    $x = $x.wrapping_add(d);
                } else {
                    break;
                }
                j += 1;
            }
            observer = j;
            tokens[writer] = $var($x);
            writer += 1;
        }};
    }

    macro_rules! _normal_ir {
        () => {{
            tokens[writer] = tokens[observer];
            writer += 1;
            observer += 1;
        }};
    }

    macro_rules! _loop_start_ir {
        () => {{
            stk.push(writer);
            tokens[writer] = Token::LoopStart(0);
            writer += 1;
            observer += 1;
        }};
    }

    macro_rules! _loop_end_ir {
        () => {{
            let org: usize = stk.pop().unwrap();
            if tokens.get(org) == Some(&Token::LoopStart(0)) {
                tokens[org] = Token::LoopStart(writer as u32);
            }
            tokens[writer] = Token::LoopEnd(org as u32);
            writer += 1;
            observer += 1;
        }};
    }

    use Token::*;
    while observer < len {
        match tokens[observer] {
            IncrementData(mut x) => _flod_ir!(IncrementData, x),
            DecrementData(mut x) => _flod_ir!(DecrementData, x),
            IncrementPointer(mut x) => _flod_ir!(IncrementPointer, x),
            DecrementPointer(mut x) => _flod_ir!(DecrementPointer, x),
            Input => _normal_ir!(),
            Output => _normal_ir!(),
            LoopStart(_) => _loop_start_ir!(),
            LoopEnd(_) => _loop_end_ir!(),
        }
    }
    tokens.truncate(writer);
    tokens.shrink_to_fit();
}

#[test]
fn test_compile() {
    assert_eq!(
        tokenizer("+[,.]").unwrap(),
        vec![
            Token::IncrementData(1),
            Token::LoopStart(4),
            Token::Input,
            Token::Output,
            Token::LoopEnd(1),
        ]
    );

    assert_eq!(
        tokenizer(
            "[arst]+[,.  
        +]"
        )
        .unwrap(),
        vec![
            Token::LoopStart(1),
            Token::LoopEnd(0),
            Token::IncrementData(1),
            Token::LoopStart(7),
            Token::Input,
            Token::Output,
            Token::IncrementData(1),
            Token::LoopEnd(3),
        ]
    );

    match tokenizer("]").unwrap_err().kind {
        TokenizerErrorKind::UncloseLeftBracket => {}
        _ => panic!(),
    }

    match tokenizer("[").unwrap_err().kind {
        TokenizerErrorKind::UncloseRightBracket => {}
        _ => panic!(),
    }

    let mut token = tokenizer("[++++++]").unwrap();
    optimize(&mut token);
    assert_eq!(
        token,
        vec![
            Token::LoopStart(2),
            Token::IncrementData(6),
            Token::LoopEnd(0),
        ]
    )
}
