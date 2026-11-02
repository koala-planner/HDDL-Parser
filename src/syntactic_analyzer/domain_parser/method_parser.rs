use super::*;

impl<'a> Parser<'a> {
    pub fn parse_method(&self) -> Result<Method<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Identifier(method_name) => {
                let name = Symbol::new(
                    method_name,
                    self.tokenizer.get_last_token_position(),
                    None,
                    None,
                );
                match self.tokenizer.get_token()? {
                    Token::Keyword(KeywordName::Parameters) => {
                        match self.tokenizer.get_token()? {
                            Token::Punctuator(PunctuationType::LParentheses) => {
                                let params = self.parse_args()?;
                                match self.tokenizer.get_token()? {
                                    Token::Keyword(KeywordName::Task) => {
                                        match self.tokenizer.get_token()? {
                                            Token::Punctuator(PunctuationType::LParentheses) => {
                                                match self.tokenizer.get_token()? {
                                                    Token::Identifier(task_name) => {
                                                        let task = Symbol::new(
                                                            task_name,
                                                            self.tokenizer
                                                                .get_last_token_position(),
                                                            None,
                                                            None,
                                                        );
                                                        let terms = self.parse_args()?;
                                                        match self.tokenizer.lookahead()? {
                                                            Token::Keyword(
                                                                KeywordName::Precondition,
                                                            ) => {
                                                                // skip "precondition" keyword
                                                                let _ = self.tokenizer.get_token();
                                                                let precondition =
                                                                    self.parse_formula()?;
                                                                let tn = self.parse_htn()?;
                                                                return Ok(Method {
                                                                    name,
                                                                    params,
                                                                    task,
                                                                    task_terms: terms,
                                                                    precondition: Some(
                                                                        precondition,
                                                                    ),
                                                                    tn,
                                                                });
                                                            }
                                                            Token::Keyword(
                                                                KeywordName::Subtasks,
                                                            )
                                                            | Token::Keyword(
                                                                KeywordName::OrderedSubtasks,
                                                            ) => {
                                                                let tn = self.parse_htn()?;
                                                                return Ok(Method {
                                                                    name,
                                                                    params,
                                                                    task,
                                                                    task_terms: terms,
                                                                    precondition: None,
                                                                    tn,
                                                                });
                                                            }
                                                            token => {
                                                                let error = SyntacticError {
                                                            expected: format!(
                                                                "Either preconditions for {} or its decomposition",
                                                                method_name
                                                            )
                                                            .to_string(),
                                                            found: token.to_string(),
                                                            position: self.tokenizer.get_last_token_position(),
                                                        };
                                                                return Err(
                                                                    ParsingError::Syntactic(error),
                                                                );
                                                            }
                                                        }
                                                    }
                                                    token => {
                                                        let error = SyntacticError {
                                                            expected: format!("The task that method {} decomposes", method_name).to_string(),
                                                            found: token.to_string(),
                                                            position: self.tokenizer.get_last_token_position(),
                                                        };
                                                        return Err(ParsingError::Syntactic(error));
                                                    }
                                                }
                                            }
                                            token => {
                                                let error = SyntacticError {
                                                    expected: "'(' after keyword :task".to_string(),
                                                    found: token.to_string(),
                                                    position: self
                                                        .tokenizer
                                                        .get_last_token_position(),
                                                };
                                                return Err(ParsingError::Syntactic(error));
                                            }
                                        }
                                    }
                                    token => {
                                        let error = SyntacticError {
                                            expected: "keyword :task".to_string(),
                                            found: token.to_string(),
                                            position: self.tokenizer.get_last_token_position(),
                                        };
                                        return Err(ParsingError::Syntactic(error));
                                    }
                                }
                            }
                            token => {
                                let error = SyntacticError {
                                    expected: "'(' after keyword :parameters".to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                return Err(ParsingError::Syntactic(error));
                            }
                        }
                    }
                    token => {
                        let error = SyntacticError {
                            expected: format!("The parameters of method {} ", method_name)
                                .to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "method name".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
    }
}
