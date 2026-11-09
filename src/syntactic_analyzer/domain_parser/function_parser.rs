use super::*;

impl<'a> Parser<'a> {
    pub fn parse_functions(&self) -> Result<Vec<Function<'a>>, ParsingError> {
        let mut finished = false;
        let mut functions = vec![];
        while !finished {
            match self.tokenizer.get_token()? {
                Token::Punctuator(PunctuationType::LParentheses) => {
                    let function = self.parse_function()?;
                    functions.push(function);
                }
                // an optional result type for the preceding functions; every
                // function is numeric, so only `number` is accepted and
                // nothing is recorded
                Token::Punctuator(PunctuationType::Dash) => match self.tokenizer.get_token()? {
                    Token::Identifier("number") => {}
                    token => {
                        let error = SyntacticError {
                            expected: "'number' as the type of the function".to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        return Err(ParsingError::Syntactic(error));
                    }
                },
                Token::Punctuator(PunctuationType::RParentheses) => {
                    finished = true;
                }
                token => {
                    let error = SyntacticError {
                        expected: "function definition".to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            }
        }
        Ok(functions)
    }

    // parses a function name with its arguments, after the opening '(': a
    // declaration in :functions, or a fluent in a numeric effect
    pub fn parse_function(&self) -> Result<Function<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Identifier(function_name) => Ok(Function {
                name: function_name,
                name_pos: self.tokenizer.get_last_token_position(),
                variables: self.parse_args()?,
            }),
            token => {
                let error = SyntacticError {
                    expected: "a function name".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }
}
