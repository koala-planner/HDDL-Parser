use super::*;

impl<'a> Parser<'a> {
    pub fn parse_initial_tn(&self) -> Result<InitialTaskNetwork<'a>, ParsingError> {
        loop {
            match self.tokenizer.lookahead()? {
                Token::Keyword(KeywordName::Parameters) => {
                    let _ = self.tokenizer.get_token()?;
                    match self.tokenizer.get_token()? {
                        Token::Punctuator(PunctuationType::LParentheses) => {
                            return Ok(InitialTaskNetwork {
                                parameters: Some(self.parse_args()?),
                                tn: self.parse_htn()?,
                            });
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "'(' afer keyword :parameters".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    }
                }
                Token::Keyword(KeywordName::Subtasks)
                | Token::Keyword(KeywordName::OrderedSubtasks) => {
                    return Ok(InitialTaskNetwork {
                        parameters: None,
                        tn: self.parse_htn()?,
                    });
                }
                // empty init tn
                Token::Punctuator(PunctuationType::RParentheses) => {
                    return Ok(InitialTaskNetwork {
                        parameters: None,
                        tn: HTN {
                            subtasks: vec![],
                            ordering_pos: None,
                            orderings: TaskOrdering::Partial(vec![]),
                            constraints: None,
                        },
                    });
                }
                token => {
                    let error = SyntacticError {
                        expected: "expected the definition of the initial task network".to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            }
        }
    }

    pub fn parse_htn(&self) -> Result<HTN<'a>, ParsingError> {
        let mut subtasks = vec![];
        let mut orderings = vec![];
        let mut constraints = None;
        let mut ordering_pos = None;
        // check if it is an empty task network
        match self.tokenizer.lookahead()? {
            Token::Punctuator(PunctuationType::RParentheses) => {
                return Ok(HTN {
                    subtasks,
                    ordering_pos,
                    orderings: TaskOrdering::Partial(orderings),
                    constraints,
                });
            }
            _ => {}
        }
        match self.tokenizer.get_token()? {
            Token::Keyword(KeywordName::Subtasks) => {
                subtasks = self.parse_subtasks()?;
                loop {
                    match self.tokenizer.get_token()? {
                        Token::Keyword(KeywordName::Ordering) => {
                            ordering_pos = Some(self.tokenizer.get_last_token_position());
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::LParentheses) => {
                                    match self.tokenizer.get_token()? {
                                        Token::Operator(OperationType::And) => loop {
                                            match self.tokenizer.get_token()? {
                                                Token::Punctuator(
                                                    PunctuationType::LParentheses,
                                                ) => {
                                                    orderings
                                                        .extend(self.parse_ordering()?.into_iter());
                                                }
                                                Token::Punctuator(
                                                    PunctuationType::RParentheses,
                                                ) => {
                                                    break;
                                                }
                                                token => {
                                                    let error = SyntacticError {
                                                        expected: "'('".to_string(),
                                                        found: token.to_string(),
                                                        position: self
                                                            .tokenizer
                                                            .get_last_token_position(),
                                                    };
                                                    return Err(ParsingError::Syntactic(error));
                                                }
                                            }
                                        },
                                        Token::Operator(OperationType::LessThan) => {
                                            match self.tokenizer.get_token()? {
                                                Token::Identifier(t1) => loop {
                                                    match self.tokenizer.get_token()? {
                                                        Token::Identifier(t2) => {
                                                            orderings.push((t1, t2));
                                                        }
                                                        Token::Punctuator(
                                                            PunctuationType::RParentheses,
                                                        ) => {
                                                            break;
                                                        }
                                                        token => {
                                                            let error = SyntacticError {
                                                                expected: format!(
                                                                    "another task id after {}",
                                                                    t1
                                                                )
                                                                .to_string(),
                                                                found: token.to_string(),
                                                                position: self
                                                                    .tokenizer
                                                                    .get_last_token_position(),
                                                            };
                                                            return Err(ParsingError::Syntactic(
                                                                error,
                                                            ));
                                                        }
                                                    }
                                                },
                                                token => {
                                                    let error = SyntacticError {
                                                        expected: "expected a task identifier"
                                                            .to_string(),
                                                        found: token.to_string(),
                                                        position: self
                                                            .tokenizer
                                                            .get_last_token_position(),
                                                    };
                                                    return Err(ParsingError::Syntactic(error));
                                                }
                                            }
                                        }
                                        // no ordering
                                        Token::Punctuator(PunctuationType::RParentheses) => {}
                                        token => {
                                            let error = SyntacticError {
                                                expected: "ordering constraints".to_string(),
                                                found: token.to_string(),
                                                position: self.tokenizer.get_last_token_position(),
                                            };
                                            return Err(ParsingError::Syntactic(error));
                                        }
                                    }
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "'('".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                        Token::Keyword(KeywordName::Constraints) => {
                            constraints = Some(self.parse_constraints()?);
                        }
                        Token::Punctuator(PunctuationType::RParentheses) => {
                            return Ok(HTN {
                                subtasks,
                                ordering_pos,
                                orderings: TaskOrdering::Partial(orderings),
                                constraints,
                            });
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "the (potentially empty) ordering constraints of the task network".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    }
                }
            }
            Token::Keyword(KeywordName::OrderedSubtasks) => {
                subtasks = self.parse_subtasks()?;
                match self.tokenizer.get_token()? {
                    Token::Keyword(KeywordName::Constraints) => {
                        constraints = Some(self.parse_constraints()?);
                        return Ok(HTN {
                            subtasks,
                            ordering_pos,
                            orderings: TaskOrdering::Total,
                            constraints,
                        });
                    }
                    Token::Punctuator(PunctuationType::RParentheses) => {
                        return Ok(HTN {
                            subtasks,
                            ordering_pos,
                            orderings: TaskOrdering::Total,
                            constraints,
                        });
                    }
                    token => {
                        let error = SyntacticError {
                            expected: "closing ')' after task network definition".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: ":subtasks or :ordered-subtasks keyword".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
    }

    // parse a single ordering constraint
    fn parse_ordering(&self) -> Result<Vec<(&'a str, &'a str)>, ParsingError> {
        let mut orderings: Vec<(&str, &str)> = vec![];
        match self.tokenizer.get_token()? {
            Token::Operator(OperationType::LessThan) => match self.tokenizer.get_token()? {
                Token::Identifier(t1) => loop {
                    match self.tokenizer.get_token()? {
                        Token::Identifier(t2) => {
                            orderings.push((t1, t2));
                        }
                        Token::Punctuator(PunctuationType::RParentheses) => {
                            return Ok(orderings);
                        }
                        token => {
                            let error = SyntacticError {
                                expected: format!("the task ids that come after {}", t1)
                                    .to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    }
                },
                token => {
                    let error = SyntacticError {
                        expected: "task identifier".to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            },
            token => {
                let error = SyntacticError {
                    expected: "character '<' to start an ordering constraint".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
    }

    fn parse_subtasks(&self) -> Result<Vec<Subtask<'a>>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::LParentheses) => {
                match self.tokenizer.lookahead()? {
                    Token::Operator(OperationType::And) => {
                        // skip '('
                        let _ = self.tokenizer.get_token()?;
                        let mut subtasks = vec![];
                        loop {
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    return Ok(subtasks);
                                }
                                Token::Punctuator(PunctuationType::LParentheses) => {
                                    subtasks.push(self.parse_subtask()?);
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "subtask declarations".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                    }
                    // one subtask
                    Token::Identifier(_) => {
                        return Ok(vec![self.parse_subtask()?]);
                    }
                    // no subtasks
                    Token::Punctuator(PunctuationType::RParentheses) => {
                        // consume ')'
                        let _ = self.tokenizer.get_token()?;
                        return Ok(vec![]);
                    }
                    token => {
                        let error = SyntacticError {
                            expected: "subtask declarations".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "'('".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
    }

    // parses a single subtask
    fn parse_subtask(&self) -> Result<Subtask<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Identifier(id) => {
                let id_symbol =
                    Symbol::new(id, self.tokenizer.get_last_token_position(), None, None);
                let mut terms = vec![];
                match self.tokenizer.get_token()? {
                    Token::Punctuator(PunctuationType::LParentheses) => {
                        match self.tokenizer.get_token()? {
                            Token::Identifier(task) => {
                                let task_symbol = Symbol::new(
                                    task,
                                    self.tokenizer.get_last_token_position(),
                                    None,
                                    None,
                                );
                                loop {
                                    match self.tokenizer.get_token()? {
                                        Token::Identifier(term) => {
                                            terms.push(Symbol::new(
                                                term,
                                                self.tokenizer.get_last_token_position(),
                                                None,
                                                None,
                                            ));
                                        }
                                        Token::Punctuator(PunctuationType::RParentheses) => {
                                            match self.tokenizer.get_token()? {
                                                Token::Punctuator(
                                                    PunctuationType::RParentheses,
                                                ) => {
                                                    return Ok(Subtask {
                                                        id: Some(id_symbol),
                                                        task: task_symbol,
                                                        terms: terms,
                                                    });
                                                }
                                                token => {
                                                    let error = SyntacticError {
                                                        expected: format!(
                                                            "')' to close the block of {}",
                                                            task
                                                        )
                                                        .to_string(),
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
                                                expected: "either a ')' or an identifier"
                                                    .to_string(),
                                                found: token.to_string(),
                                                position: self.tokenizer.get_last_token_position(),
                                            };
                                            return Err(ParsingError::Syntactic(error));
                                        }
                                    }
                                }
                            }
                            token => {
                                let error = SyntacticError {
                                    expected: format!("a subtask name for {}!=...", id).to_string(),
                                    found: token.to_string(),
                                    position: self.tokenizer.get_last_token_position(),
                                };
                                return Err(ParsingError::Syntactic(error));
                            }
                        }
                    }
                    Token::Identifier(term) => {
                        terms.push(Symbol::new(
                            term,
                            self.tokenizer.get_last_token_position(),
                            None,
                            None,
                        ));
                        loop {
                            match self.tokenizer.get_token()? {
                                Token::Identifier(term) => {
                                    terms.push(Symbol::new(
                                        term,
                                        self.tokenizer.get_last_token_position(),
                                        None,
                                        None,
                                    ));
                                }
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    return Ok(Subtask {
                                        id: None,
                                        task: id_symbol,
                                        terms: terms,
                                    })
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: format!("either a term for {}, or ')'", term)
                                            .to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                    }
                    Token::Punctuator(PunctuationType::RParentheses) => {
                        return Ok(Subtask {
                            id: None,
                            task: id_symbol,
                            terms: terms,
                        })
                    }
                    token => {
                        let error = SyntacticError {
                            expected: "subtask definition".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "task id".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
    }

    pub fn parse_constraints(&self) -> Result<Vec<Constraint<'a>>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::LParentheses) => {
                let mut constraints = vec![];
                match self.tokenizer.lookahead()? {
                    Token::Punctuator(PunctuationType::RParentheses) => {
                        // skip lookahead
                        let _ = self.tokenizer.get_token();
                        return Ok(constraints);
                    }
                    // mutiple constrait declarations
                    Token::Operator(OperationType::And) => loop {
                        // skip lookahead
                        let _ = self.tokenizer.get_token();
                        // parse each constraint
                        loop {
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::LParentheses) => {
                                    constraints.push(self.parse_constraint()?);
                                }
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    return Ok(constraints);
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "a constraint definition".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                    },
                    // single constraint declaration
                    Token::Operator(OperationType::Not) | Token::Operator(OperationType::Equal) => {
                        constraints.push(self.parse_constraint()?);
                        return Ok(constraints);
                    }
                    token => {
                        let error = SyntacticError {
                            expected: "constraint declerations".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "'('".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
    }

    pub fn parse_constraint(&self) -> Result<Constraint<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Operator(OperationType::Not) => match self.tokenizer.get_token()? {
                Token::Punctuator(PunctuationType::LParentheses) => {
                    match self.tokenizer.get_token()? {
                        Token::Operator(OperationType::Equal) => {
                            match self.tokenizer.get_token()? {
                                Token::Identifier(t1) => match self.tokenizer.get_token()? {
                                    Token::Identifier(t2) => {
                                        match self.tokenizer.get_token()? {
                                            Token::Punctuator(PunctuationType::RParentheses) => {
                                                match self.tokenizer.get_token()? {
                                                    Token::Punctuator(
                                                        PunctuationType::RParentheses,
                                                    ) => {
                                                        return Ok(Constraint::NotEqual(t1, t2));
                                                    }
                                                    token => {
                                                        let error = SyntacticError{
                                                                    expected: format!(") to close the inequality constraint").to_string(),
                                                                    found: token.to_string(),
                                                                    position: self.tokenizer.get_last_token_position(),
                                                                };
                                                        return Err(ParsingError::Syntactic(error));
                                                    }
                                                }
                                            }
                                            token => {
                                                let error = SyntacticError {
                                                    expected: format!(
                                                        ") to close the inequality constraint"
                                                    )
                                                    .to_string(),
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
                                            expected: format!("right hand side of {}!=...", t1)
                                                .to_string(),
                                            found: token.to_string(),
                                            position: self.tokenizer.get_last_token_position(),
                                        };
                                        return Err(ParsingError::Syntactic(error));
                                    }
                                },
                                token => {
                                    let error = SyntacticError {
                                        expected: "task identifier".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "equality keyword '='".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    }
                }
                token => {
                    let error = SyntacticError {
                        expected: "'(' after keyword 'not'".to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            },
            Token::Operator(OperationType::Equal) => match self.tokenizer.get_token()? {
                Token::Identifier(t1) => match self.tokenizer.get_token()? {
                    Token::Identifier(t2) => match self.tokenizer.get_token()? {
                        Token::Punctuator(PunctuationType::RParentheses) => {
                            return Ok(Constraint::Equal(t1, t2));
                        }
                        token => {
                            let error = SyntacticError {
                                expected: format!(") to close the equality constraint").to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    },
                    token => {
                        let error = SyntacticError {
                            expected: format!("right hand side of {}=...", t1).to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                },
                token => {
                    let error = SyntacticError {
                        expected: "a task identifier".to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            },
            token => {
                let error = SyntacticError {
                    expected: "either an equalilty or non-equality constraint".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
    }
}
