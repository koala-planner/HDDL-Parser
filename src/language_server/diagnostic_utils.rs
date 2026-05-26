use crate::lexical_analyzer::LexicalAnalyzer;
use crate::semantic_analyzer::DomainSemanticAnalyzer;
use crate::semantic_analyzer::ProblemSemanticAnalyzer;
use crate::syntactic_analyzer::AbstractSyntaxTree;
use crate::syntactic_analyzer::FileVariant;
use crate::syntactic_analyzer::Parser;
use crate::ParsingError;
use crate::SemanticErrorType;

use tower_lsp::lsp_types::DocumentDiagnosticReportResult;
use tower_lsp::lsp_types::{FullDocumentDiagnosticReport, DocumentDiagnosticReport};
use tower_lsp::lsp_types::RelatedFullDocumentDiagnosticReport;
use tower_lsp::lsp_types::{DiagnosticRelatedInformation, Location, Url};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};


pub fn classify_file(content: &Vec<u8>) -> FileVariant {
    let lexer = LexicalAnalyzer::new(content);
    let parser = Parser::new(lexer);
    parser.classify()
}

pub fn diagnose_domain(content: &Vec<u8>) -> DocumentDiagnosticReportResult {
    let lexer = LexicalAnalyzer::new(content);
    let parser = Parser::new(lexer);
    let mut items = vec![];
    match parser.parse() {
        Ok(ast) => {
            if let AbstractSyntaxTree::Domain(d_ast) = ast {
                match DomainSemanticAnalyzer::new(&d_ast).verify_domain() {
                    Ok(_) => {
                        // TODO: add warnings
                    }
                    Err(semantic_error) => {
                        items.push(ParsingError::Semantic(semantic_error).into());
                    }
                }
            }
        },
        Err(paring_error) => {
            items.push(paring_error.into());
        }
    }
    DocumentDiagnosticReportResult::Report(
        DocumentDiagnosticReport::Full(
            RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report:
                FullDocumentDiagnosticReport {
                    items: items,
                    result_id: None,
                },
            },
        ),
    )
}

pub fn generate_empty_report() -> DocumentDiagnosticReportResult {
    DocumentDiagnosticReportResult::Report(
        DocumentDiagnosticReport::Full(
            RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report:
                FullDocumentDiagnosticReport {
                    items: vec![],
                    result_id: None,
                },
            },
        ),
    )
}

pub fn diagnose_problem(domain_content: Option<&Vec<u8>>, problem_content: &Vec<u8>) -> DocumentDiagnosticReportResult {
    let mut items = vec![];
    let mut symbol_table = None;
    
    let domain_lexer;
    let domain_parser;
    let domain_verifier;
    let domain_ast;
    match domain_content {
        Some(content) => {
            domain_lexer = LexicalAnalyzer::new(content);
            domain_parser = Parser::new(domain_lexer);
            match domain_parser.parse() {
                Ok(d_ast) => {
                    if let AbstractSyntaxTree::Domain(d) = d_ast {
                        domain_ast = d;
                        domain_verifier = DomainSemanticAnalyzer::new(&domain_ast);
                        match domain_verifier.verify_domain() {
                            Ok(symbols) => {
                                symbol_table = Some(symbols);
                                // TODO: add warnings
                            }
                            Err(semantic_error) => {
                                items.push(ParsingError::Semantic(semantic_error).into());
                            }
                        }
                    }
                }
                Err(parsing_error) => {
                    items.push(parsing_error.into());
                }
            }
        }
        None => {}
    }
    let problem_lexer = LexicalAnalyzer::new(problem_content);
    match Parser::new(problem_lexer).parse() {
        Ok(ast) if symbol_table.is_some() => {
            if let AbstractSyntaxTree::Problem(p_ast) = ast {
                let semantic_verifier = ProblemSemanticAnalyzer::new(
                    &p_ast, 
                    symbol_table.unwrap()
                );
                match semantic_verifier.verify_problem() {
                    Ok(_warnings) => {
                        // TODO: add warnings
                    },
                    Err(semantic_error) => {
                        items.push(ParsingError::Semantic(semantic_error).into());
                    }
                }
            }
        },
        Ok(_) => {}
        Err(parsing_error) => {
            items.push(parsing_error.into());
        }
    }
    DocumentDiagnosticReportResult::Report(
        DocumentDiagnosticReport::Full(
            RelatedFullDocumentDiagnosticReport {
                // TODO: fix related documentes
                related_documents: None,
                full_document_diagnostic_report:
                FullDocumentDiagnosticReport {
                    items,
                    result_id: None,
                },
            },
        ),
    )
}


