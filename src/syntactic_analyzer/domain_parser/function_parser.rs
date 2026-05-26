use super::*;

impl<'a> Parser<'a> {
    pub fn parse_functions(&'a self) -> Result<Vec<Function<'a>>, ParsingError> {
        let mut finished = false;
        let mut functions = vec![];
        while !finished {
            match self.tokenizer.get_token()? {
                Token::Punctuator(PunctuationType::LParentheses) => {
                    let function = self.parse_function_definition()?;
                    functions.push(function);
                }
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

    // parses a SINGLE function definition
    fn parse_function_definition(&'a self) -> Result<Function<'a>, ParsingError> {
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