use self::definition_types::ProblemDefinition;

use super::*;

pub struct Parser<'a> {
    pub tokenizer: LexicalAnalyzer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenizer: LexicalAnalyzer<'a>) -> Parser<'a> {
        Parser { tokenizer }
    }
    pub fn parse(&self) -> Result<AbstractSyntaxTree<'a>, ParsingError> {
        // match opening '('
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::LParentheses) => {
                // Determine file type
                match self.parse_document_type()? {
                    // Domain Definition
                    DefinitionType::Domain(domain_name) => {
                        Ok(self.parse_domain(domain_name)?.into())
                    }
                    // Problem Definition
                    DefinitionType::Problem(problem_definition) => {
                        Ok(self.parse_problem(problem_definition)?.into())
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "start of the file with '('".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }

    pub fn classify(&self) -> FileVariant {
        self.tokenizer.reset_cursor();
        let mut variant = FileVariant::MaybeNotHDDL;
        if let Ok(Token::Punctuator(PunctuationType::LParentheses)) = self.tokenizer.get_token() {
            if let Ok(Token::Keyword(KeywordName::Define)) = self.tokenizer.get_token() {
                if let Ok(Token::Punctuator(PunctuationType::LParentheses)) =
                    self.tokenizer.get_token()
                {
                    match self.tokenizer.get_token() {
                        Ok(Token::Keyword(KeywordName::Domain)) => variant = FileVariant::Domain,
                        Ok(Token::Keyword(KeywordName::Problem)) => variant = FileVariant::Problem,
                        _ => {}
                    }
                }
            }
        }
        self.tokenizer.reset_cursor();
        variant
    }

    fn parse_document_type(&self) -> Result<DefinitionType<'a>, ParsingError> {
        // match keyword 'define'
        match self.tokenizer.get_token()? {
            Token::Keyword(KeywordName::Define) => {
                // match '(' after keyword 'define
                match self.tokenizer.get_token()? {
                    Token::Punctuator(PunctuationType::LParentheses) => {
                        // match either 'domain' or 'problem'
                        match self.tokenizer.get_token()? {
                            Token::Keyword(KeywordName::Domain) => self.parse_domain_header(),
                            Token::Keyword(KeywordName::Problem) => self.parse_problem_header(),
                            token => {
                                let error = SyntacticError {
                                    expected: "either keyword 'domain' or 'problem'".to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                Err(ParsingError::Syntactic(error))
                            }
                        }
                    }
                    token => {
                        let error = SyntacticError {
                            expected: "'(' after keyword 'define'".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        Err(ParsingError::Syntactic(error))
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "keyword 'define'".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }

    fn parse_domain_header(&self) -> Result<DefinitionType<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Identifier(domain_name) => {
                // match closing paranthesis
                match self.tokenizer.get_token()? {
                    Token::Punctuator(PunctuationType::RParentheses) => {
                        Ok(DefinitionType::Domain(domain_name))
                    }
                    token => {
                        let error = SyntacticError {
                            expected: "')'".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        Err(ParsingError::Syntactic(error))
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "domain name".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }

    pub fn parse_problem_header(&self) -> Result<DefinitionType<'a>, ParsingError> {
        // match problem name
        match self.tokenizer.get_token()? {
            Token::Identifier(problem_name) => {
                // match closing paranthesis
                match self.tokenizer.get_token()? {
                    Token::Punctuator(PunctuationType::RParentheses) => {
                        // match '(' for domain name
                        match self.tokenizer.get_token()? {
                            Token::Punctuator(PunctuationType::LParentheses) => {
                                match self.tokenizer.get_token()? {
                                    Token::Keyword(KeywordName::Domain) => {
                                        match self.tokenizer.get_token()? {
                                            Token::Identifier(domain_name) => {
                                                match self.tokenizer.get_token()? {
                                                    Token::Punctuator(
                                                        PunctuationType::RParentheses,
                                                    ) => Ok(DefinitionType::Problem(
                                                        ProblemDefinition {
                                                            domain_name,
                                                            problem_name,
                                                        },
                                                    )),
                                                    token => {
                                                        let error = SyntacticError {
                                                            expected: format!("the block of the definition of problem '{}' is not closed with ')'", problem_name),
                                                            found: token.to_string(),
                                                            position: self.tokenizer.get_last_token_position(),
                                                        };
                                                        Err(ParsingError::Syntactic(error))
                                                    }
                                                }
                                            }
                                            token => {
                                                let error = SyntacticError {
                                                    expected: "domain name".to_string(),
                                                    found: token.to_string(),
                                                    position: self
                                                        .tokenizer
                                                        .get_last_token_position(),
                                                };
                                                Err(ParsingError::Syntactic(error))
                                            }
                                        }
                                    }
                                    token => {
                                        let error = SyntacticError {
                                            expected: "keyword 'domain'".to_string(),
                                            found: token.to_string(),
                                            position: self.tokenizer.get_last_token_position(),
                                        };
                                        Err(ParsingError::Syntactic(error))
                                    }
                                }
                            }
                            token => {
                                let error = SyntacticError {
                                    expected: "'('".to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                Err(ParsingError::Syntactic(error))
                            }
                        }
                    }
                    token => {
                        let error = SyntacticError {
                            expected: "')'".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        Err(ParsingError::Syntactic(error))
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "problem name".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }

    pub fn parse_requirements(&self) -> Result<Vec<RequirementType>, ParsingError> {
        let mut requirements = vec![];
        let mut finished = false;
        while !finished {
            match self.tokenizer.get_token()? {
                Token::Requirement(req) => {
                    requirements.push(req);
                }
                Token::Punctuator(PunctuationType::RParentheses) => {
                    finished = true;
                }
                token => {
                    let error = SyntacticError {
                        expected: "either a requirement or a ')'".to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            }
        }
        Ok(requirements)
    }
}
