use super::*;

fn assert_error(
    result: Result<Token, LexicalError>,
    error_type: LexicalErrorType,
    lexeme: &str,
    (line, column, length): (u32, u32, u32),
) {
    let error = result.expect_err("error not detected");
    assert_eq!(error.error_type, error_type);
    assert_eq!(error.lexeme, lexeme);
    assert_eq!(
        error.position,
        TokenPosition {
            line,
            column,
            length
        }
    );
}

#[test]
pub fn variable_name_error_test() {
    // peeking reports the error without consuming it
    let program = String::from("\n\n?ca<sd ").into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let error = LexicalErrorType::InvalidIdentifier;
    assert_error(lexer.lookahead(), error, "?ca<sd", (2, 0, 6));
    assert_error(lexer.get_token(), error, "?ca<sd", (2, 0, 6));

    // lexing continues after a bad token; a tab at the default width of 4
    // advances to the next tab stop
    let program = String::from("\n\n?rt/asd \n\n\n\n ?f*ta \t %x954s ? ").into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    assert_error(lexer.get_token(), error, "?rt/asd", (2, 0, 7));
    assert_error(lexer.get_token(), error, "?f*ta", (6, 1, 5));
    assert_error(lexer.get_token(), error, "%x954s", (6, 9, 6));

    let program = String::from("\n\n\n\n\n\n \t %x954s ? ").into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    assert_error(lexer.get_token(), error, "%x954s", (6, 5, 6));
    // with a width of 1 a tab is a single column, as in LSP positions
    let lexer = LexicalAnalyzer::with_tab_width(&program, 1);
    assert_error(lexer.get_token(), error, "%x954s", (6, 3, 6));
    // a wider tab still lands on a tab stop rather than advancing by a fixed amount
    let lexer = LexicalAnalyzer::with_tab_width(&program, 8);
    assert_error(lexer.get_token(), error, "%x954s", (6, 9, 6));

    // messages print positions 1-based with an inclusive end column.
    let program = String::from("(pred_5 ?x%y ;; trailing comment").into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    lexer.get_token().unwrap();
    lexer.get_token().unwrap();
    let error = lexer.get_token().unwrap_err();
    assert_eq!(
        error.position,
        TokenPosition {
            line: 0,
            column: 8,
            length: 4
        }
    );
    assert_eq!(
        error.to_string(),
        "[1:9-12]: '?x%y' is an invalid identifier."
    );
    assert_eq!(lexer.get_token().unwrap(), Token::EOF);
    assert_eq!(
        lexer.get_last_token_position(),
        TokenPosition {
            line: 0,
            column: 32,
            length: 0
        }
    );
}

#[test]
pub fn keyword_error_test() {
    // an invalid keyword is reported from its ':', which the lexeme excludes;
    // the last one ends the file without a trailing newline
    let program = String::from("\n\n:cra :pred \n\n\n\n :defne").into_bytes();
    let lexer = LexicalAnalyzer::new(&program);
    let error = LexicalErrorType::InvalidKeyword;
    assert_error(lexer.get_token(), error, "cra", (2, 0, 4));
    assert_error(lexer.get_token(), error, "pred", (2, 5, 5));
    assert_error(lexer.get_token(), error, "defne", (6, 1, 6));
    assert_eq!(lexer.get_token().unwrap(), Token::EOF);
    assert_eq!(
        lexer.get_last_token_position(),
        TokenPosition {
            line: 6,
            column: 7,
            length: 0
        }
    );
}
