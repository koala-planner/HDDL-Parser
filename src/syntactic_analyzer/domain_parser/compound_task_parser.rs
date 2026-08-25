use super::*;

impl<'a> Parser<'a> {
    pub fn parse_task(&self) -> Result<Task<'a>, ParsingError> {
        match self.tokenizer.get_token()? {
            Token::Identifier(task_name) => {
                let task_name_pos = self.tokenizer.get_last_token_position();
                match self.tokenizer.get_token()? {
                    Token::Keyword(KeywordName::Parameters) => match self.tokenizer.get_token()? {
                        Token::Punctuator(PunctuationType::LParentheses) => {
                            Ok(Task::new(task_name, task_name_pos, self.parse_args()?))
                        }
                        token => {
                            let error = SyntacticError {
                                expected: "'(' after :parameters".to_string(),
                                found: token.to_string(),
                                position: self.tokenizer.get_last_token_position(),
                            };
                            Err(ParsingError::Syntactic(error))
                        }
                    },
                    token => {
                        let error = SyntacticError {
                            expected: format!(
                                "a (potentially empty) list of parameters after defininig {}",
                                task_name
                            )
                            .to_string(),
                            found: token.to_string(),
                            position: self.tokenizer.get_last_token_position(),
                        };
                        Err(ParsingError::Syntactic(error))
                    }
                }
            }
            token => {
                let error = SyntacticError {
                    expected: "a task/action name (identifier)".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }
}
