use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn variable_name_error_test() {
        let program = String::from("\n\n?ca<sd ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Err(x) => match x.error_type {
                LexicalErrorType::InvalidIdentifier => {
                    assert_eq!(x.position.line, 3);
                    assert_eq!(x.lexeme, "?ca<sd");
                }
                _ => panic!("wrong error detected"),
            },
            _ => panic!("error not detected"),
        }
        let program = String::from("\n\n?rt/asd \n\n\n\n ?f*ta \t %x954s ? ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Err(x) => match x.error_type {
                LexicalErrorType::InvalidIdentifier => {
                    assert_eq!(x.position.line, 3);
                    assert_eq!(x.lexeme, "?rt/asd");
                }
                _ => panic!("wrong error detected"),
            },
            _ => panic!("error not detected"),
        }
        let program = String::from("\n\n\n\n\n\n ?f*ta \t %x954s ? ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Err(x) => match x.error_type {
                LexicalErrorType::InvalidIdentifier => {
                    assert_eq!(x.position.line, 7);
                    assert_eq!(x.lexeme, "?f*ta");
                }
                _ => panic!("wrong error detected"),
            },
            _ => panic!("error not detected"),
        }
        let program = String::from("\n\n\n\n\n\n \t %x954s ? ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Err(x) => match x.error_type {
                LexicalErrorType::InvalidIdentifier => {
                    assert_eq!(x.position.line, 7);
                    assert_eq!(x.lexeme, "%x954s");
                }
                _ => panic!("wrong error detected"),
            },
            _ => panic!("error not detected"),
        }
    }

    #[test]
    pub fn keyword_error_test() {
        let program = String::from("\n\n:cra :pred \n\n\n\n :defne ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Err(x) => match x.error_type {
                LexicalErrorType::InvalidKeyword => {
                    assert_eq!(x.position.line, 3);
                    assert_eq!(x.lexeme, "cra");
                }
                _ => panic!("wrong error detected"),
            },
            _ => panic!("error not detected"),
        }
        match lexer.get_token() {
            Err(x) => match x.error_type {
                LexicalErrorType::InvalidKeyword => {
                    assert_eq!(x.position.line, 3);
                    assert_eq!(x.lexeme, "pred");
                }
                _ => panic!("wrong error detected"),
            },
            _ => panic!("error not detected"),
        }
        match lexer.get_token() {
            Err(x) => match x.error_type {
                LexicalErrorType::InvalidKeyword => {
                    assert_eq!(x.position.line, 7);
                    assert_eq!(x.lexeme, "defne");
                }
                _ => panic!("wrong error detected"),
            },
            _ => panic!("error not detected"),
        }
    }
}
