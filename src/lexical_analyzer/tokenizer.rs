use std::{cell::Cell, str::from_utf8};

use super::*;

// a token and the offset just past it, or an error and the offset just past
// the lexeme it reports, so the lexer can move on after an error
type Lexed<'a, T> = Result<(T, usize), (LexicalError, usize)>;

// how many columns a tab occupies at most when no width is given
pub const DEFAULT_TAB_WIDTH: u32 = 4;

pub struct LexicalAnalyzer<'a> {
    program: &'a Vec<u8>,
    tab_width: u32,
    cursor: Cell<usize>,
    // the position of the cursor (both 0-based).
    line: Cell<u32>,
    column: Cell<u32>,
    // the position of the token most recently returned by get_token or lookahead
    last_token_pos: Cell<TokenPosition>,
}

impl<'a> LexicalAnalyzer<'a> {
    pub fn new(program: &'a Vec<u8>) -> LexicalAnalyzer<'a> {
        LexicalAnalyzer::with_tab_width(program, DEFAULT_TAB_WIDTH)
    }

    pub fn with_tab_width(program: &'a Vec<u8>, tab_width: u32) -> LexicalAnalyzer<'a> {
        LexicalAnalyzer {
            program,
            tab_width: tab_width.max(1),
            cursor: Cell::new(0),
            line: Cell::new(0),
            column: Cell::new(0),
            last_token_pos: Cell::new(TokenPosition::default()),
        }
    }

    // sets cursor to its initial position at zero
    pub fn reset_cursor(&self) {
        self.cursor.set(0);
        self.line.set(0);
        self.column.set(0);
    }

    // get the next token without advancing the cursor
    pub fn lookahead(&self) -> Result<Token<'a>, LexicalError> {
        self.parse(true)
    }

    pub fn get_last_token_position(&self) -> TokenPosition {
        self.last_token_pos.get()
    }

    pub fn get_token(&self) -> Result<Token<'a>, LexicalError> {
        self.parse(false)
    }

