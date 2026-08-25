use super::*;

impl<'a> Parser<'a> {
    pub fn parse_action(&self) -> Result<Action<'a>, ParsingError> {
        let task = self.parse_task()?;
        let mut preconditions = None;
        let mut effects = None;
        // Parse Preconditions
        match self.tokenizer.lookahead()? {
            Token::Keyword(KeywordName::Precondition) => {
                // skip precondition keyword
                let _ = self.tokenizer.get_token();
                preconditions = Some(self.parse_formula()?);
            }
            // the action has no precondition
            Token::Keyword(KeywordName::Effect)
            | Token::Punctuator(PunctuationType::RParentheses) => {}
            // undefined sequenec
            token => {
                let error = SyntacticError {
                    expected: format!("(potentially empty) preconditions of {}", task.name)
                        .to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
        // Parse Effects
        match self.tokenizer.lookahead()? {
            Token::Keyword(KeywordName::Effect) => {
                // skip effects keyword
                let _ = self.tokenizer.get_token();
                let formula = self.parse_formula()?;
                effects = Some(formula);
            }
            // action has no effects
            Token::Punctuator(PunctuationType::RParentheses) => {}
            token => {
                let error = SyntacticError {
                    expected: format!("(potentially empty) effects of {}", task.name).to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }
        // skip action block's closing parantheses
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::RParentheses) => {}
            token => {
                let error = SyntacticError {
                    expected: format!("closing the scope of {} using ')'", task.name).to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        }

        Ok(Action {
            name: task.name,
            name_pos: task.name_pos,
            parameters: task.parameters,
            preconditions,
            effects,
        })
    }
}
