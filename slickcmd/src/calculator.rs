use bc::interpreter::Interpreter;
use bc::number::Number;
use bc::parser;
use std::collections::HashSet;

#[derive(PartialEq)]
enum PartType {
    None,
    Variable,
    Number,
    Operator,
    Separator,
}

struct ParsedPart {
    typ: PartType,
    text: String,
}

fn parse_parts(chars: &[char]) -> Option<Vec<ParsedPart>> {
    let mut parts = Vec::<ParsedPart>::new();

    let mut part = ParsedPart {
        typ: PartType::None,
        text: String::new(),
    };
    for &c in chars {
        let typ = match c {
            ' ' => PartType::None,
            '+' | '-' | '*' | '/' | '%' | '^' | '!' | '=' | ',' => PartType::Operator,
            '(' | ')' | '[' | ']' | '<' | '>' | '{' | '}' => PartType::Operator,
            ';' | '\n' => PartType::Separator,
            '.' => {
                if part.typ == PartType::Number && part.text.find('.').is_some() {
                    return None;
                }
                PartType::Number
            }
            '0'..='9' => {
                if part.typ == PartType::Variable {
                    PartType::Variable
                } else {
                    PartType::Number
                }
            }
            'a'..='z' => {
                if part.typ == PartType::Number {
                    return None;
                }
                PartType::Variable
            }
            _ => return None,
        };
        if typ == part.typ {
            part.text.push(c);
        } else {
            if part.typ != PartType::None {
                parts.push(part);
            }
            part = ParsedPart {
                typ,
                text: c.to_string(),
            }
        }
    }
    if part.typ != PartType::None {
        parts.push(part);
    }

    Some(parts)
}

pub fn accepts_input(input: &str) -> bool {
    let input = input.trim_ascii();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    if len == 0 {
        return false;
    }
    if len > 2 && chars[0] == '0' {
        let base = match chars[1] {
            'x' | 'X' => 16,
            'o' | 'O' => 8,
            'b' | 'B' => 2,
            _ => 0,
        };
        if base != 0 && Number::parse(&input[2..], base).is_some() {
            return true;
        }
    }

    let parts = parse_parts(&chars);
    if parts.is_none() {
        return false;
    }
    let mut parts = parts.unwrap();
    parts.push(ParsedPart {
        typ: PartType::Separator,
        text: String::new(),
    });

    let mut defined_vars: HashSet<String> = HashSet::new();
    let count = parts.len();
    for n in 0..count - 1 {
        let part = &parts[n];
        let next_part = &parts[n + 1];
        if part.typ == PartType::Variable {
            if next_part.typ != PartType::Operator && next_part.typ != PartType::Separator {
                return false;
            }
            if !defined_vars.contains(&part.text) {
                if next_part.text == "=" {
                    defined_vars.insert(part.text.clone());
                } else if next_part.text == "(" {
                    //func call?
                } else {
                    return false;
                }
            }
        } else if part.typ == PartType::Number {
            if !(next_part.typ != PartType::Operator || next_part.typ != PartType::Separator) {
                return false;
            }
        }
    }
    true
}

pub fn evaluate(input: &str) -> String {
    let input = input.trim_ascii();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    if len > 2 && chars[0] == '0' {
        let base = match chars[1] {
            'x' | 'X' => 16,
            'o' | 'O' => 8,
            'b' | 'B' => 2,
            _ => 0,
        };
        if base != 0 {
            let num = Number::parse(&input[2..], base);
            if num.is_some() {
                return num.unwrap().to_string(10);
            }
        }
    }

    let program = input.to_string() + "\n";
    let mut interpreter = Interpreter::default();
    match parser::parse_program(&program, None) {
        Ok(program) => {
            let result = interpreter.exec(program);
            result.unwrap_or_else(|e| format!("{}{}", e.partial_output(), e))
        }
        Err(e) => {
            e.to_string()
        }
    }
}
