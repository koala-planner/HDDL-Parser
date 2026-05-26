use super::*;

impl <'a> Parser<'a> {
    pub fn parse_domain(&'a self, domain_name: &'a str) -> Result<DomainAST<'a>, ParsingError> {
        let mut syntax_tree = DomainAST::new(domain_name.to_string());
        loop {
            match self.tokenizer.get_token()? {
                Token::Punctuator(PunctuationType::LParentheses) => {
                    match self.tokenizer.get_token()? {
                        // predicate definitions
                        Token::Keyword(KeywordName::Predicates) => {
                            let predicates = self.parse_predicates()?;
                            for predicate in predicates {
                                syntax_tree.add_predicate(predicate);
                            }
                        }
                        // function definitions
                        Token::Keyword(KeywordName::Functions) => {
                            let functions = self.parse_functions()?;
                            for function in functions {
                                syntax_tree.add_function(function);
                            }
                        }
                        // compund task definition
                        Token::Keyword(KeywordName::Task) => {
                            let task = self.parse_task()?;
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(
                                    PunctuationType::RParentheses,
                                ) => {
                                    syntax_tree.add_compound_task(task);
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: format!(
                                            "')' after definition of {}",
                                            task.name
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
                        // method definition
                        Token::Keyword(KeywordName::Method) => {
                            let method = self.parse_method()?;
                            syntax_tree.add_method(method);
                        }
                        // action definition
                        Token::Keyword(KeywordName::Action) => {
                            let action = self.parse_action()?;
                            syntax_tree.add_action(action);
                        }
                        // requirement declaration
                        Token::Keyword(KeywordName::Requirements) => {
                            let requirements = self.parse_requirements()?;
                            for requirement in requirements {
                                syntax_tree.add_requirement(requirement);
                            }
                        }
                        // type hierarchy declaration
                        Token::Keyword(KeywordName::Types) => {
                            let var_types = self.parse_args()?;
                            for var_type in var_types {
                                syntax_tree.add_var_type(var_type);
                            }
                        }
                        // constants declaration
                        Token::Keyword(KeywordName::Constants) => {
                            let constants = self.parse_args()?;
                            for constant in constants {
                                syntax_tree.add_constant(constant);
                            }
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "a keyword".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    }
                }
                Token::Punctuator(PunctuationType::RParentheses) => {
                    return Ok(syntax_tree);
                }
                token => {
                    let error = SyntacticError {
                        expected: format!("either ')' to close the definition of {}, or '(' to start defining new components", domain_name),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            }
        }
    }
}
