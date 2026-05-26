use super::*;

#[cfg(test)]
mod lexer_test {
    use super::*;
    #[test]
    pub fn punctuation_recognition_test() {
        let program = String::from("-( \n) ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Ok(Token::Punctuator(PunctuationType::Dash)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Punctuator(PunctuationType::LParentheses)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Punctuator(PunctuationType::RParentheses)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::EOF) => {},
            _ => panic!("wrong token")
        }
    }

    #[test]
    pub fn ordering_relation_recognition_test() {
        let program = String::from("<=  \n> >= < \n").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::LessThanOrEqual)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::GreaterThan)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::GreaterThanOrEqual)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::LessThan)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::EOF) => {},
            _ => panic!("wrong token")
        }
    }

    #[test]
    pub fn logical_operator_recognition_test() {
        let program = String::from("and or oneof not exists forall imply\n").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::And)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::Or)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::Xor)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::Not)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::Exists)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::ForAll)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Operator(OperationType::Implication)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::EOF) => {},
            _ => panic!("wrong token")
        }
    }

    #[test]
    pub fn variable_recognition_test() {
        let program = String::from("?test_id ?pred-aa ").into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Ok(Token::Identifier(x)) => {
                assert_eq!(x, &String::from("test_id"))
            },
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Identifier(x)) => {
                assert_eq!(x, &String::from("pred-aa"))
            },
            _ => panic!("wrong token")
        }
    }

    #[test]
    pub fn keyword_recognition_test() {
        let program = String::from(
            "define domain problem :requirements :objects :types :constants\n
            :predicates :init :htn :action :parameters :method :precondition\n
            :effect :subtasks :tasks :ordered-tasks :ordered-subtasks :order\n
            :ordering :constraints\n"
        ).into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Define)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Domain)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Problem)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Requirements)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Objects)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Types)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Constants)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Predicates)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Init)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::HTN)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Action)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Parameters)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Method)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Precondition)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Effect)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Subtasks)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Subtasks)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::OrderedSubtasks)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::OrderedSubtasks)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Ordering)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Ordering)) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Keyword(KeywordName::Constraints)) => {},
            _ => panic!("wrong token")
        }
    }

    #[test]
    pub fn number_recognition_test() {
        let program = String::from(
            "123.54 42\n"
        ).into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Ok(Token::Number(NumberType::Real(x))) => {assert_eq!(x, 123.54)},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Number(NumberType::Integer(x))) => {assert_eq!(x, 42)},
            _ => panic!("wrong token")
        }
    }

    #[test]
    pub fn identifier_recognition_test() {
        let program = String::from(
            "var123 var_3123 te23 v\n"
        ).into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        match lexer.get_token() {
            Ok(Token::Identifier("var123")) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Identifier("var_3123")) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Identifier("te23")) => {},
            _ => panic!("wrong token")
        }
        match lexer.get_token() {
            Ok(Token::Identifier("v")) => {},
            _ => panic!("wrong token")
        }
    }

    #[test]
    pub fn lookahead_test() {
        let program = String::from(
            "(:method vDC_to_vPC_2
                :parameters (?d1 ?d2 - AbstractDevice ?t - SignalType ?p1 ?p2 - Port)
                :task (ValidateDeviceConnection ?d1 ?d2 ?t)
                :precondition (and
                        (isPartOf ?p1 ?d1)
                        (isPartOf ?p2 ?d2)
                        (isPlugDirection ?p1 out)
                        (isPlugDirection ?p2 both)
                        (isSignalSource ?p1 ?t)
                        (isSignalDestination ?p2 ?t)
                )
                :subtasks (and
                    (ValidatePortConnection ?p1 ?p2 ?t)
                )
            ) "
        ).into_bytes();
        let lexer = LexicalAnalyzer::new(&program);
        loop {
            let peek = lexer.lookahead();
            let actual = lexer.get_token();
            match actual {
                Ok(Token::EOF) => {break;}
                _ => {}
            }
            assert_eq!(peek.unwrap(), actual.unwrap());
        }
    }
}