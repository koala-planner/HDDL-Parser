use super::*;

impl<'a> Parser<'a> {
    pub fn parse_formula(&'a self) -> Result<Formula, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::RParentheses) => {
                return Ok(Formula::Empty);
            }
            Token::Punctuator(PunctuationType::LParentheses) => {
                match self.tokenizer.get_token()? {
                    // Not Operation
                    Token::Operator(OperationType::Not) => {
                        let formula = self.parse_formula()?;
                        match self.tokenizer.get_token()? {
                            Token::Punctuator(PunctuationType::RParentheses) => {
                                return Ok(Formula::Not(Box::new(formula)));
                            }
                            token => {
                                let error = SyntacticError {
                                    expected: "closing the not operator with ')'".to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                return Err(ParsingError::Syntactic(error));
                            }
                        }
                    }
                    // And Connector
                    Token::Operator(OperationType::And) => {
                        let mut expressions = vec![];
                        loop {
                            let formula = self.parse_formula()?;
                            if let Formula::Empty = formula {
                                return Ok(Formula::And(expressions));
                            } else {
                                expressions.push(Box::new(formula));
                            }
                        }
                    }
                    // Xor Connector
                    Token::Operator(OperationType::Xor) => {
                        let mut expressions = vec![];
                        loop {
                            let formula = self.parse_formula()?;
                            if let Formula::Empty = formula {
                                return Ok(Formula::Xor(expressions));
                            } else {
                                expressions.push(Box::new(formula));
                            }
                        }
                    }
                    // Or Connector
                    Token::Operator(OperationType::Or) => {
                        let mut expressions = vec![];
                        loop {
                            let formula = self.parse_formula()?;
                            if let Formula::Empty = formula {
                                return Ok(Formula::Or(expressions));
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
                                    return Ok(Formula::Equals(p1, p2));
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "equality's closing parenthesis".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            },
                            token => {
                                let error = SyntacticError {
                                    expected: "right hand side of the equality".to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                return Err(ParsingError::Syntactic(error));
                            }
                        },
                        token => {
                            let error = SyntacticError {
                                expected: "left hand side of the equality".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    },
                    // Universal Quantifier
                    Token::Operator(OperationType::ForAll) => match self.tokenizer.get_token()? {
                        Token::Punctuator(PunctuationType::LParentheses) => {
                            let params = self.parse_args()?;
                            let expression = Box::new(self.parse_formula()?);
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    return Ok(Formula::ForAll(params, expression));
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "')' to close the forall statement".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "'(' after forall keyword".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    },
                    // Existential Quantifier
                    Token::Operator(OperationType::Exists) => match self.tokenizer.get_token()? {
                        Token::Punctuator(PunctuationType::LParentheses) => {
                            let params = self.parse_args()?;
                            let expression = Box::new(self.parse_formula()?);
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    return Ok(Formula::Exists(params, expression));
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "')' to close the existential statement".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "'(' after existential quantification keyword".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    },
                    // Stochastic Operations (represented as the And of Weighted branches)
                    Token::Operator(OperationType::Probabilistic) => {
                        let mut branches = vec![];
                        loop {
                            match self.tokenizer.get_token()? {
                                // closing paren ends the block
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    return Ok(Formula::And(branches));
                                }
                                // a probability weight, immediately followed by its effect
                                Token::Number(weight) => {
                                    let f = self.parse_formula()?;
                                    if let Formula::Empty = f {
                                        let error = SyntacticError {
                                            expected: "a formula following the probability".to_string(),
                                            found: ")".to_string(),
                                            position: self.tokenizer.get_last_token_position(),
                                        };
                                        return Err(ParsingError::Syntactic(error));
                                    }
                                    branches.push(Box::new(Formula::Weighted(weight, Box::new(f))));
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "a probability or ')' to close the probabilistic block".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                    }
                    // Single Atom
                    Token::Identifier(name) => {
                        let predicate = Predicate {
                            name: name,
                            name_pos: self.tokenizer.get_last_token_position(),
                            variables: self.parse_args()?,
                        };
                        return Ok(Formula::Atom(predicate));
                    }
                    Token::Punctuator(PunctuationType::RParentheses) => {
                        return Ok(Formula::Empty);
                    }
                    token => {
                        let error = SyntacticError {
                            expected: "a boolean formula".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "a (potentially empty) boolean formula definition".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
    }
}
