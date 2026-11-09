use super::*;

impl<'a> Parser<'a> {
    pub fn parse_problem(
        &self,
        meta_data: ProblemDefinition<'a>,
    ) -> Result<ProblemAST<'a>, ParsingError> {
        let mut syntax_tree = ProblemAST::new(meta_data.problem_name, meta_data.domain_name);
        loop {
            match self.tokenizer.get_token()? {
                Token::Punctuator(PunctuationType::LParentheses) => {
                    // match declaration type
                    match self.tokenizer.get_token()? {
                        // requirement declaration
                        Token::Keyword(KeywordName::Requirements) => {
                            let requirements = self.parse_requirements()?;
                            for requirement in requirements {
                                syntax_tree.add_requirement(requirement);
                            }
                        }
                        // objects declaration
                        Token::Keyword(KeywordName::Objects) => {
                            let objects = self.parse_args()?;
                            for object in objects {
                                match object.symbol_type {
                                    Some(t) => {
                                        syntax_tree.add_typed_object(
                                            object.name,
                                            object.name_pos,
                                            t,
                                            object.type_pos.unwrap(),
                                        );
                                    }
                                    None => {
                                        syntax_tree.add_object(object.name, object.name_pos);
                                    }
                                }
                            }
                        }
                        // initial task network declaration
                        Token::Keyword(KeywordName::HTN) => {
                            let init_tn = self.parse_initial_tn()?;
                            syntax_tree.add_init_tn(init_tn);
                        }
                        // goal state (optional)
                        Token::Keyword(KeywordName::Goal) => {
                            if let Token::Punctuator(PunctuationType::RParentheses) =
                                self.tokenizer.lookahead()?
                            {
                                self.tokenizer.get_token()?;
                            } else {
                                syntax_tree.add_goal(self.parse_precondition()?);
                                match self.tokenizer.get_token()? {
                                    Token::Punctuator(PunctuationType::RParentheses) => {}
                                    token => {
                                        let error = SyntacticError {
                                            expected: "')' to close the goal".to_string(),
                                            found: token.to_string(),
                                            position: self.tokenizer.get_last_token_position(),
                                        };
                                        return Err(ParsingError::Syntactic(error));
                                    }
                                }
                            }
                        }
                        // initial state: atoms and the initial values of fluents
                        Token::Keyword(KeywordName::Init) => {
                            let (atoms, values) = self.parse_init()?;
                            syntax_tree.init_state = atoms;
                            syntax_tree.init_values = (!values.is_empty()).then_some(values);
                        }
                        // plan metric (optional)
                        Token::Keyword(KeywordName::Metric) => {
                            let metric = self.parse_metric()?;
                            syntax_tree.metric = Some(metric);
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "a keyword for block definition".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    }
                }
                Token::EOF | Token::Punctuator(PunctuationType::RParentheses) => {
                    return Ok(syntax_tree);
                }
                err => {
                    panic!("unexpected token {:?}", err)
                }
            }
        }
    }
}
