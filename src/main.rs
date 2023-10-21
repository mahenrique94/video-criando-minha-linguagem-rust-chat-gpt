use std::fs;
use std::process::Command;

#[derive(Debug)]
enum ValueType {
    Str(String),
    Int(i32),
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Var,
    Mut,
    Identifier(String),
    Equals,
    StringLiteral(String),
    IntLiteral(i32),
    Semicolon,
    Print,
    OpenBrace,
    CloseBrace,
}

#[derive(Debug)]
struct VariableDeclaration {
    mutable: bool,
    name: String,
    value: ValueType,
}

#[derive(Debug)]
struct FunctionCall {
    name: String,
    arguments: Vec<ValueType>,
}

#[derive(Debug)]
enum Statement {
    VariableDeclaration(VariableDeclaration),
    FunctionCall(FunctionCall),
}

type AST = Vec<Statement>;

fn lexer(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => { chars.next(); },
            'a'..='z' | 'A'..='Z' => {
                let mut name = String::new();
                while let Some(&ch) = chars.peek() {
                    match ch {
                        'a'..='z' | 'A'..='Z' => {
                            name.push(chars.next().unwrap());
                        },
                        _ => break,
                    }
                }

                if name == "print" {
                    tokens.push(Token::Print);
                }

                if name == "var" {
                    tokens.push(Token::Var);
                } else if name == "mut" {
                    tokens.push(Token::Mut);
                } else {
                    tokens.push(Token::Identifier(name));
                }
            },
            '"' => {
                chars.next();
                let mut string = String::new();
                while let Some(&ch) = chars.peek() {
                    match ch {
                        '"' => { chars.next(); break; },
                        ch => string.push(chars.next().unwrap()),
                    }
                }
                tokens.push(Token::StringLiteral(string));
            },
            '0'..='9' => {
                let mut number = String::new();
                while let Some(&ch) = chars.peek() {
                    match ch {
                        '0'..='9' => {
                            number.push(chars.next().unwrap());
                        },
                        _ => break,
                    }
                }
                tokens.push(Token::IntLiteral(number.parse().unwrap()));
            },
            '=' => {
                chars.next();
                tokens.push(Token::Equals);
            },
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            },
            '{' => {
                chars.next();
                tokens.push(Token::OpenBrace);
            },
            '}' => {
                chars.next();
                tokens.push(Token::CloseBrace);
            },
            _ => {
                chars.next();
            }
        }
    }

    tokens
}

fn parser(tokens: &[Token]) -> AST {
    let mut ast = Vec::new();
    let mut tokens = tokens.iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            Token::Var => {
                let is_mut = matches!(tokens.peek(), Some(Token::Mut));
                if is_mut {
                    tokens.next(); // consume Mut
                }
                if let Some(Token::Identifier(name)) = tokens.next() {
                    tokens.next(); // consume Equals
                    match tokens.next() {
                        Some(Token::StringLiteral(value)) => {
                            ast.push(Statement::VariableDeclaration(VariableDeclaration {
                                mutable: is_mut,
                                name: name.clone(),
                                value: ValueType::Str(value.clone()),
                            }));
                        },
                        Some(Token::IntLiteral(value)) => {
                            ast.push(Statement::VariableDeclaration(VariableDeclaration {
                                mutable: is_mut,
                                name: name.clone(),
                                value: ValueType::Int(*value),
                            }));
                        },
                        _ => panic!("Unexpected token after equals!"),
                    }
                    tokens.next(); // consume Semicolon
                } else {
                    panic!("Expected identifier after var/mut!");
                }
            },
            Token::Print => {
                tokens.next(); // consume '('
                let argument = match tokens.next() {
                    Some(Token::StringLiteral(value)) => ValueType::Str(value.clone()),
                    Some(Token::IntLiteral(value)) => ValueType::Int(*value),
                    _ => panic!("Unexpected token in print arguments!"),
                };
                tokens.next(); // consume ')'
                ast.push(Statement::FunctionCall(FunctionCall {
                    name: "print".to_string(),
                    arguments: vec![argument],
                }));
            },
            _ => {}
        }
    }

    ast
}

fn generate_js(ast: &AST) -> String {
    let mut js_code = String::new();

    for statement in ast {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                let var_type = if declaration.mutable { "let" } else { "const" };
                match &declaration.value {
                    ValueType::Str(s) => {
                        js_code.push_str(&format!("{} {} = \'{}\'\n", var_type, declaration.name, s));
                    },
                    ValueType::Int(i) => {
                        js_code.push_str(&format!("{} {} = {}\n", var_type, declaration.name, i));
                    },
                }
            },
            Statement::FunctionCall(call) => {
                if call.name == "print" {
                    for arg in &call.arguments {
                        match arg {
                            ValueType::Str(s) => {
                                let mut interpolated_str = s.clone();
                                for segment in s.split(|c| c == '{' || c == '}').collect::<Vec<_>>() {
                                    if lexer(segment).first() == Some(&Token::Identifier(segment.to_string())) {
                                        interpolated_str = interpolated_str.replace(&format!("{{{}}}", segment), &format!("${{{}}}", segment));
                                    }
                                }
                                js_code.push_str(&format!("console.log(`{}`)\n", interpolated_str));
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    js_code
}

fn main() {
    let input_filename = "index.mc";
    let output_filename = "index.js";
    let code = fs::read_to_string(input_filename)
        .expect("Failed to read the source file.");

    let tokens = lexer(&code);
    let ast: Vec<Statement> = parser(&tokens);
    let js_code = generate_js(&ast);

    fs::write(output_filename, js_code)
        .expect("Failed to write the output file.");

    Command::new("node")
        .arg(output_filename)
        .status()
        .expect("Failed to execute command");
}
