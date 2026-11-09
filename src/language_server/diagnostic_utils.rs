use crate::ParsingError;
use crate::SemanticErrorType;
use crate::lexical_analyzer::{LexicalAnalyzer, TokenPosition};
use crate::semantic_analyzer::DomainSemanticAnalyzer;
use crate::semantic_analyzer::ProblemSemanticAnalyzer;
use crate::syntactic_analyzer::AbstractSyntaxTree;
use crate::syntactic_analyzer::FileVariant;
use crate::syntactic_analyzer::Parser;

use tower_lsp::lsp_types::DocumentDiagnosticReportResult;
use tower_lsp::lsp_types::RelatedFullDocumentDiagnosticReport;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use tower_lsp::lsp_types::{DiagnosticRelatedInformation, Location, Url};
use tower_lsp::lsp_types::{DocumentDiagnosticReport, FullDocumentDiagnosticReport};

pub fn classify_file(content: &Vec<u8>) -> FileVariant {
    let lexer = LexicalAnalyzer::with_tab_width(content, LSP_TAB_WIDTH);
    let parser = Parser::new(lexer);
    parser.classify()
}

pub fn diagnose_domain(content: &Vec<u8>) -> DocumentDiagnosticReportResult {
    let lexer = LexicalAnalyzer::with_tab_width(content, LSP_TAB_WIDTH);
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
        }
        Err(paring_error) => {
            items.push(paring_error.into());
        }
    }
    DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
        RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                items,
                result_id: None,
            },
        },
    ))
}

pub fn generate_empty_report() -> DocumentDiagnosticReportResult {
    DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
        RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                items: vec![],
                result_id: None,
            },
        },
    ))
}

pub fn diagnose_problem(
    domain_content: Option<&Vec<u8>>,
    problem_content: &Vec<u8>,
) -> DocumentDiagnosticReportResult {
    let mut items = vec![];
    let mut symbol_table = None;

    let domain_lexer;
    let domain_parser;
    let domain_verifier;
    let domain_ast;
    if let Some(content) = domain_content {
        domain_lexer = LexicalAnalyzer::with_tab_width(content, LSP_TAB_WIDTH);
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
    let problem_lexer = LexicalAnalyzer::with_tab_width(problem_content, LSP_TAB_WIDTH);
    match Parser::new(problem_lexer).parse() {
        Ok(ast) if symbol_table.is_some() => {
            if let AbstractSyntaxTree::Problem(p_ast) = ast {
                let semantic_verifier = ProblemSemanticAnalyzer::new(&p_ast, symbol_table.unwrap());
                match semantic_verifier.verify_problem() {
                    Ok(_warnings) => {
                        // TODO: add warnings
                    }
                    Err(semantic_error) => {
                        items.push(ParsingError::Semantic(semantic_error).into());
                    }
                }
            }
        }
        Ok(_) => {}
        Err(parsing_error) => {
            items.push(parsing_error.into());
        }
    }
    DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
        RelatedFullDocumentDiagnosticReport {
            // TODO: fix related documentes
            related_documents: None,
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                items,
                result_id: None,
            },
        },
    ))
}

// LSP positions count a tab as a single character (the client renders it), so
// columns must not expand tabs
const LSP_TAB_WIDTH: u32 = 1;

// the range a token occupies. a zero-length position (end of file,
// synthesized nodes) still gets a one-character range so the client has
// something to underline
fn range(position: TokenPosition) -> Range {
    Range {
        start: Position::new(position.line, position.column),
        end: Position::new(position.line, position.column + position.length.max(1)),
    }
}

// a whole-line range for errors that have no position of their own
fn lines(start: u32, end: u32) -> Range {
    Range {
        start: Position::new(start, 0),
        end: Position::new(end, 0),
    }
}

impl From<ParsingError> for Diagnostic {
    fn from(error: ParsingError) -> Self {
        let source = Some("HDDL Analyzer".to_string());
        let message = error.to_string();
        let mut severity = DiagnosticSeverity::ERROR;
        let mut related_information = None;
        let range = match error {
            ParsingError::Lexiacal(lexical_error) => range(lexical_error.position),
            ParsingError::Syntactic(syntactic_error) => range(syntactic_error.position),
            ParsingError::JSON(json_error) => range(json_error.position),
            ParsingError::Transformation(_) => lines(0, 1),
            ParsingError::Semantic(semantic_error) => match semantic_error {
                // Duplicate Errors
                SemanticErrorType::DuplicateObjectDeclaration(duplicate)
                | SemanticErrorType::DuplicatePredicateDeclaration(duplicate)
                | SemanticErrorType::DuplicateFunctionDeclaration(duplicate)
                | SemanticErrorType::DuplicateActionDeclaration(duplicate)
                | SemanticErrorType::DuplicateCompoundTaskDeclaration(duplicate)
                | SemanticErrorType::DuplicateMethodDeclaration(duplicate)
                | SemanticErrorType::DuplicateParameterDeclaration(duplicate) => {
                    related_information = Some(vec![DiagnosticRelatedInformation {
                        location: Location {
                            // TODO: fix the dummy URI
                            uri: Url::parse("//").unwrap(),
                            range: range(duplicate.first_pos),
                        },
                        message: message.clone(),
                    }]);
                    range(duplicate.second_pos)
                }
                SemanticErrorType::DuplicateRequirementDeclaration(_) => {
                    // TODO: fix this dummy range
                    severity = DiagnosticSeverity::INFORMATION;
                    lines(0, 1)
                }
                // Undefined Entities
                SemanticErrorType::UndefinedPredicate(undefined)
                | SemanticErrorType::UndefinedFunction(undefined)
                | SemanticErrorType::UndefinedType(undefined)
                | SemanticErrorType::UndefinedSubtask(undefined)
                | SemanticErrorType::UndefinedTask(undefined)
                | SemanticErrorType::UndefinedParameter(undefined)
                | SemanticErrorType::UndefinedObject(undefined) => range(undefined.position),
                // Inconsistency Error
                SemanticErrorType::InconsistentPredicateArity(arity_error)
                | SemanticErrorType::InconsistentFunctionArity(arity_error)
                | SemanticErrorType::InconsistentTaskArity(arity_error) => {
                    range(arity_error.position)
                }
                SemanticErrorType::InconsistentPredicateArgType(type_error)
                | SemanticErrorType::InconsistentFunctionArgType(type_error)
                | SemanticErrorType::InconsistentTaskArgType(type_error) => {
                    range(type_error.position)
                }
                // Ordering Errors
                // TODO: fix the dummy range
                SemanticErrorType::CyclicTypeDeclaration => lines(0, 10),
                SemanticErrorType::CyclicOrderingDeclaration(pos) => range(pos),
            },
        };
        Diagnostic::new(
            range,
            Some(severity),
            None,
            source,
            message,
            related_information,
            None,
        )
    }
}
