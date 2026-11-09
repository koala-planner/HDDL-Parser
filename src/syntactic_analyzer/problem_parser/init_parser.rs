use super::*;

impl<'a> Parser<'a> {
    // the :init block after its keyword: ground atoms mixed with the initial
    // values of fluents, (= fluent number)
    pub fn parse_init(&self) -> Result<(Vec<Predicate<'a>>, Vec<FluentValue<'a>>), ParsingError> {
        let mut atoms = vec![];
        let mut values = vec![];
        loop {
            match self.tokenizer.get_token()? {
                Token::Punctuator(PunctuationType::RParentheses) => return Ok((atoms, values)),
                Token::Punctuator(PunctuationType::LParentheses) => {
                    match self.tokenizer.get_token()? {
                        Token::Identifier(name) => atoms.push(Predicate {
                            name,
                            name_pos: self.tokenizer.get_last_token_position(),
                            variables: self.parse_args()?,
                        }),
                        Token::Operator(OperationType::Equal) => {
                            let fluent = self.parse_fluent()?;
                            let value = match self.tokenizer.get_token()? {
                                Token::Number(number) => number,
                                token => {
                                    let error = SyntacticError {
                                        expected: "a number as the initial value of the fluent"
                                            .to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            };
                            match self.tokenizer.get_token()? {
                                Token::Punctuator(PunctuationType::RParentheses) => {
                                    values.push(FluentValue { fluent, value })
                                }
                                token => {
                                    let error = SyntacticError {
                                        expected: "')' to close the initial value".to_string(),
                                        found: token.to_string(),
                                        position: self.tokenizer.get_last_token_position(),
                                    };
                                    return Err(ParsingError::Syntactic(error));
                                }
                            }
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "a predicate name or '=' for a fluent's initial value"
                                    .to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            return Err(ParsingError::Syntactic(error));
                        }
                    }
                }
                token => {
                    let error = SyntacticError {
                        expected:
                            "an atom, a fluent's initial value, or ')' to close the :init block"
                                .to_string(),
                        found: token.to_string(),
                        position: self.tokenizer.get_last_token_position(),
                    };
                    return Err(ParsingError::Syntactic(error));
                }
            }
        }
    }
}