    fn parse(&self, peek: bool) -> Result<Token<'a>, LexicalError> {
        self.skip_whitespace_and_comments();
        let start = self.cursor.get();
        let (result, end) = match self.lex_token(start) {
            Ok((token, end)) => {
                self.last_token_pos.set(self.token_position(start, end));
                (Ok(token), end)
            }
            Err((error, end)) => {
                self.last_token_pos.set(error.position);
                (Err(error), end)
            }
        };
        if !peek {
            self.consume(end);
        }
        result
    }

    // the token starting at `start` and the offset just past it; the cursor is
    // left where it is, so the current line and column are the token's
    fn lex_token(&self, start: usize) -> Lexed<'a, Token<'a>> {
        let Some(&byte) = self.program.get(start) else {
            return Ok((Token::EOF, start));
        };
        let single = |token| Ok((token, start + 1));
        match byte {
            // Punctuations
            b'-' => single(Token::Punctuator(PunctuationType::Dash)),
            b'(' => single(Token::Punctuator(PunctuationType::LParentheses)),
            b')' => single(Token::Punctuator(PunctuationType::RParentheses)),
            // Arithmetic
            b'+' => single(Token::Operator(OperationType::Addition)),
            b'*' => single(Token::Operator(OperationType::Multiplication)),
            // Ordering Relations
            b'=' => single(Token::Operator(OperationType::Equal)),
            b'<' | b'>' => {
                let or_equal = self.program.get(start + 1) == Some(&b'=');
                let operator = match (byte, or_equal) {
                    (b'<', false) => OperationType::LessThan,
                    (b'<', true) => OperationType::LessThanOrEqual,
                    (_, false) => OperationType::GreaterThan,
                    (_, true) => OperationType::GreaterThanOrEqual,
                };
                Ok((Token::Operator(operator), start + 1 + or_equal as usize))
            }
            // Variables (the '?' prefix is kept as part of the name)
            b'?' => {
                let (name, end) = self.lexeme(start, start)?;
                Ok((Token::Identifier(name), end))
            }
            // Keywords (Note that 2 keywords, namely "domain" and "problem", can start without ':' as well)
            b':' => {
                let (lexeme, end) = self.lexeme(start, start + 1)?;
                let token = match lexeme {
                    // Requirements
                    "negative-preconditions" => {
                        Token::Requirement(RequirementType::NegativePreconditions)
                    }
                    "strips" => Token::Requirement(RequirementType::STRIPS),
                    "hierarchy" => Token::Requirement(RequirementType::Hierarchy),
                    "equality" => Token::Requirement(RequirementType::Equality),
                    "method-preconditions" => {
                        Token::Requirement(RequirementType::MethodPreconditions)
                    }
                    "typing" => Token::Requirement(RequirementType::TypedObjects),
                    "universal-preconditions" => {
                        Token::Requirement(RequirementType::UniversalPreconditions)
                    }
                    "existential-preconditions" => {
                        Token::Requirement(RequirementType::ExistentialPreconditions)
                    }
                    "quantified-preconditions" => {
                        Token::Requirement(RequirementType::QuantifiedPreconditions)
                    }
                    "conditional-effects" => {
                        Token::Requirement(RequirementType::ConditionalEffects)
                    }
                    "probabilistic-effects" => {
                        Token::Requirement(RequirementType::ProbabilisticEffects)
                    }
                    "domain-axioms" => Token::Requirement(RequirementType::DomainAxioms),
                    "adl" => Token::Requirement(RequirementType::ADL),
                    "fluents" => Token::Requirement(RequirementType::Fluents),
                    "numeric-fluents" => Token::Requirement(RequirementType::NumericFluents),
                    "action-costs" => Token::Requirement(RequirementType::ActionCosts),
                    "disjunctive-preconditions" => {
                        Token::Requirement(RequirementType::DisjunctivePreconditions)
                    }
                    "durative-actions" => Token::Requirement(RequirementType::DurativeActions),
                    "duration-inequalities" => {
                        Token::Requirement(RequirementType::DurationInequalities)
                    }
                    "continuous-effects" => Token::Requirement(RequirementType::ContinuousEffects),
                    // Keywords
                    "requirements" => Token::Keyword(KeywordName::Requirements),
                    "objects" => Token::Keyword(KeywordName::Objects),
                    "types" => Token::Keyword(KeywordName::Types),
                    "constants" => Token::Keyword(KeywordName::Constants),
                    "predicates" => Token::Keyword(KeywordName::Predicates),
                    "functions" => Token::Keyword(KeywordName::Functions),
                    "init" => Token::Keyword(KeywordName::Init),
                    "htn" => Token::Keyword(KeywordName::HTN),
                    "task" => Token::Keyword(KeywordName::Task),
                    "action" => Token::Keyword(KeywordName::Action),
                    "parameters" => Token::Keyword(KeywordName::Parameters),
                    "method" => Token::Keyword(KeywordName::Method),
                    "precondition" => Token::Keyword(KeywordName::Precondition),
                    "effect" => Token::Keyword(KeywordName::Effect),
                    "subtasks" | "tasks" => Token::Keyword(KeywordName::Subtasks),
                    "ordered-subtasks" | "ordered-tasks" => {
                        Token::Keyword(KeywordName::OrderedSubtasks)
                    }
                    "ordering" | "order" => Token::Keyword(KeywordName::Ordering),
                    "constraints" => Token::Keyword(KeywordName::Constraints),
                    "goal" => Token::Keyword(KeywordName::Goal),
                    "metric" => Token::Keyword(KeywordName::Metric),
                    "domain" => Token::Keyword(KeywordName::Domain),
                    "problem" => Token::Keyword(KeywordName::Problem),
                    _ => {
                        return Err(self.error(
                            LexicalErrorType::InvalidKeyword,
                            start,
                            start + 1,
                            end,
                        ));
                    }
                };
                Ok((token, end))
            }
            // Numeric literal
            b'0'..=b'9' => {
                let (value, end) = self.number(start)?;
                Ok((Token::Number(value), end))
            }
            // Other
            _ => {
                let (lexeme, end) = self.lexeme(start, start)?;
                let token = match lexeme {
                    // Remaining Keywords
                    "define" => Token::Keyword(KeywordName::Define),
                    "domain" => Token::Keyword(KeywordName::Domain),
                    "problem" => Token::Keyword(KeywordName::Problem),
                    _ => match LexicalAnalyzer::is_operator(lexeme) {
                        // Logical Operators
                        Some(operator) => Token::Operator(operator),
                        // Identifier
                        None => {
                            if lexeme
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                            {
                                Token::Identifier(lexeme)
                            } else {
                                return Err(self.error(
                                    LexicalErrorType::InvalidIdentifier,
                                    start,
                                    start,
                                    end,
                                ));
                            }
                        }
                    },
                };
                Ok((token, end))
            }
        }
    }

    // the run of identifier characters from `start` and the offset just past
    // it; `token_start` is where the enclosing token begins (before a ':'
    // prefix), which is where an error is reported
    fn lexeme(&self, token_start: usize, start: usize) -> Lexed<'a, &'a str> {
        let mut end = start;
        let mut is_invalid = false;
        while let Some(&byte) = self.program.get(end) {
            match byte {
                b'(' | b')' | b' ' | b'\t' | b'\n' | b'\r' => break,
                b'_' | b'-' => {}
                // a variable's '?' prefix is only valid as the lexeme's first character
                b'?' => is_invalid |= end != start,
                byte => is_invalid |= !(byte as char).is_alphanumeric(),
            }
            end += 1;
        }
        if is_invalid {
            Err(self.error(LexicalErrorType::InvalidIdentifier, token_start, start, end))
        } else {
            Ok((self.text(start, end), end))
        }
    }

    fn number(&self, start: usize) -> Lexed<'a, NumberType> {
        let mut end = start;
        let mut seen_dot = false;
        while let Some(&byte) = self.program.get(end) {
            if byte.is_ascii_digit() {
                end += 1;
            } else if byte == b'.' && !seen_dot {
                seen_dot = true;
                end += 1;
            } else {
                break;
            }
        }
        let lexeme = self.text(start, end);
        let value = if seen_dot {
            lexeme
                .parse::<f64>()
                .map(NumberType::Real)
                .map_err(|_| self.error(LexicalErrorType::InvalidRealNumber, start, start, end))
        } else {
            lexeme
                .parse::<i64>()
                .map(NumberType::Integer)
                .map_err(|_| self.error(LexicalErrorType::InvalidIdentifier, start, start, end))
        }?;
        Ok((value, end))
    }

    fn error(
        &self,
        error_type: LexicalErrorType,
        token_start: usize,
        lexeme_start: usize,
        end: usize,
    ) -> (LexicalError, usize) {
        let error = LexicalError {
            error_type,
            lexeme: self.text(lexeme_start, end).to_string(),
            position: self.token_position(token_start, end),
        };
        (error, end)
    }

    // the position of a token spanning `start..end`, which must begin at the cursor
    fn token_position(&self, start: usize, end: usize) -> TokenPosition {
        TokenPosition {
            line: self.line.get(),
            column: self.column.get(),
            length: LexicalAnalyzer::char_count(&self.program[start..end]),
        }
    }

    // moves the cursor to `end` within the current line
    fn consume(&self, end: usize) {
        let start = self.cursor.get();
        self.column
            .set(self.column.get() + LexicalAnalyzer::char_count(&self.program[start..end]));
        self.cursor.set(end);
    }

    fn skip_whitespace_and_comments(&self) {
        while let Some(&byte) = self.program.get(self.cursor.get()) {
            match byte {
                b'\n' => {
                    self.line.set(self.line.get() + 1);
                    self.column.set(0);
                }
                b'\t' => {
                    let column = self.column.get();
                    self.column
                        .set(column + self.tab_width - column % self.tab_width);
                }
                b' ' => self.column.set(self.column.get() + 1),
                b'\r' => {}
                // a comment runs to the end of its line; the newline is consumed as whitespace
                b';' => {
                    let start = self.cursor.get();
                    let end = self.program[start..]
                        .iter()
                        .position(|&byte| byte == b'\n')
                        .map_or(self.program.len(), |offset| start + offset);
                    self.consume(end);
                    continue;
                }
                _ => break,
            }
            self.cursor.set(self.cursor.get() + 1);
        }
    }

    fn text(&self, start: usize, end: usize) -> &'a str {
        from_utf8(&self.program[start..end]).unwrap()
    }

    // characters, not bytes: UTF-8 continuation bytes are not counted
    fn char_count(bytes: &[u8]) -> u32 {
        bytes.iter().filter(|&&byte| byte & 0xC0 != 0x80).count() as u32
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
            "imply" | "when" => Some(OperationType::Implication),
            // Stochastic operators
            "probabilistic" => Some(OperationType::Probabilistic),
            // Numeric effect operators
            "assign" => Some(OperationType::Assign),
            "increase" => Some(OperationType::Increase),
            "decrease" => Some(OperationType::Decrease),
            "scale-up" => Some(OperationType::ScaleUp),
            "scale-down" => Some(OperationType::ScaleDown),
            _ => None,
        }
    }
}
