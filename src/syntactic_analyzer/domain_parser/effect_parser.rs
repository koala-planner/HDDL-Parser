use super::*;

impl<'a> Parser<'a> {
    pub fn parse_effect(&self) -> Result<Effect<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::RParentheses) => Ok(Effect::Empty),
            Token::Punctuator(PunctuationType::LParentheses) => {
                match self.tokenizer.get_token()? {
                    // Not Operation
                    Token::Operator(OperationType::Not) => {
                        let formula = self.parse_effect()?;
                        match self.tokenizer.get_token()? {
                            Token::Punctuator(PunctuationType::RParentheses) => {
                                Ok(Effect::Not(Box::new(formula)))
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
                            let formula = self.parse_effect()?;
                            if let Effect::Empty = formula {
                                return Ok(Effect::And(expressions));
                            } else {
                                expressions.push(Box::new(formula));
                            }
                        }
                    }
                    // Xor Connector
                    Token::Operator(OperationType::Xor) => {
                        let mut expressions = vec![];
                        loop {
                            let formula = self.parse_effect()?;
                            if let Effect::Empty = formula {
                                return Ok(Effect::Xor(expressions));
                            } else {
                                expressions.push(Box::new(formula));
                            }
                        }
                    }
                    // Or Connector
                    Token::Operator(OperationType::Or) => {
                        let mut expressions = vec![];
                        loop {
                            let formula = self.parse_effect()?;
                            if let Effect::Empty = formula {
                                return Ok(Effect::Or(expressions));
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
                                    Ok(Effect::Equals(p1, p2))
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
                    // Conditional effect: a precondition guards an effect
                    Token::Operator(OperationType::Implication) => {
                        let guard = self.parse_precondition()?;
                        let body = self.parse_effect()?;
                        match self.tokenizer.get_token()? {
                            Token::Punctuator(PunctuationType::RParentheses) => {
                                Ok(Effect::Imply(Box::new(guard), Box::new(body)))
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
                            let expression = Box::new(self.parse_effect()?);
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    Ok(Effect::ForAll(params, expression))
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
                            let expression = Box::new(self.parse_effect()?);
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    Ok(Effect::Exists(params, expression))
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
                    // Probabilistic effect: every probability is followed by its branch
                    Token::Operator(OperationType::Probabilistic) => {
                        let mut branches = vec![];
                        loop {
                            match self.tokenizer.get_token()? {
                                // closing paren ends the block
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    return Ok(Effect::Probabilistic(branches));
                                }
                                // a probability, immediately followed by its effect
                                Token::Number(probability) => {
                                    let value = match &probability {
                                        NumberType::Real(r) => *r,
                                        NumberType::Integer(i) => *i as f64,
                                    };
                                    if !(0.0..=1.0).contains(&value) {
                                        let error = SyntacticError {
                                            expected: "a probability in [0, 1]".to_string(),
                                            found: probability.to_string(),
                                            position: self.tokenizer.get_last_token_position(),
                                        };
                                        return Err(ParsingError::Syntactic(error));
                                    }
                                    let f = self.parse_effect()?;
                                    if let Effect::Empty = f {
                                        let error = SyntacticError {
                                            expected: "a formula following the probability"
                                                .to_string(),
                                            found: ")".to_string(),
                                            position: self.tokenizer.get_last_token_position(),
                                        };
                                        return Err(ParsingError::Syntactic(error));
                                    }
                                    branches.push((probability, Box::new(f)));
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected:
                                            "a probability or ')' to close the probabilistic block"
                                                .to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                    }
                    // Numeric effect: an update of a fluent by a numeric expression
                    Token::Operator(
                        operator @ (OperationType::Assign
                        | OperationType::Increase
                        | OperationType::Decrease
                        | OperationType::ScaleUp
                        | OperationType::ScaleDown),
                    ) => {
                        let operator = match operator {
                            OperationType::Assign => AssignmentOperator::Assign,
                            OperationType::Increase => AssignmentOperator::Increase,
                            OperationType::Decrease => AssignmentOperator::Decrease,
                            OperationType::ScaleUp => AssignmentOperator::ScaleUp,
                            _ => AssignmentOperator::ScaleDown,
                        };
                        let fluent = self.parse_fluent()?;
                        let operand = self.parse_numeric_expr()?;
                        match self.tokenizer.get_token()? {
                            Token::Punctuator(PunctuationType::RParentheses) => {
                                Ok(Effect::Numeric(operator, fluent, operand))
                            }
                            token => {
                                let error = SyntacticError {
                                    expected: format!("')' to close the {} effect", operator),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                Err(ParsingError::Syntactic(error))
                            }
                        }
                    }
                    // Single Atom
                    Token::Identifier(name) => {
                        let predicate = Predicate {
                            name,
                            name_pos: self.tokenizer.get_last_token_position(),
                            variables: self.parse_args()?,
                        };
                        Ok(Effect::Atom(predicate))
                    }
                    Token::Punctuator(PunctuationType::RParentheses) => Ok(Effect::Empty),
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

    // a fluent written as (name args...) or as a bare name
    pub fn parse_fluent(&self) -> Result<Function<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::LParentheses) => self.parse_function(),
            Token::Identifier(name) => Ok(Function {
                name,
                name_pos: self.tokenizer.get_last_token_position(),
                variables: vec![],
            }),
            token => {
                let error = SyntacticError {
                    expected: "a fluent".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }

    // a number, a fluent, or a sum or product of such terms
    pub fn parse_numeric_expr(&self) -> Result<NumericExpr<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Number(number) => Ok(NumericExpr::Number(number)),
            Token::Identifier(name) => Ok(NumericExpr::Fluent(Function {
                name,
                name_pos: self.tokenizer.get_last_token_position(),
                variables: vec![],
            })),
            Token::Punctuator(PunctuationType::LParentheses) => match self.tokenizer.lookahead()? {
                // (+ e1 e2 ...) or (* e1 e2 ...): every operand up to the closing parenthesis
                Token::Operator(
                    operator @ (OperationType::Addition | OperationType::Multiplication),
                ) => {
                    self.tokenizer.get_token()?;
                    let mut operands = vec![];
                    while !matches!(
                        self.tokenizer.lookahead()?,
                        Token::Punctuator(PunctuationType::RParentheses)
                    ) {
                        operands.push(self.parse_numeric_expr()?);
                    }
                    self.tokenizer.get_token()?;
                    if operands.len() < 2 {
                        let error = SyntacticError {
                            expected: format!("at least two operands for {}", operator),
                            found: ")".to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                    Ok(match operator {
                        OperationType::Addition => NumericExpr::Sum(operands),
                        _ => NumericExpr::Product(operands),
                    })
                }
                _ => Ok(NumericExpr::Fluent(self.parse_function()?)),
            },
            token => {
                let error = SyntacticError {
                    expected: "a number, a fluent, a sum or a product".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }
}
