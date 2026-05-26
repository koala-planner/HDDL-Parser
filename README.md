An open-source tool for validating planning models specified in the Hierarchical Domain Definition Language (HDDL)[^1] format based on the official HDDL syntax specification and concrete tests (including those pointing to *potential* issues) as proposed by Sleath and Bercher (2023)[^2]. HDDL Parser aims to help domain modelers catch bugs by providing strong type checking, error localization, and ensuring syntactic correctness.

# Detected Errors
In what follows, we provide the currently supported list of errors (for further details, see the paper by Sleath and Bercher (2023)[^2]). Once detected, actionable error messages in conjunction with the exact error location are returned to help modelers fix problems quickly.
* **Basic Syntax Issues**: Detects deviations from the HDDL grammar.
* **Inconsistent Parameter Usage**: Catches type mismatches and incorrect parameter counts in task/predicate usage.
* **Undefined Entities**: Identifies undefined predicates, types, objects, and tasks.
* **Duplicate Definitions**: Finds duplicate tasks, methods, predicates, and constants.
* **Cyclic Type Declarations**: Identifies cycles in the type hierarchy.
* **Undeclared Parameters**: Catches use of parameters not declared in tasks or methods.
* **Task Network Issues**: Detects cyclic ordering declaration.
* **Unrefinable Tasks**: Flags compound tasks that do not have a primitive refinement.

# Build Instruction
The following steps must be taken to compile the project. Wherever we mention "project_directory", we mean the root folder where the ```cargo.toml``` file is located.
1. This project was written in the Rust programming language and requires its compiler (and cargo package manager) to be built.
If you do not have it installed, follow the official installation guide ([link](https://www.rust-lang.org/tools/install)).
2. Open a terminal in the project_directory, and execute ```cargo build --release```.
3. If all steps are done successfully, the executable file can be located in ```/project_directory/target/release/hddl_analyzer.exe```. Notice that in this step and subsequent ones where we refer to the ```hddl_analyzer.exe``` file, the ".exe" part might be something else based on your operating system.

# Usage
The build process creates two binary files: The command line tool, ```hddl_analyzer.exe```, and the language server, ```language_server.exe```.
## Command Line
With ```hddl_analyzer.exe```, you can execute the following commands. 
* To verify a domain, use ```/path/to/hddl_analyzer.exe verify /path/to/domain.hddl```
* To verify a problem, use ```/path/to/hddl_analyzer.exe verify /path/to/domain.hddl -p /path/to/problem.hddl```
* To get general information about the domain (e.g., hierarchy class), use ```/path/to/hddl_analyzer.exe metadata /path/to/domain.hddl```
* For a complete list of commands, use ```/path/to/hddl_analyzer.exe --help```
## Language Server
Executing ```language_server.exe```, activates the language server on the current terminal. At the moment, we only support communication via the standard IO, specified by the Language Server Protocol (LSP).

# Correctness
This tool is validated against all 33 hierarchical domains in IPC 2023 (<a href="https://github.com/ipc2023-htn/ipc2023-domains">link</a>), and the Sleath-Bercher benchmark (<a href="https://github.com/ProfDrChaos/flawedPlanningModels/tree/main/HDDL">link</a>).

# Feedback
We welcome contributions and feedback from the planning community. The tool is designed to be extensible for adding new error checks as domain modeling best practices evolve. If you have spotted any bugs, please report them using this repository's "Issues" tab. Alternatively, you can email
- Mohammad Yousefi (<a href="mailto:mohammad.yousefi@anu.edu.au">mohammad.yousefi@anu.edu.au</a>) for technical enquiries, or
- Pascal Bercher (<a href="mailto:pascal.bercher@anu.edu.au">pascal.bercher@anu.edu.au</a>) for other purposes.

[^1]: Höller, Daniel, Gregor Behnke, Pascal Bercher, Susanne Biundo, Humbert Fiorino, Damien Pellier, and Ron Alford. "HDDL: An Extension to PDDL for Expressing Hierarchical Planning Problems". In Proceedings of the 34th Association for the Advancement of Artificial Intelligence (AAAI) Conference, 9883–9891. AAAI Press, 2020.

[^2]: Sleath, Kayleigh, and Pascal Bercher. "Detecting AI Planning Modelling Mistakes -- Potential Errors and Benchmark Domains". In Proceedings of the 20th Pacific Rim International Conference on Artificial Intelligence (PRICAI), 448–454. Springer, 2023.