impl From<ParsingError> for Diagnostic {
    fn from(error: ParsingError) -> Self {
        let source = Some("HDDL Analyzer".to_string());
        match error {
            ParsingError::Lexiacal(lexical_error) => {
                let line_start = lexical_error.position.line;
                Diagnostic::new(
                    Range {
                        start: Position::new(line_start - 1, 0),
                        end: Position::new(line_start, 0),
                    },
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    source,
                    lexical_error.to_string(),
                    None,
                    None,
                )
            }
            ParsingError::Syntactic(syntactic_error) => {
                let line_start = syntactic_error.position.line;
                Diagnostic::new(
                    Range {
                        start: Position::new(line_start - 1, 0),
                        end: Position::new(line_start, 0),
                    },
                    Some(DiagnosticSeverity::ERROR),
                    None,
                    source,
                    syntactic_error.to_string(),
                    None,
                    None,
                )
            }
            ParsingError::Semantic(semantic_error) => {
                match semantic_error {
                    // Duplicate Errors
                    SemanticErrorType::DuplicateObjectDeclaration(ref duplicate)
                    | SemanticErrorType::DuplicatePredicateDeclaration(ref duplicate)
                    | SemanticErrorType::DuplicateActionDeclaration(ref duplicate)
                    | SemanticErrorType::DuplicateCompoundTaskDeclaration(ref duplicate)
                    | SemanticErrorType::DuplicateMethodDeclaration(ref duplicate)
                    | SemanticErrorType::DuplicateParameterDeclaration(ref duplicate) => {
                        Diagnostic::new(
                            Range {
                                start: Position { line: duplicate.second_pos.line - 1, character: 0 },
                                end: Position { line: duplicate.second_pos.line, character: 0 }
                            },
                            Some(DiagnosticSeverity::ERROR), 
                            None, 
                            source, 
                            semantic_error.to_string(), 
                            Some(vec![
                                DiagnosticRelatedInformation {
                                    location: Location {
                                        // TODO: fix the dummy URI
                                        uri: Url::parse("//").unwrap(),
                                        range: Range {
                                            start: Position { line: duplicate.first_pos.line - 1, character: 0 },
                                            end: Position { line: duplicate.first_pos.line, character: 0 }
                                        }
                                    },
                                    message: semantic_error.to_string()
                                }
                            ]), 
                            None
                        )
                    }
                    SemanticErrorType::DuplicateRequirementDeclaration(_) => {
                        // TODO: fix this dummy range
                        Diagnostic::new(
                            Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 1, character: 0 }
                            },
                            Some(DiagnosticSeverity::INFORMATION), 
                            None, 
                            source, 
                            semantic_error.to_string(), 
                            None, 
                            None
                        )
                    }
                    // Undefined Entities
                    SemanticErrorType::UndefinedPredicate(ref undefined)
                    | SemanticErrorType::UndefinedType(ref undefined)
                    | SemanticErrorType::UndefinedSubtask(ref undefined)
                    | SemanticErrorType::UndefinedTask(ref undefined)
                    | SemanticErrorType::UndefinedParameter(ref undefined)
                    | SemanticErrorType::UndefinedObject(ref undefined) => {
                        Diagnostic::new(
                            Range {
                                start: Position { line: undefined.position.line - 1, character: 0 },
                                end: Position { line: undefined.position.line, character: 0 }
                            },
                            Some(DiagnosticSeverity::ERROR), 
                            None, 
                            source, 
                            semantic_error.to_string(), 
                            None, 
                            None
                        )
                    }
                    // Inconsistency Error
                    SemanticErrorType::InconsistentPredicateArity(ref arity_error)
                    | SemanticErrorType::InconsistentTaskArity(ref arity_error) => {
                        Diagnostic::new(
                            Range {
                                start: Position { line: arity_error.position.line - 1, character: 0 },
                                end: Position { line: arity_error.position.line, character: 0 }
                            },
                            Some(DiagnosticSeverity::ERROR), 
                            None, 
                            source, 
                            semantic_error.to_string(), 
                            None, 
                            None
                        )
                    }
                    SemanticErrorType::InconsistentPredicateArgType(ref type_error)
                    | SemanticErrorType::InconsistentTaskArgType(ref type_error) => {
                        Diagnostic::new(
                            Range {
                                start: Position { line: type_error.position.line - 1, character: 0 },
                                end: Position { line: type_error.position.line, character: 0 }
                            },
                            Some(DiagnosticSeverity::ERROR), 
                            None, 
                            source, 
                            semantic_error.to_string(), 
                            None, 
                            None
                        )
                    }
                    // Ordering Errors
                    SemanticErrorType::CyclicTypeDeclaration => {
                        // TODO: fix the dummy range
                        Diagnostic::new(
                            Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 10, character: 0 }
                            },
                            Some(DiagnosticSeverity::ERROR), 
                            None, 
                            source, 
                            semantic_error.to_string(), 
                            None, 
                            None
                        )
                    }
                    SemanticErrorType::CyclicOrderingDeclaration(pos) => {
                        Diagnostic::new(
                            Range {
                                start: Position { line: pos.line - 1, character: 0 },
                                end: Position { line: pos.line, character: 0 }
                            },
                            Some(DiagnosticSeverity::ERROR), 
                            None, 
                            source, 
                            semantic_error.to_string(), 
                            None, 
                            None
                        )
                    }
                }
            }
        }
    }
}
