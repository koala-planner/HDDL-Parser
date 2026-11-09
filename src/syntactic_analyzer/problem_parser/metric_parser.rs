use super::*;

impl<'a> Parser<'a> {
    // the :metric block after its keyword: a direction and the expression to
    // optimise, closed by ')'
    pub fn parse_metric(&self) -> Result<Metric<'a>, ParsingError> {
        let metric = match self.tokenizer.get_token()? {
            Token::Identifier("minimize") => Metric::Minimize(self.parse_numeric_expr()?),
            Token::Identifier("maximize") => Metric::Maximize(self.parse_numeric_expr()?),
            token => {
                let error = SyntacticError {
                    expected: "'minimize' or 'maximize'".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                return Err(ParsingError::Syntactic(error));
            }
        };
        match self.tokenizer.get_token()? {
            Token::Punctuator(PunctuationType::RParentheses) => Ok(metric),
            token => {
                let error = SyntacticError {
                    expected: "')' to close the metric".to_string(),
                    found: token.to_string(),
                    position: self.tokenizer.get_last_token_position(),
                };
                Err(ParsingError::Syntactic(error))
            }
        }
    }
}
