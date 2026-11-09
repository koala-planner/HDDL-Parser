use super::*;

impl<'a> Parser<'a> {
    pub fn parse_precondition(&self) -> Result<Precondition<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::RParentheses) => Ok(Precondition::Empty),
            Token::Punctuator(PunctuationType::LParentheses) => {
                match self.tokenizer.get_token()? {
                    // Not Operation
                    Token::Operator(OperationType::Not) => {
                        let formula = self.parse_precondition()?;
                        match self.tokenizer.get_token()? {
                            Token::Punctuator(PunctuationType::RParentheses) => {
                                Ok(Precondition::Not(Box::new(formula)))
                            }
                            token => {
                                let error = SyntacticError {
                                    expected: "closing the not operator with ')'".to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                Err(ParsingError::Syntactic(error))
                            }
                        }
                    }
                    // And Connector
                    Token::Operator(OperationType::And) => {
                        let mut expressions = vec![];
                        loop {
                            let formula = self.parse_precondition()?;
                            if let Precondition::Empty = formula {
                                return Ok(Precondition::And(expressions));
                            } else {
                                expressions.push(Box::new(formula));
                            }
                        }
                    }
                    // Xor Connector
                    Token::Operator(OperationType::Xor) => {
                        let mut expressions = vec![];
                        loop {
                            let formula = self.parse_precondition()?;
                            if let Precondition::Empty = formula {
                                return Ok(Precondition::Xor(expressions));
                            } else {
                                expressions.push(Box::new(formula));
                            }
                        }
                    }
                    // Or Connector
                    Token::Operator(OperationType::Or) => {
                        let mut expressions = vec![];
                        loop {
                            let formula = self.parse_precondition()?;
                            if let Precondition::Empty = formula {
                                return Ok(Precondition::Or(expressions));
                            } else {
                                expressions.push(Box::new(formula));
                            }
                        }
                    }
                    // Equality
                    Token::Operator(OperationType::Equal) => match self.tokenizer.get_token()? {
                        Token::Identifier(p1) => match self.tokenizer.get_token()? {
                            Token::Identifier(p2) => match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    Ok(Precondition::Equals(p1, p2))
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "equality's closing parenthesis".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    Err(ParsingError::Syntactic(error))
                                }
                            },
                            token => {
                                let error = SyntacticError {
                                    expected: "right hand side of the equality".to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                Err(ParsingError::Syntactic(error))
                            }
                        },
                        token => {
                            let error = SyntacticError {
                                expected: "left hand side of the equality".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            Err(ParsingError::Syntactic(error))
                        }
                    },
                    // Implication
                    Token::Operator(OperationType::Implication) => {
                        let lhs = self.parse_precondition()?;
                        let rhs = self.parse_precondition()?;
                        match self.tokenizer.get_token()? {
                            Token::Punctuator(PunctuationType::RParentheses) => {
                                Ok(Precondition::Imply(Box::new(lhs), Box::new(rhs)))
                            }
                            token => {
                                let error = SyntacticError {
                                    expected: "')' to close the implication".to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                Err(ParsingError::Syntactic(error))
                            }
                        }
                    }
                    // Universal Quantifier
                    Token::Operator(OperationType::ForAll) => match self.tokenizer.get_token()? {
                        Token::Punctuator(PunctuationType::LParentheses) => {
                            let params = self.parse_args()?;
                            let expression = Box::new(self.parse_precondition()?);
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    Ok(Precondition::ForAll(params, expression))
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "')' to close the forall statement".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    Err(ParsingError::Syntactic(error))
                                }
                            }
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "'(' after forall keyword".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            Err(ParsingError::Syntactic(error))
                        }
                    },
                    // Existential Quantifier
                    Token::Operator(OperationType::Exists) => match self.tokenizer.get_token()? {
                        Token::Punctuator(PunctuationType::LParentheses) => {
                            let params = self.parse_args()?;
                            let expression = Box::new(self.parse_precondition()?);
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    Ok(Precondition::Exists(params, expression))
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "')' to close the existential statement"
                                            .to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    Err(ParsingError::Syntactic(error))
                                }
                            }
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "'(' after existential quantification keyword"
                                    .to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            Err(ParsingError::Syntactic(error))
                        }
                    },
                    // Single Atom
                    Token::Identifier(name) => {
                        let predicate = Predicate {
                            name,
                            name_pos: self.tokenizer.get_last_token_position(),
                            variables: self.parse_args()?,
                        };
                        Ok(Precondition::Atom(predicate))
                    }
                    Token::Punctuator(PunctuationType::RParentheses) => Ok(Precondition::Empty),
                    token => {
                        let error = SyntacticError {
                            expected: "a boolean formula".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        Err(ParsingError::Syntactic(error))
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "a (potentially empty) boolean formula definition".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }
}
