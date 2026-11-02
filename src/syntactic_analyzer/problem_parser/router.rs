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
                            let goal = self.parse_formula()?;
                            syntax_tree.add_goal(goal)
                        }
                        // initial state
                        Token::Keyword(KeywordName::Init) => {
                            let init_state = self.parse_predicates()?;
                            syntax_tree.add_init_state(init_state)
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
