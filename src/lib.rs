mod lexical_analyzer;
mod output;
mod semantic_analyzer;
mod syntactic_analyzer;
mod transpiler;

mod language_server;
pub use language_server::RequestHandler;
pub use transpiler::{Input, Transformation, Transpiler};

pub use crate::lexical_analyzer::*;
pub use output::*;
use semantic_analyzer::*;
use serde::{Deserialize, Serialize};
pub use syntactic_analyzer::*;

#[derive(Serialize, Deserialize)]
pub struct HDDLProgram<'a> {
    #[serde(borrow)]
    pub domain: DomainAST<'a>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<ProblemAST<'a>>,
}

impl<'a> HDDLProgram<'a> {
    pub const HTN_TOP_TASK: &'static str = "htn_top";
    pub const HTN_TOP_METHOD: &'static str = "htn_top_method";

    pub fn from_hddl(
        domain: &'a Vec<u8>,
        problem: Option<&'a Vec<u8>>,
    ) -> Result<HDDLProgram<'a>, ParsingError> {
        let lexer = LexicalAnalyzer::new(domain);
        let domain_parser = syntactic_analyzer::Parser::new(lexer);
        let domain_ast = match domain_parser.parse()? {
            AbstractSyntaxTree::Domain(d) => d,
            _ => panic!("expected domain, found problem"),
        };
        let problem_ast = match problem {
            Some(p) => {
                let lexer = LexicalAnalyzer::new(p);
                let problem_parser = syntactic_analyzer::Parser::new(lexer);
                match problem_parser.parse()? {
                    AbstractSyntaxTree::Problem(p_ast) => Some(p_ast),
                    _ => panic!("expected problem, found domain"),
                }
            }
            None => None,
        };
        Ok(HDDLProgram {
            domain: domain_ast,
            problem: problem_ast,
        })
    }

    pub fn from_json(json: &'a str) -> Result<HDDLProgram<'a>, ParsingError> {
        let json = json.strip_prefix('\u{feff}').unwrap_or(json);
        Ok(serde_json::from_str(json)?)
    }

    pub fn verify(&self) -> Result<Vec<WarningType>, ParsingError> {
        let domain_semantic_verifier = DomainSemanticAnalyzer::new(&self.domain);
        let symbol_table = domain_semantic_verifier.verify_domain()?;
        match &self.problem {
            Some(p_ast) => {
                let problem_semantic_verifier = ProblemSemanticAnalyzer::new(p_ast, symbol_table);
                let warnings = problem_semantic_verifier.verify_problem()?;
                Ok(warnings)
            }
            None => Ok(symbol_table.warnings),
        }
    }

    pub fn metadata(&self) -> Result<MetaData, ParsingError> {
        let tdg = Tdg::new(&self.domain);
        let nullables = tdg.compute_nullables();
        let recursion_type = tdg.classify_cycles(&nullables);
        Ok(MetaData {
            recursion: recursion_type,
            nullables: nullables.iter().map(|x| x.to_string()).collect(),
            domain_name: self.domain.name.clone(),
            n_actions: self.domain.actions.len() as u32,
            n_tasks: self.domain.compound_tasks.len() as u32,
            n_methods: self.domain.methods.len() as u32,
        })
    }
}
