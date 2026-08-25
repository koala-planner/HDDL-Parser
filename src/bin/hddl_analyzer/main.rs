use clap::Parser;
use hddl_analyzer::{Input, ParsingError, Transpiler};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

mod cli_args;

use cli_args::{CLIArgs, Commands, ConvertArgs, InputArgs, OutputFormat};

// ANSI escape color codes
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

// the raw input bytes; the parsed program borrows from them, so each command
// keeps this alive in its own frame for as long as the Transpiler is used
enum InputData {
    Json(String),
    Hddl {
        domain: Vec<u8>,
        problem: Option<Vec<u8>>,
    },
}

impl InputData {
    fn read(input: &InputArgs) -> Result<InputData, String> {
        let extension = Path::new(&input.input_path)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_lowercase());
        match extension.as_deref() {
            Some("json") => {
                if input.problem_path.is_some() {
                    return Err(
                        "a JSON input already contains everything; --problem-path only applies to HDDL input"
                            .to_string(),
                    );
                }
                let json = fs::read_to_string(&input.input_path).map_err(|err| err.to_string())?;
                Ok(InputData::Json(json))
            }
            Some("hddl") => {
                let domain = fs::read(&input.input_path).map_err(|err| err.to_string())?;
                let problem = match &input.problem_path {
                    Some(path) => Some(fs::read(path).map_err(|err| err.to_string())?),
                    None => None,
                };
                Ok(InputData::Hddl { domain, problem })
            }
            Some(other) => Err(format!(
                "unrecognized input extension '.{other}' (expected .hddl or .json)"
            )),
            None => Err("the input file has no extension; cannot infer its format".to_string()),
        }
    }

    fn transpiler(&self) -> Result<Transpiler<'_>, ParsingError> {
        Transpiler::from_input(match self {
            InputData::Json(json) => Input::Json(json),
            InputData::Hddl { domain, problem } => Input::Hddl {
                domain,
                problem: problem.as_ref(),
            },
        })
    }
}

pub fn main() {
    let args = CLIArgs::parse();
    match args.command {
        Commands::Convert(args) => convert(args),
        Commands::Verify(input) => verify(input),
        Commands::Metadata(input) => metadata(input),
        Commands::Format(input) => format_files(input),
    }
}

fn convert(args: ConvertArgs) {
    let data = match InputData::read(&args.input) {
        Ok(data) => data,
        Err(error) => return eprintln!("{RED}[Error]{RESET} {error}"),
    };
    let transpiler = match data.transpiler() {
        Ok(transpiler) => transpiler,
        Err(parsing_error) => return eprintln!("{RED}[Error]{RESET} {parsing_error}"),
    };
    let mut transpiler = transpiler;
    for transformation in args.transform.iter() {
        transpiler = match transpiler.transform(transformation.clone()) {
            Ok(transpiler) => transpiler,
            Err(error) => return eprintln!("{RED}[Error]{RESET} {error}"),
        };
    }
    match args.to {
        OutputFormat::Json => write_or_print(args.output_file.as_deref(), &transpiler.to_json()),
        OutputFormat::Hddl => {
            let (domain, problem) = transpiler.to_hddl();
            match (args.output_file.as_deref(), problem) {
                (Some(path), Some(problem)) => {
                    write_file(path, &domain);
                    write_file(&problem_sibling(path), &problem);
                }
                (Some(path), None) => write_file(path, &domain),
                (None, problem) => {
                    println!("{GREEN}[Ok]{RESET}");
                    println!("{domain}");
                    if let Some(problem) = problem {
                        println!();
                        println!("{problem}");
                    }
                }
            }
        }
    }
}

fn verify(input: InputArgs) {
    let data = match InputData::read(&input) {
        Ok(data) => data,
        Err(error) => return eprintln!("{RED}[Error]{RESET} {error}"),
    };
    match data.transpiler().and_then(|transpiler| transpiler.verify()) {
        Ok(warnings) => {
            for warning in warnings {
                println!("{YELLOW}[Warning]{RESET} {warning}");
            }
            println!("{GREEN}[Ok]{RESET}");
        }
        Err(parsing_error) => eprintln!("{RED}[Error]{RESET} {parsing_error}"),
    }
}

fn metadata(input: InputArgs) {
    let data = match InputData::read(&input) {
        Ok(data) => data,
        Err(error) => return eprintln!("{RED}[Error]{RESET} {error}"),
    };
    match data
        .transpiler()
        .and_then(|transpiler| transpiler.metadata())
    {
        Ok(result) => print!("{result}"),
        Err(parsing_error) => eprintln!("{RED}[Error]{RESET} {parsing_error}"),
    }
}

// parses the input and writes it back pretty-printed
fn format_files(input: InputArgs) {
    let data = match InputData::read(&input) {
        Ok(data) => data,
        Err(error) => return eprintln!("{RED}[Error]{RESET} {error}"),
    };
    let transpiler = match data.transpiler() {
        Ok(transpiler) => transpiler,
        Err(parsing_error) => return eprintln!("{RED}[Error]{RESET} {parsing_error}"),
    };
    match &data {
        InputData::Json(_) => {
            return eprintln!(
                "{RED}[Error]{RESET} {}",
                "this is only supported for HDDL files."
            )
        }
        InputData::Hddl { .. } => {
            let (domain, problem) = transpiler.to_hddl();
            write_file(&input.input_path, &domain);
            if let Some(problem) = problem {
                write_file(input.problem_path.as_deref().unwrap(), &problem);
            }
        }
    }
}

fn write_or_print(output_file: Option<&str>, content: &str) {
    match output_file {
        None => {
            println!("{GREEN}[Ok]{RESET}");
            println!("{content}");
        }
        Some(path) => write_file(path, content),
    }
}

fn write_file(path: &str, content: &str) {
    let mut output_path = env::current_dir().unwrap();
    output_path.push(path);
    match fs::write(&output_path, content) {
        Ok(_) => println!("Result successfully written to {}", output_path.display()),
        Err(err) => eprintln!("{RED}[Error]{RESET} {err}"),
    }
}

// "out.hddl" -> "out.problem.hddl"; "out" -> "out.problem"
fn problem_sibling(path: &str) -> String {
    let extension = PathBuf::from(path)
        .extension()
        .and_then(|ext| ext.to_str().map(|ext| ext.to_string()));
    match extension {
        Some(extension) => {
            let stem = &path[..path.len() - extension.len() - 1];
            format!("{stem}.problem.{extension}")
        }
        None => format!("{path}.problem"),
    }
}
