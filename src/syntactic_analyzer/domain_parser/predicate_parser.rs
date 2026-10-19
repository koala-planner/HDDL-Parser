use super::*;

impl<'a> Parser<'a> {
    pub fn parse_predicates(&self) -> Result<Vec<Predicate<'a>>, ParsingError> {
        let mut finished = false;
        let mut predicates = vec![];
        while !finished {
            match self.tokenizer.get_token()? {
                Token::Punctuator(PunctuationType::LParentheses) => {
                    let predicate = self.parse_predicate_definition()?;
                    predicates.push(predicate);
                }
                Token::Punctuator(PunctuationType::RParentheses) => {
                    finished = true;
                }
                token => {
                    let error = SyntacticError {
                        expected: "predicate definition".to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            }
        }
        Ok(predicates)
    }

    // parses a SINGLE predicate definition
    fn parse_predicate_definition(&self) -> Result<Predicate<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Identifier(predicate_name) => Ok(Predicate {
                name: predicate_name,
                name_pos: self.tokenizer.get_last_token_position(),
                variables: self.parse_args()?,
            }),
            token => {
                let error = SyntacticError {
                    expected: "a predicate name".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }
}
