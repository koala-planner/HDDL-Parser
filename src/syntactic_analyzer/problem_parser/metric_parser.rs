use super::*;

impl<'a> Parser<'a> {
    // the :metric block after its keyword: a direction and the expression to
    // optimise, closed by ')'
    pub fn parse_metric(&self) -> Result<Metric<'a>, ParsingError> {
        let metric = match self.tokenizer.get_token()? {
            // makespan is written like a parameterless fluent but is not one
            Token::Identifier("minimize") => match self.parse_numeric_expr()? {
                NumericExpr::Fluent(fluent)
                    if fluent.name == "makespan" && fluent.variables.is_empty() =>
                {
                    Metric::Makespan
                }
                expression => Metric::Minimize(expression),
            },
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
