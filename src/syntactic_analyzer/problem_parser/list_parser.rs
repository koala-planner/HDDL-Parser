use super::*;

impl<'a> Parser<'a> {
    pub fn parse_args(&self) -> Result<Vec<Symbol<'a>>, ParsingError> {
        let mut objects = vec![];
        let mut result = vec![];
        let mut token = self.tokenizer.get_token()?;
        loop {
            while let Token::Identifier(symbol) = token {
                objects.push((symbol, self.tokenizer.get_last_token_position()));
                token = self.tokenizer.get_token()?;
            }
            match token {
                Token::Punctuator(PunctuationType::Dash) => {
                    // match type
                    let object_type = self.tokenizer.get_token()?;
                    let type_pos = self.tokenizer.get_last_token_position();
                    match object_type {
                        Token::Identifier(t) => {
                            for (o, obj_pos) in objects {
                                result.push(Symbol::new(o, obj_pos, Some(t), Some(type_pos)));
                            }
                            objects = vec![];
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "The type of objects".to_string(),
                                found: token.to_string(),
                                position: type_pos,
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    }
                    token = self.tokenizer.get_token()?;
                }
                Token::Punctuator(PunctuationType::RParentheses) => {
                    for (object, object_pos) in objects {
                        result.push(Symbol::new(object, object_pos, None, None));
                    }
                    return Ok(result);
                }
                token => {
                    let error = SyntacticError {
                        expected: "an identifier".to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            }
        }
    }
}
