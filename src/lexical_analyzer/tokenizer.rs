use std::{cell::Cell, str::from_utf8};

use super::*;

pub struct LexicalAnalyzer<'a> {
    program: &'a Vec<u8>,
    cursor: Cell<usize>,
    last_token_pos: Cell<TokenPosition>,
}

impl<'a> LexicalAnalyzer<'a> {
    pub fn new(program: &'a Vec<u8>) -> LexicalAnalyzer<'a> {
        LexicalAnalyzer {
            program,
            cursor: Cell::new(0),
            last_token_pos: Cell::new(TokenPosition { line: 1 }),
        }
    }

    // sets cursor to its initial position at zero
    pub fn reset_cursor(&self) {
        self.cursor.set(0);
    }

    // get the next token without advancing the cursor
    pub fn lookahead(&self) -> Result<Token, LexicalError> {
        return self.parse(true);
    }

    pub fn get_last_token_position(&self) -> TokenPosition {
        self.last_token_pos.get()
    }

    pub fn get_token(&self) -> Result<Token, LexicalError> {
        return self.parse(false);
    }

    fn parse(&self, peek: bool) -> Result<Token, LexicalError> {
        self.skip_whitespaces();
        if self.cursor.get() == self.program.len() {
            return Ok(Token::EOF);
        }
        if let Some(char) = self.peek_next_char() {
            if !peek {
                self.cursor.set(self.cursor.get() + 1);
            }
            match char {
                // Punctuations
                '-' => Ok(Token::Punctuator(PunctuationType::Dash)),
                '(' => Ok(Token::Punctuator(PunctuationType::LParentheses)),
                ')' => Ok(Token::Punctuator(PunctuationType::RParentheses)),
                // Ordering Relations
                p @ ('<' | '>' | '=') => Ok(Token::Operator(self.ordering_type(&p, peek))),
                // Variables
                '?' => {
                    let mut init_cur_pos = self.cursor.get();
                    if peek {
                        init_cur_pos += 1;
                    }
                    let (var_name, new_cur_pos) = self.peek_lexeme(init_cur_pos)?;
                    if !peek {
                        self.cursor.set(new_cur_pos);
                    }
                    Ok(Token::Identifier(var_name))
                }
                // Keywords (Note that 2 keywords, namely "domain" and "problem", can start without ':' as well)
                ':' => {
                    let mut init_cur_pos = self.cursor.get();
                    if peek {
                        init_cur_pos += 1;
                    }
                    let (lexeme, new_cur_pos) = self.peek_lexeme(init_cur_pos)?;
                    if !peek {
                        self.cursor.set(new_cur_pos);
                    }
                    match lexeme {
                        // Requirements
                        "negative-preconditions" => {
                            Ok(Token::Requirement(RequirementType::NegativePreconditions))
                        }
                        "strips" => Ok(Token::Requirement(RequirementType::STRIPS)),
                        "hierarchy" => Ok(Token::Requirement(RequirementType::Hierarchy)),
                        "equality" => Ok(Token::Requirement(RequirementType::Equality)),
                        "method-preconditions" => {
                            Ok(Token::Requirement(RequirementType::MethodPreconditions))
                        }
                        "typing" => Ok(Token::Requirement(RequirementType::TypedObjects)),
                        "universal-preconditions" => {
                            Ok(Token::Requirement(RequirementType::UniversalPreconditions))
                        }
                        // Keywords
                        "requirements" => Ok(Token::Keyword(KeywordName::Requirements)),
                        "objects" => Ok(Token::Keyword(KeywordName::Objects)),
                        "types" => Ok(Token::Keyword(KeywordName::Types)),
                        "constants" => Ok(Token::Keyword(KeywordName::Constants)),
                        "predicates" => Ok(Token::Keyword(KeywordName::Predicates)),
                        "functions" => Ok(Token::Keyword(KeywordName::Functions)),
                        "init" => Ok(Token::Keyword(KeywordName::Init)),
                        "htn" => Ok(Token::Keyword(KeywordName::HTN)),
                        "task" => Ok(Token::Keyword(KeywordName::Task)),
                        "action" => Ok(Token::Keyword(KeywordName::Action)),
                        "parameters" => Ok(Token::Keyword(KeywordName::Parameters)),
                        "method" => Ok(Token::Keyword(KeywordName::Method)),
                        "precondition" => Ok(Token::Keyword(KeywordName::Precondition)),
                        "effect" => Ok(Token::Keyword(KeywordName::Effect)),
                        "subtasks" | "tasks" => Ok(Token::Keyword(KeywordName::Subtasks)),
                        "ordered-subtasks" | "ordered-tasks" => {
                            Ok(Token::Keyword(KeywordName::OrderedSubtasks))
                        }
                        "ordering" | "order" => Ok(Token::Keyword(KeywordName::Ordering)),
                        "constraints" => Ok(Token::Keyword(KeywordName::Constraints)),
                        "goal" => Ok(Token::Keyword(KeywordName::Goal)),
                        "domain" => return Ok(Token::Keyword(KeywordName::Domain)),
                        "problem" => return Ok(Token::Keyword(KeywordName::Problem)),
                        _ => Err(LexicalError {
                            error_type: LexicalErrorType::InvalidKeyword,
                            lexeme: lexeme.to_string(),
                            position: self.last_token_pos.get(),
                        }),
                    }
                }
                // Comment
                ';' => {
                    let mut current = self.program[self.cursor.get()] as char;
                    while current != '\n' {
                        self.cursor.set(self.cursor.get() + 1);
                        current = self.program[self.cursor.get()] as char;
                    }
                    return self.parse(peek);
                }
                // Other
                _ => {
                    let mut init_cur_pos = self.cursor.get() - 1;
                    if peek {
                        init_cur_pos += 1;
                    }
                    // Numeric literal
                    if char.is_ascii_digit() {
                        let (value, new_cur_pos) = self.peek_number(init_cur_pos)?;
                        if !peek {
                            self.cursor.set(new_cur_pos);
                        }
                        return Ok(Token::Number(value));
                    }
                    let (lexeme, new_cur_pos) = self.peek_lexeme(init_cur_pos)?;
                    if !peek {
                        self.cursor.set(new_cur_pos);
                    }
                    match lexeme {
                        // Remaining Keywords
                        "define" => return Ok(Token::Keyword(KeywordName::Define)),
                        "domain" => return Ok(Token::Keyword(KeywordName::Domain)),
                        "problem" => return Ok(Token::Keyword(KeywordName::Problem)),
                        _ => {
                            // Logical Operators
                            match LexicalAnalyzer::is_operator(&lexeme) {
                                Some(x) => return Ok(Token::Operator(x)),
                                // Identifier
                                None => {
                                    if lexeme
                                        .chars()
                                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                                    {
                                        return Ok(Token::Identifier(lexeme));
                                    } else {
                                        Err(LexicalError {
                                            error_type: LexicalErrorType::InvalidIdentifier,
                                            lexeme: lexeme.to_string(),
                                            position: self.last_token_pos.get(),
                                        })
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            Ok(Token::EOF)
        }
    }

    // get next lexeme and new cursor position (to commit peek)
    fn peek_lexeme(&self, init_cur_pos: usize) -> Result<(&str, usize), LexicalError> {
        let mut cursor_pos = init_cur_pos;
        let mut next_ch = self.program[cursor_pos] as char;
        let mut is_invalid = false;
        let mut is_valid_character = |c| match c {
            '_' | '-' => true,
            ')' | '(' => false,
            _ => {
                if LexicalAnalyzer::is_whitespace(&c) {
                    false
                } else {
                    if !c.is_alphanumeric() {
                        is_invalid = true;
                    }
                    true
                }
            }
        };
        while is_valid_character(next_ch) {
            if cursor_pos < self.program.len() - 1 {
                cursor_pos += 1;
                next_ch = self.program[cursor_pos] as char;
            } else {
                break;
            }
        }
        if is_invalid {
            return Err(LexicalError {
                error_type: LexicalErrorType::InvalidIdentifier,
                lexeme: from_utf8(&self.program[init_cur_pos..cursor_pos])
                    .unwrap()
                    .to_string(),
                position: self.last_token_pos.get(),
            });
        } else {
            return Ok((
                from_utf8(&self.program[init_cur_pos..cursor_pos]).unwrap(),
                cursor_pos,
            ));
        }
    }

    fn peek_next_char(&self) -> Option<char> {
        if self.cursor.get() >= self.program.len() {
            return None;
        }
        let current = self.program[self.cursor.get()] as char;
        Some(current)
    }

    fn skip_whitespaces(&self) {
        while self.cursor.get() != self.program.len() {
            let current = self.program[self.cursor.get()] as char;
            if !LexicalAnalyzer::is_whitespace(&current) {
                break;
            } else if current == '\n' {
                self.last_token_pos.set(TokenPosition {
                    line: self.last_token_pos.get().line + 1,
                });
            }
            self.cursor.set(self.cursor.get() + 1);
        }
    }

    fn peek_number(&self, init_cur_pos: usize) -> Result<(NumberType, usize), LexicalError> {
        let mut cursor_pos = init_cur_pos;
        let mut seen_dot = false;
        while cursor_pos < self.program.len() {
            let c = self.program[cursor_pos] as char;
            if c.is_ascii_digit() {
                cursor_pos += 1;
            } else if c == '.' && !seen_dot {
                seen_dot = true;
                cursor_pos += 1;
            } else {
                break;
            }
        }
        let lexeme = from_utf8(&self.program[init_cur_pos..cursor_pos]).unwrap();
        if seen_dot {
            match lexeme.parse::<f64>() {
                Ok(value) => Ok((NumberType::Real(value), cursor_pos)),
                Err(_) => Err(LexicalError {
                    error_type: LexicalErrorType::InvalidRealNumber,
                    lexeme: lexeme.to_string(),
                    position: self.last_token_pos.get(),
                }),
            }
        } else {
            match lexeme.parse::<i64>() {
                Ok(value) => Ok((NumberType::Integer(value), cursor_pos)),
                Err(_) => Err(LexicalError {
                    error_type: LexicalErrorType::InvalidIdentifier,
                    lexeme: lexeme.to_string(),
                    position: self.last_token_pos.get(),
                }),
            }
        }
    }

    fn is_operator(word: &str) -> Option<OperationType> {
        match word {
            // Logical Operators
            "and" => Some(OperationType::And),
            "or" => Some(OperationType::Or),
            "oneof" => Some(OperationType::Xor),
            "not" => Some(OperationType::Not),
            "forall" => Some(OperationType::ForAll),
            "exists" => Some(OperationType::Exists),
            "imply" => Some(OperationType::Implication),
            // Stochastic operators
            "probabilistic" => return Some(OperationType::Probabilistic),
            _ => None,
        }
    }

    fn ordering_type(&self, c: &char, peek: bool) -> OperationType {
        match c {
            '<' => match self.peek_next_char() {
                Some('=') => {
                    if !peek {
                        self.cursor.set(self.cursor.get() + 1);
                    }
                    OperationType::LessThanOrEqual
                }
                _ => OperationType::LessThan,
            },
            '>' => match self.peek_next_char() {
                Some('=') => {
                    if !peek {
                        self.cursor.set(self.cursor.get() + 1);
                    }
                    OperationType::GreaterThanOrEqual
                }
                _ => OperationType::GreaterThan,
            },
            '=' => OperationType::Equal,
            _ => {
                panic!("not an ordering relation");
            }
        }
    }

    fn is_whitespace(c: &char) -> bool {
        match c {
            ' ' | '\t' | '\n' | '\r' => true,
            _ => false,
        }
    }
}
