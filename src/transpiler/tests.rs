use core::panic;
use std::{assert_eq, vec};

use super::{Input, Transpiler};
use crate::{
    AbstractSyntaxTree, Action, AssignmentOperator, Effect, HDDLProgram, LexicalAnalyzer,
    NumericExpr, Parser,
    Precondition::{self, Atom},
    Predicate, RequirementType, Subtask, Symbol, TokenPosition,
};

fn symbol(name: &'static str) -> Symbol<'static> {
    Symbol::new(name, TokenPosition { line: 0 }, None, None)
}

fn predicate(name: &'static str, vars: Vec<&'static str>) -> Predicate<'static> {
    Predicate::new(
        name,
        TokenPosition { line: 0 },
        vars.into_iter().map(symbol).collect(),
    )
}

fn atom(name: &'static str, vars: Vec<&'static str>) -> Box<Precondition<'static>> {
    Box::new(Precondition::Atom(predicate(name, vars)))
}

fn parse_domain(program: &Vec<u8>) -> crate::DomainAST<'_> {
    let lexer = LexicalAnalyzer::new(program);
    match Parser::new(lexer).parse() {
        Ok(AbstractSyntaxTree::Domain(ast)) => ast,
        _ => panic!("domain parsing errors"),
    }
}

fn parse_problem(program: &Vec<u8>) -> crate::ProblemAST<'_> {
    let lexer = LexicalAnalyzer::new(program);
    match Parser::new(lexer).parse() {
        Ok(AbstractSyntaxTree::Problem(ast)) => ast,
        _ => panic!("problem parsing errors"),
    }
}

fn assert_domain_round_trip(canonical: &str) {
    let program = canonical.as_bytes().to_vec();
    assert_eq!(parse_domain(&program).to_string(), canonical);
}

fn assert_problem_round_trip(canonical: &str) {
    let program = canonical.as_bytes().to_vec();
    assert_eq!(parse_problem(&program).to_string(), canonical);
}

#[test]
pub fn pddl_display_test() {
    let formula = Precondition::And(vec![
        atom("at", vec!["p_1", "p_2"]),
        Box::new(Precondition::Not(atom("clear", vec![]))),
        Box::new(Precondition::Equals("a", "b")),
    ]);
    assert_eq!(
        formula.to_string(),
        "(and\n\t(at p_1 p_2)\n\t(not (clear))\n\t(= a b)\n)"
    );

    let var = Symbol::new("x", TokenPosition { line: 0 }, Some("loc"), None);
    let for_all = Precondition::ForAll(vec![var], atom("p_1", vec![]));
    assert_eq!(for_all.to_string(), "(forall (x - loc) (p_1))");

    let when = Precondition::Imply(
        atom("p_1", vec![]),
        Box::new(Precondition::And(vec![
            atom("p_2", vec![]),
            atom("p_3", vec![]),
        ])),
    );
    assert_eq!(
        when.to_string(),
        "(when (p_1) (and\n\t\t(p_2)\n\t\t(p_3)\n\t))"
    );

    let oneof = Precondition::Xor(vec![atom("p_1", vec![]), atom("p_2", vec![])]);
    assert_eq!(oneof.to_string(), "(oneof\n\t(p_1)\n\t(p_2)\n)");
}

#[test]
pub fn action_display_test() {
    let action = Action {
        name: "pickup",
        name_pos: TokenPosition { line: 0 },
        parameters: vec![Symbol::new(
            "?x",
            TokenPosition { line: 0 },
            Some("block"),
            None,
        )],
        preconditions: Some(Precondition::And(vec![
            atom("clear", vec!["?x"]),
            atom("handempty", vec![]),
        ])),
        effects: Some(Effect::And(vec![
            Box::new(Effect::Atom(predicate("holding", vec!["?x"]))),
            Box::new(Effect::Not(Box::new(Effect::Atom(predicate(
                "clear",
                vec!["?x"],
            ))))),
        ])),
    };
    assert_eq!(
        action.to_string(),
        "(:action pickup\n :parameters (?x - block)\n \
        :precondition (and\n \t(clear ?x)\n \t(handempty)\n )\n \
        :effect (and\n \t(holding ?x)\n \t(not (clear ?x))\n )\n)"
    );
}

#[test]
pub fn action_display_omits_empty_test() {
    let action = Action {
        name: "noop",
        name_pos: TokenPosition { line: 0 },
        parameters: vec![],
        preconditions: None,
        effects: None,
    };
    assert_eq!(action.to_string(), "(:action noop\n :parameters ()\n)");
}

#[test]
pub fn subtask_display_test() {
    let with_id = Subtask {
        id: Some(symbol("task0")),
        task: symbol("deliver"),
        terms: vec![symbol("?p"), symbol("?l")],
    };
    assert_eq!(with_id.to_string(), "(task0 (deliver ?p ?l))");

    let bare = Subtask {
        id: None,
        task: symbol("noop"),
        terms: vec![],
    };
    assert_eq!(bare.to_string(), "(noop)");
}

#[test]
pub fn predicate_display_test() {
    let predicate = Predicate::new(
        "at",
        TokenPosition { line: 0 },
        vec![
            Symbol::new("?x", TokenPosition { line: 0 }, Some("loc"), None),
            symbol("?y"),
        ],
    );
    assert_eq!(predicate.to_string(), "(at ?x - loc ?y)");
}

#[test]
pub fn action_display_round_trip_test() {
    assert_domain_round_trip(
        "(define (domain d)

	(:action pickup
	 :parameters (?x - block)
	 :precondition (and
	 	(clear ?x)
	 	(handempty)
	 )
	 :effect (and
	 	(holding ?x)
	 	(not (clear ?x))
	 )
	)

	(:action drop
	 :parameters (?x - block)
	 :effect (forall (?y) (when (at ?y) (not (at ?y))))
	)

	(:action flip
	 :parameters (?x - block)
	 :effect (probabilistic
	 	0.8 (holding ?x)
	 	0.2 (not (clear ?x))
	 )
	)

)",
    );
}

#[test]
pub fn method_display_round_trip_test() {
    assert_domain_round_trip(
        "(define (domain d)

	(:method m_1
	 :parameters (?p1 - p ?l1 - loc ?l2 - loc)
	 :task (deliver ?p1 ?l1)
	 :precondition (at ?p1 ?l1)
	 :subtasks (and (task0 (pickup ?p1 ?l1)) (task1 (drop ?p1 ?l2)))
	 :ordering (and (< task0 task1))
	 :constraints (and (not (= ?l1 ?l2)))
	)

)",
    );
}

#[test]
pub fn ordered_subtasks_round_trip_test() {
    assert_domain_round_trip(
        "(define (domain d)

	(:method m_2
	 :parameters (?p - p)
	 :task (deliver ?p)
	 :ordered-subtasks (and (pickup ?p) (drop ?p))
	)

)",
    );
}

#[test]
pub fn init_tn_display_round_trip_test() {
    assert_problem_round_trip(
        "(define (problem p1) (:domain d)

	(:htn
	 :parameters (?v)
	 :subtasks (and (task0 (deliver pkg_0 loc_0)) (task1 (deliver pkg_1 loc_1)))
	 :ordering (and (< task0 task1))
	)

)",
    );
}

#[test]
pub fn domain_to_hddl_round_trip_test() {
    assert_domain_round_trip(
        "(define (domain transport)

	(:requirements :typing :hierarchy)

	(:types
		loc - object
		pkg - object
	)

	(:constants depot - loc)

	(:predicates
		(at ?p - pkg ?l - loc)
		(road ?l1 - loc ?l2 - loc)
	)

	(:functions (fuel ?l - loc))

	(:task deliver
	 :parameters (?p - pkg ?l - loc)
	)

	(:method m_deliver
	 :parameters (?p - pkg ?l1 - loc ?l2 - loc)
	 :task (deliver ?p ?l2)
	 :ordered-subtasks (and (pickup ?p ?l1) (drop ?p ?l2))
	)

	(:action pickup
	 :parameters (?p - pkg ?l - loc)
	 :precondition (at ?p ?l)
	 :effect (not (at ?p ?l))
	)

)",
    );
}

#[test]
pub fn problem_to_hddl_round_trip_test() {
    assert_problem_round_trip(
        "(define (problem p_transport) (:domain transport)

	(:requirements :typing)

	(:objects
		pkg_0 - pkg
		loc_0 - loc
		loc_1 - loc
	)

	(:htn
	 :subtasks (and (task0 (deliver pkg_0 loc_1)))
	)

	(:init
		(at pkg_0 loc_0)
		(road loc_0 loc_1)
	)

	(:goal (at pkg_0 loc_1))

)",
    );
}

#[test]
pub fn transpiler_pipeline_test() {
    let domain = "(define (domain transport)

	(:predicates
		(at ?p ?l)
		(road ?l1 ?l2)
	)

	(:task deliver
	 :parameters (?p ?l)
	)

	(:method m_deliver
	 :parameters (?p ?l)
	 :task (deliver ?p ?l)
	 :ordered-subtasks (and (pickup ?p ?l))
	)

	(:action pickup
	 :parameters (?p ?l)
	 :precondition (at ?p ?l)
	 :effect (not (at ?p ?l))
	)

)";
    let problem = "(define (problem p_transport) (:domain transport)

	(:objects
		pkg_0
		loc_0
	)

	(:htn
	 :subtasks (and (deliver pkg_0 loc_0))
	)

	(:init (at pkg_0 loc_0))

)";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();

    // HDDL in, both outputs
    let from_hddl = Transpiler::from_hddl(&domain_bytes, Some(&problem_bytes)).unwrap();
    assert_eq!(
        from_hddl.to_hddl(),
        (domain.to_string(), Some(problem.to_string()))
    );
    let json = from_hddl.to_json();

    // JSON in, both outputs; agrees with the HDDL path
    let from_json = Transpiler::from_json(&json).unwrap();
    assert_eq!(
        from_json.to_hddl(),
        (domain.to_string(), Some(problem.to_string()))
    );
    assert_eq!(from_json.to_json(), json);

    // verify and metadata work through the facade
    assert!(from_json.verify().is_ok());
    assert_eq!(from_json.metadata().unwrap().domain_name, "transport");

    // from_input dispatches to the same constructors
    let from_hddl_input = Transpiler::from_input(Input::Hddl {
        domain: &domain_bytes,
        problem: Some(&problem_bytes),
    })
    .unwrap();
    assert_eq!(from_hddl_input.to_json(), json);
    let from_json_input = Transpiler::from_input(Input::Json(&json)).unwrap();
    assert_eq!(from_json_input.to_json(), json);
}

#[test]
pub fn transpiler_to_hddl_test() {
    let domain = "(define (domain transport)

	(:predicates (at ?p ?l))

)";
    let problem = "(define (problem p_transport) (:domain transport)

	(:objects
		pkg_0
		loc_0
	)

	(:init (at pkg_0 loc_0))

)";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();
    let program = HDDLProgram::from_hddl(&domain_bytes, Some(&problem_bytes)).unwrap();
    let (domain_hddl, problem_hddl) = Transpiler::new(program).to_hddl();
    assert_eq!(domain_hddl, domain);
    assert_eq!(problem_hddl.as_deref(), Some(problem));
}

#[test]
pub fn untyping_test() {
    let domain = "(define (domain transport)
        (:types car truck - vehicle vehicle - something location) 
        (:constants up - location)
        (:predicates (at ?c - car ?l - location ?a - car))
        (:functions (fuel ?c - car))
        (:task Drive
            :parameters (?l1 ?l2 - location ?veh - vehicle)
        )
        (:method abs_drive
            :parameters (?l1 ?l2 - location ?c - car)
            :task (Drive ?l1 ?l2 ?c)
            :subtasks (and
                (act ?c ?l1)
                (act ?c ?l2)
            )
        )
        (:action act
         :parameters (?c - car ?loc1 - location)
         :precondition (and (not (at ?c ?loc1 ?c)))
        )
        (:action act2
         :parameters (?c - car)
         :precondition (exists (?l - location) (at ?c ?l ?c))
         :effect (forall (?l - location) (not (at ?c ?l ?c)))
        )
    )";
    let problem = "(define (problem p_transport) (:domain transport)
        (:objects c1 - car loc_0 - location pkg_0)
        (:htn
            :parameters (?l1 ?l2 - location ?veh - vehicle)
            :subtasks (and (Drive ?l1 ?l2 ?veh))
        )
        (:init (at c1 loc_0 c1))
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();
    let program = HDDLProgram::from_hddl(&domain_bytes, Some(&problem_bytes)).unwrap();
    // assert domain changes
    let result = Transpiler::new(program)
        .transform(crate::Transformation::RemoveTypes)
        .unwrap()
        .into_program();
    let action = &result.domain.actions[0];
    assert!(
        action
            .parameters
            .iter()
            .all(|x| { x.symbol_type.is_none() && x.type_pos.is_none() })
    );
    if let Some(Precondition::And(prec)) = &action.preconditions {
        assert_eq!(prec.len(), 3);
        if let Precondition::Not(inner) = &*prec[0] {
            match &**inner {
                Precondition::Atom(p) => {
                    assert_eq!(p.name, "at");
                }
                _ => panic!(),
            }
        } else {
            panic!()
        }
        if let Precondition::Atom(x) = &*prec[1] {
            assert_eq!(x.name, "car");
            assert_eq!(
                x.variables,
                vec![Symbol::new_untyped("?c", TokenPosition::default())]
            )
        } else {
            panic!();
        }
        if let Precondition::Atom(x) = &*prec[2] {
            assert_eq!(x.name, "location");
            assert_eq!(
                x.variables,
                vec![Symbol::new_untyped("?loc1", TokenPosition::default())]
            )
        } else {
            panic!();
        }
    } else {
        panic!("precondition does not match the pattern")
    }
    // assert quantifier changes
    let action = &result.domain.actions[1];
    if let Some(Precondition::And(prec)) = &action.preconditions {
        assert_eq!(prec.len(), 2);
        if let Precondition::Exists(vars, body) = &*prec[0] {
            assert!(
                vars.iter()
                    .all(|x| { x.symbol_type.is_none() && x.type_pos.is_none() })
            );
            if let Precondition::And(inner) = &**body {
                assert_eq!(inner.len(), 2);
                if let Atom(typing) = &*inner[0] {
                    assert_eq!(typing.name, "location");
                    assert_eq!(
                        typing.variables,
                        vec![Symbol::new_untyped("?l", TokenPosition::default())]
                    );
                } else {
                    panic!()
                }
                if let Atom(at) = &*inner[1] {
                    assert_eq!(at.name, "at");
                } else {
                    panic!()
                }
            } else {
                panic!()
            }
        } else {
            panic!()
        }
        if let Atom(typing) = &*prec[1] {
            assert_eq!(typing.name, "car");
        } else {
            panic!()
        }
    } else {
        panic!()
    }
    if let Some(Effect::ForAll(vars, body)) = &action.effects {
        assert!(
            vars.iter()
                .all(|x| { x.symbol_type.is_none() && x.type_pos.is_none() })
        );
        if let Effect::Imply(guard, rhs) = &**body {
            if let Precondition::Atom(typing) = &**guard {
                assert_eq!(typing.name, "location");
                assert_eq!(
                    typing.variables,
                    vec![Symbol::new_untyped("?l", TokenPosition::default())]
                );
            } else {
                panic!()
            }
            assert!(matches!(&**rhs, Effect::Not(_)));
        } else {
            panic!()
        }
    } else {
        panic!()
    }
    let task = &result.domain.compound_tasks[0];
    assert!(
        task.parameters
            .iter()
            .all(|x| { x.symbol_type.is_none() && x.type_pos.is_none() })
    );
    let method = &result.domain.methods[0];
    assert!(
        method
            .params
            .iter()
            .all(|x| { x.symbol_type.is_none() && x.type_pos.is_none() })
    );
    let assert_typings = |formula: &Precondition, mut types: Vec<&str>| match formula {
        Precondition::And(inner) => {
            assert_eq!(inner.len(), types.len());
            assert!(inner.iter().all(|x| {
                if let Atom(pred) = &**x {
                    if let Some(pos) = types.iter().position(|x| *x == pred.name) {
                        types.remove(pos);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }));
        }
        _ => panic!(),
    };
    match &method.precondition {
        None => panic!(),
        Some(prec) => assert_typings(prec, vec!["location", "location", "car"]),
    }
    assert!(
        result
            .domain
            .constants
            .unwrap()
            .iter()
            .all(|x| { x.symbol_type.is_none() && x.type_pos.is_none() })
    );
    let predicates = &result.domain.predicates;
    assert!(predicates.iter().all(|p| {
        p.variables
            .iter()
            .all(|v| v.symbol_type.is_none() && v.type_pos.is_none())
    }));
    assert!(result.domain.functions.iter().all(|f| {
        f.variables
            .iter()
            .all(|v| v.symbol_type.is_none() && v.type_pos.is_none())
    }));
    assert!(predicates.contains(&Predicate {
        name: "object",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("?var", TokenPosition::default())]
    }));
    assert!(predicates.contains(&Predicate {
        name: "something",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("?var", TokenPosition::default())]
    }));
    assert!(predicates.contains(&Predicate {
        name: "location",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("?var", TokenPosition::default())]
    }));
    assert!(predicates.contains(&Predicate {
        name: "vehicle",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("?var", TokenPosition::default())]
    }));
    assert!(predicates.contains(&Predicate {
        name: "car",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("?var", TokenPosition::default())]
    }));
    assert!(predicates.contains(&Predicate {
        name: "truck",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("?var", TokenPosition::default())]
    }));
    // assert problem changes
    let problem = result.problem.as_ref().unwrap();
    let objects = &problem.objects;
    for object in objects {
        assert!(object.symbol_type.is_none());
        assert!(object.type_pos.is_none());
    }
    let init = &problem.init_state;
    assert!(init.contains(&Predicate {
        name: "object",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("up", TokenPosition::default())]
    }));
    assert!(init.contains(&Predicate {
        name: "location",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("up", TokenPosition::default())]
    }));
    assert!(init.contains(&Predicate {
        name: "object",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("c1", TokenPosition::default())]
    }));
    assert!(init.contains(&Predicate {
        name: "car",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("c1", TokenPosition::default())]
    }));
    assert!(init.contains(&Predicate {
        name: "vehicle",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("c1", TokenPosition::default())]
    }));
    assert!(init.contains(&Predicate {
        name: "object",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("loc0", TokenPosition::default())]
    }));
    assert!(init.contains(&Predicate {
        name: "location",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("loc0", TokenPosition::default())]
    }));
    assert!(init.contains(&Predicate {
        name: "object",
        name_pos: TokenPosition::default(),
        variables: vec![Symbol::new_untyped("pkg0", TokenPosition::default())]
    }));
    // assert :htn changes
    let htn = problem.init_tn.as_ref().unwrap();
    assert!(htn.parameters.is_none());
    assert_eq!(htn.tn.subtasks.len(), 1);
    assert_eq!(htn.tn.subtasks[0].task.name, HDDLProgram::HTN_TOP_TASK);
    assert!(htn.tn.subtasks[0].terms.is_empty());
    let task = result
        .domain
        .compound_tasks
        .iter()
        .find(|task| task.name == HDDLProgram::HTN_TOP_TASK)
        .unwrap();
    assert!(task.parameters.is_empty());
    let method = result
        .domain
        .methods
        .iter()
        .find(|method| method.name.name == HDDLProgram::HTN_TOP_METHOD)
        .unwrap();
    assert_eq!(method.task.name, HDDLProgram::HTN_TOP_TASK);
    assert!(method.task_terms.is_empty());
    assert!(
        method
            .params
            .iter()
            .all(|x| { x.symbol_type.is_none() && x.type_pos.is_none() })
    );
    match &method.precondition {
        Some(prec) => assert_typings(prec, vec!["location", "location", "vehicle"]),
        None => panic!(),
    }
    assert_eq!(method.tn.subtasks.len(), 1);
    assert_eq!(method.tn.subtasks[0].task.name, "Drive");
}

#[test]
pub fn untyping_rejects_invalid_program_test() {
    let domain = "(define (domain d)
        (:types car location)
        (:predicates (at ?c - car ?l - location))
        (:action act
         :parameters (?l1 ?l2 - location)
         :precondition (at ?l1 ?l2)
        )
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let transpiler = Transpiler::from_hddl(&domain_bytes, None).unwrap();
    assert!(
        transpiler
            .transform(crate::Transformation::RemoveTypes)
            .is_err()
    );
}

#[test]
pub fn untyping_preserves_init_test() {
    let domain = "(define (domain transport)
        (:types car location)
        (:constants up - location)
        (:predicates (at ?c - car ?l - location))
        (:action act
         :parameters (?c - car ?loc1 - location)
         :effect (not (at ?c ?loc1))
        )
    )";
    let problem = "(define (problem p_transport) (:domain transport)
        (:objects c1 - car loc_0 - location)
        (:init (at c1 loc_0))
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();
    let program = HDDLProgram::from_hddl(&domain_bytes, Some(&problem_bytes)).unwrap();
    let result = Transpiler::new(program)
        .transform(crate::Transformation::RemoveTypes)
        .unwrap()
        .into_program();
    let init = &result.problem.as_ref().unwrap().init_state;
    // the original facts survive untyping
    assert_eq!(
        init.iter()
            .filter(|f| f.name == "at"
                && f.variables.iter().map(|v| v.name).collect::<Vec<_>>() == vec!["c1", "loc_0"])
            .count(),
        1
    );
    // and each type atom is emitted exactly once
    for (type_name, object) in [("car", "c1"), ("location", "loc_0"), ("location", "up")] {
        assert_eq!(
            init.iter()
                .filter(|f| f.name == type_name
                    && f.variables.iter().map(|v| v.name).collect::<Vec<_>>() == vec![object])
                .count(),
            1,
            "expected exactly one ({type_name} {object}) fact"
        );
    }
}

#[test]
pub fn compile_equality_test() {
    let domain = "(define (domain eq)
        (:predicates (p ?x) (r ?x))
        (:task Top :parameters (?x ?y))
        (:method m
            :parameters (?x ?y)
            :task (Top ?x ?y)
            :precondition (not (= ?x ?y))
            :subtasks (and (act ?x))
        )
        (:action act
         :parameters (?x)
         :effect (forall (?z) (when (= ?z ?x) (r ?z)))
        )
    )";
    let problem = "(define (problem p_eq) (:domain eq)
        (:objects a b)
        (:init (p a))
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();
    let result = Transpiler::from_hddl(&domain_bytes, Some(&problem_bytes))
        .unwrap()
        .transform(crate::Transformation::RemoveEqualityConstraints)
        .unwrap()
        .into_program();
    // the equal predicate is declared and replaces every = literal
    assert!(result.domain.predicates.iter().any(|p| p.name == "EQUAL"));
    match &result.domain.methods[0].precondition {
        Some(Precondition::Not(inner)) => match &**inner {
            Precondition::Atom(predicate) => {
                assert_eq!(predicate.name, "EQUAL");
                assert_eq!(predicate.variables[0].name, "?x");
                assert_eq!(predicate.variables[1].name, "?y");
            }
            other => panic!("unexpected literal {other:?}"),
        },
        other => panic!("unexpected precondition {other:?}"),
    }
    // the rewrite reaches the guard of the conditional effect
    let effect = result.domain.actions[0].effects.as_ref().unwrap();
    let mut rewritten = false;
    let mut leftover_equals = false;
    effect.any_subformula(&mut |f| {
        if let Effect::Imply(guard, _) = f {
            guard.any_subformula(&mut |g| {
                match g {
                    Precondition::Atom(predicate) if predicate.name == "EQUAL" => rewritten = true,
                    Precondition::Equals(_, _) => leftover_equals = true,
                    _ => {}
                }
                false
            });
        }
        false
    });
    assert!(rewritten && !leftover_equals);
    // one reflexive init fact per object
    let init = &result.problem.as_ref().unwrap().init_state;
    for object in ["a", "b"] {
        assert_eq!(
            init.iter()
                .filter(|f| f.name == "EQUAL"
                    && f.variables.iter().map(|v| v.name).collect::<Vec<_>>()
                        == vec![object, object])
                .count(),
            1
        );
    }
    result.verify().unwrap();
}

// method task-network equality constraints become precondition literals
#[test]
pub fn compile_equality_constraints_test() {
    let domain = "(define (domain eqc)
        (:predicates (r ?x))
        (:task Top :parameters (?x ?y))
        (:method m
            :parameters (?x ?y)
            :task (Top ?x ?y)
            :subtasks (and (act ?x))
            :constraints (and (not (= ?x ?y)))
        )
        (:action act
         :parameters (?x)
         :effect (r ?x)
        )
    )";
    let problem = "(define (problem p_eqc) (:domain eqc)
        (:objects a b)
        (:init (r a))
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();
    let result = Transpiler::from_hddl(&domain_bytes, Some(&problem_bytes))
        .unwrap()
        .transform(crate::Transformation::RemoveEqualityConstraints)
        .unwrap()
        .into_program();
    let method = &result.domain.methods[0];
    assert!(method.tn.constraints.is_none());
    match &method.precondition {
        Some(Precondition::Not(inner)) => match &**inner {
            Precondition::Atom(predicate) => assert_eq!(predicate.name, "EQUAL"),
            other => panic!("unexpected literal {other:?}"),
        },
        other => panic!("unexpected precondition {other:?}"),
    }
    result.verify().unwrap();
}

#[test]
pub fn conjunctive_preconditions_splits_action_test() {
    let domain = "(define (domain disj)
        (:predicates (p ?x) (q ?x) (r ?x))
        (:task Top :parameters (?x))
        (:method m_top
            :parameters (?x)
            :task (Top ?x)
            :subtasks (and (act ?x))
        )
        (:action act
         :parameters (?x)
         :precondition (or (p ?x) (and (q ?x) (not (r ?x))))
         :effect (r ?x)
        )
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let result = Transpiler::from_hddl(&domain_bytes, None)
        .unwrap()
        .transform(crate::Transformation::ConjunctivePreconditions)
        .unwrap()
        .into_program();
    // the primitive is gone; a same-named compound task takes its place
    assert!(!result.domain.actions.iter().any(|a| a.name == "act"));
    let wrapper = result
        .domain
        .compound_tasks
        .iter()
        .find(|task| task.name == "act")
        .unwrap();
    assert_eq!(wrapper.parameters.len(), 1);
    // one primitive copy per cube, each carrying the cube and cloned effects
    let copy_0 = result
        .domain
        .actions
        .iter()
        .find(|a| a.name == "act_a0")
        .unwrap();
    match &copy_0.preconditions {
        Some(Precondition::Atom(predicate)) => assert_eq!(predicate.name, "p"),
        other => panic!("unexpected precondition {other:?}"),
    }
    let copy_1 = result
        .domain
        .actions
        .iter()
        .find(|a| a.name == "act_a1")
        .unwrap();
    match &copy_1.preconditions {
        Some(Precondition::And(conjuncts)) => assert_eq!(conjuncts.len(), 2),
        other => panic!("unexpected precondition {other:?}"),
    }
    for copy in [copy_0, copy_1] {
        match &copy.effects {
            Some(Effect::Atom(predicate)) => assert_eq!(predicate.name, "r"),
            other => panic!("unexpected effect {other:?}"),
        }
    }
    // one pass-through method per copy
    for (method_name, copy_name) in [("act_m0", "act_a0"), ("act_m1", "act_a1")] {
        let method = result
            .domain
            .methods
            .iter()
            .find(|m| m.name.name == method_name)
            .unwrap();
        assert_eq!(method.task.name, "act");
        assert!(method.precondition.is_none());
        assert_eq!(method.tn.subtasks.len(), 1);
        assert_eq!(method.tn.subtasks[0].task.name, copy_name);
    }
}

#[test]
pub fn quantifier_elimination_test() {
    let domain = "(define (domain qe)
        (:types box - pkg pkg - object location - object)
        (:predicates (at ?p - pkg ?l - location) (ready))
        (:task Top :parameters (?l - location))
        (:method m_top
            :parameters (?l - location)
            :task (Top ?l)
            :precondition (exists (?b - box) (at ?b ?l))
            :subtasks (and (act))
        )
        (:action act
         :parameters ()
         :precondition (and
            (ready)
            (forall (?l - location) (exists (?p - pkg) (at ?p ?l)))
         )
         :effect (forall (?p - pkg ?l - location) (not (at ?p ?l)))
        )
    )";
    let problem = "(define (problem p_qe) (:domain qe)
        (:objects p1 - pkg b1 - box loc_0 loc_1 - location)
        (:init (at p1 loc_0))
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();
    let result = Transpiler::from_hddl(&domain_bytes, Some(&problem_bytes))
        .unwrap()
        .transform(crate::Transformation::QuantifierElimintation)
        .unwrap()
        .into_program();
    let assert_ground_atom = |formula: &Precondition, name: &str, args: Vec<&str>| match formula {
        Precondition::Atom(predicate) => {
            assert_eq!(predicate.name, name);
            assert_eq!(
                predicate
                    .variables
                    .iter()
                    .map(|v| v.name)
                    .collect::<Vec<_>>(),
                args
            );
        }
        other => panic!("expected atom ({name}), got {other:?}"),
    };
    // the nested forall/exists becomes a conjunction of disjunctions
    let action = &result.domain.actions[0];
    if let Some(Precondition::And(outer)) = &action.preconditions {
        assert_eq!(outer.len(), 2);
        assert_ground_atom(&outer[0], "ready", vec![]);
        if let Precondition::And(per_location) = &*outer[1] {
            assert_eq!(per_location.len(), 2);
            for (disjunction, location) in per_location.iter().zip(["loc_0", "loc_1"]) {
                if let Precondition::Or(per_pkg) = &**disjunction {
                    // exists over pkg covers the box subtype object too
                    assert_eq!(per_pkg.len(), 2);
                    assert_ground_atom(&per_pkg[0], "at", vec!["p1", location]);
                    assert_ground_atom(&per_pkg[1], "at", vec!["b1", location]);
                } else {
                    panic!("expected a disjunction for {location}")
                }
            }
        } else {
            panic!("unexpected expansion {:?}", outer[1])
        }
    } else {
        panic!("unexpected precondition {:?}", action.preconditions)
    }
    // a single forall binding two variables expands to the cartesian product
    if let Some(Effect::And(effects)) = &action.effects {
        let expected = [
            ("p1", "loc_0"),
            ("p1", "loc_1"),
            ("b1", "loc_0"),
            ("b1", "loc_1"),
        ];
        assert_eq!(effects.len(), expected.len());
        for (effect, (pkg, location)) in effects.iter().zip(expected) {
            match &**effect {
                Effect::Not(inner) => match &**inner {
                    Effect::Atom(predicate) => {
                        assert_eq!(predicate.name, "at");
                        assert_eq!(
                            predicate
                                .variables
                                .iter()
                                .map(|v| v.name)
                                .collect::<Vec<_>>(),
                            vec![pkg, location]
                        );
                    }
                    other => panic!("unexpected effect instance {other:?}"),
                },
                other => panic!("unexpected effect instance {other:?}"),
            }
        }
    } else {
        panic!("unexpected effect {:?}", action.effects)
    }
    // method precondition
    let method = &result.domain.methods[0];
    if let Some(Precondition::Or(instances)) = &method.precondition {
        assert_eq!(instances.len(), 1);
        assert_ground_atom(&instances[0], "at", vec!["b1", "?l"]);
    } else {
        panic!("unexpected precondition {:?}", method.precondition)
    }
    // no quantifier survives anywhere
    let quantifier_free = |formula: &Precondition| {
        !formula.any_subformula(&mut |f| {
            matches!(f, Precondition::ForAll(_, _) | Precondition::Exists(_, _))
        })
    };
    for action in result.domain.actions.iter() {
        assert!(action.preconditions.iter().all(&quantifier_free));
        assert!(action.effects.iter().all(|effect| {
            !effect
                .any_subformula(&mut |f| matches!(f, Effect::ForAll(_, _) | Effect::Exists(_, _)))
        }));
    }
    for method in result.domain.methods.iter() {
        assert!(method.precondition.iter().all(&quantifier_free));
    }
}

#[test]
pub fn conjunctive_preconditions_splits_method_test() {
    let domain = "(define (domain mdisj)
        (:predicates (p ?x) (q ?x) (r ?x))
        (:task Top :parameters (?x))
        (:method m
            :parameters (?x)
            :task (Top ?x)
            :precondition (or (p ?x) (q ?x))
            :subtasks (and (act ?x))
        )
        (:action act
         :parameters (?x)
         :effect (r ?x)
        )
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let result = Transpiler::from_hddl(&domain_bytes, None)
        .unwrap()
        .transform(crate::Transformation::ConjunctivePreconditions)
        .unwrap()
        .into_program();
    assert!(!result.domain.methods.iter().any(|m| m.name.name == "m"));
    for (method_name, atom_name) in [("m_m0", "p"), ("m_m1", "q")] {
        let copy = result
            .domain
            .methods
            .iter()
            .find(|m| m.name.name == method_name)
            .unwrap();
        assert_eq!(copy.task.name, "Top");
        match &copy.precondition {
            Some(Precondition::Atom(predicate)) => assert_eq!(predicate.name, atom_name),
            other => panic!("unexpected precondition {other:?}"),
        }
        assert_eq!(copy.tn.subtasks[0].task.name, "act");
    }
    // the action was untouched, no wrapper task appeared
    assert_eq!(result.domain.actions.len(), 1);
    assert_eq!(result.domain.compound_tasks.len(), 1);
    result.verify().unwrap();
}

#[test]
pub fn numeric_effect_round_trip_test() {
    assert_domain_round_trip(
        "(define (domain transport)

	(:requirements :typing :numeric-fluents)

	(:types
		vehicle - object
		location - object
	)

	(:predicates (at ?v - vehicle ?l - location))

	(:functions
		(fuel ?v - vehicle)
		(road-length ?l1 - location ?l2 - location)
		(total-cost)
	)

	(:action drive
	 :parameters (?v - vehicle ?l1 - location ?l2 - location)
	 :precondition (at ?v ?l1)
	 :effect (and
	 	(not (at ?v ?l1))
	 	(at ?v ?l2)
	 	(increase (total-cost) (road-length ?l1 ?l2))
	 	(decrease (fuel ?v) 2.5)
	 	(assign (fuel ?v) 0)
	 	(forall (?l - location) (scale-up (fuel ?v) 2))
	 )
	)

)",
    );
}

#[test]
pub fn quantifier_elimination_grounds_numeric_effects_test() {
    let domain = "(define (domain qe)
        (:types location - object)
        (:predicates (visited ?l - location))
        (:functions (visits ?l - location) (distance ?from ?to - location) (total-cost))
        (:action tour
         :parameters (?start - location)
         :effect (forall (?l - location) (and
            (visited ?l)
            (increase (visits ?l) 1)
            (increase (total-cost) (+ (distance ?start ?l) 1))))
        )
    )";
    let problem = "(define (problem p_qe) (:domain qe)
        (:objects loc_0 loc_1 - location)
        (:init (visited loc_0))
    )";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();
    let result = Transpiler::from_hddl(&domain_bytes, Some(&problem_bytes))
        .unwrap()
        .transform(crate::Transformation::QuantifierElimintation)
        .unwrap()
        .into_program();
    // one instance per location, with the bound variable replaced in both the
    // updated fluent and the fluent read by the operand
    let Some(Effect::And(instances)) = &result.domain.actions[0].effects else {
        panic!("expected a conjunction of instances")
    };
    assert_eq!(instances.len(), 2);
    for (instance, location) in instances.iter().zip(["loc_0", "loc_1"]) {
        let Effect::And(conjuncts) = instance.as_ref() else {
            panic!("expected a grounded conjunction")
        };
        assert_eq!(conjuncts.len(), 3);
        match conjuncts[1].as_ref() {
            Effect::Numeric(AssignmentOperator::Increase, fluent, NumericExpr::Number(_)) => {
                assert_eq!(fluent.name, "visits");
                assert_eq!(fluent.variables[0].name, location);
            }
            other => panic!("expected (increase (visits {location}) 1), got {other}"),
        }
        match conjuncts[2].as_ref() {
            Effect::Numeric(AssignmentOperator::Increase, fluent, NumericExpr::Sum(terms)) => {
                assert_eq!(fluent.name, "total-cost");
                let NumericExpr::Fluent(read) = &terms[0] else {
                    panic!("expected a fluent term, got {}", terms[0])
                };
                assert_eq!(read.name, "distance");
                let args: Vec<&str> = read.variables.iter().map(|v| v.name).collect();
                assert_eq!(args, vec!["?start", location]);
            }
            other => panic!("expected an increase by (distance ?start {location}), got {other}"),
        }
    }
}

#[test]
pub fn numeric_problem_round_trip_test() {
    assert_problem_round_trip(
        "(define (problem p_transport) (:domain transport)

	(:requirements :typing :action-costs)

	(:objects
		truck_0 - vehicle
		loc_0 - location
		loc_1 - location
	)

	(:init
		(at truck_0 loc_0)
		(road loc_0 loc_1)
		(= (total-cost) 0)
		(= (road-length loc_0 loc_1) 22)
	)

	(:goal (at truck_0 loc_1))

	(:metric minimize (+ (total-cost) (* 2 (road-length loc_0 loc_1))))

)",
    );
    // a single inline value, an empty goal, and a maximised fluent
    assert_problem_round_trip(
        "(define (problem p_reward) (:domain d)

	(:init (= (reward) 2.5))

	(:goal ())

	(:metric maximize (reward))

)",
    );
}

#[test]
pub fn numeric_effect_arithmetic_round_trip_test() {
    assert_domain_round_trip(
        "(define (domain sums)

	(:functions
		(a ?x - loc)
		(b)
	)

	(:action act
	 :parameters (?x - loc)
	 :effect (increase (b) (+ (a ?x) (* 2 (b)) 1))
	)

)",
    );
}

#[test]
pub fn numeric_problem_pipeline_test() {
    let domain = "(define (domain transport)
        (:requirements :typing :action-costs)
        (:types vehicle location)
        (:predicates (at ?v - vehicle ?l - location) (road ?l1 ?l2 - location))
        (:functions (road-length ?l1 ?l2 - location) (total-cost))
        (:action drive
         :parameters (?v - vehicle ?l1 ?l2 - location)
         :precondition (and (at ?v ?l1) (road ?l1 ?l2))
         :effect (and
            (not (at ?v ?l1))
            (at ?v ?l2)
            (increase (total-cost) (road-length ?l1 ?l2)))
        )
    )";
    let problem = "(define (problem p_transport) (:domain transport)

	(:objects
		truck_0 - vehicle
		loc_0 - location
		loc_1 - location
	)

	(:init
		(at truck_0 loc_0)
		(road loc_0 loc_1)
		(= (total-cost) 0)
		(= (road-length loc_0 loc_1) 22)
	)

	(:goal (at truck_0 loc_1))

	(:metric minimize (total-cost))

)";
    let domain_bytes = domain.as_bytes().to_vec();
    let problem_bytes = problem.as_bytes().to_vec();
    let program = HDDLProgram::from_hddl(&domain_bytes, Some(&problem_bytes)).unwrap();
    program.verify().unwrap();
    let transpiler = Transpiler::new(program);
    let (_, printed_problem) = transpiler.to_hddl();
    assert_eq!(printed_problem.as_deref(), Some(problem));
    // the JSON export carries both new fields and is stable under re-import
    let json = transpiler.to_json();
    assert!(json.contains("\"init_values\""));
    assert!(json.contains("\"metric\""));
    let atoms_only = "(define (problem p_plain) (:domain transport) (:init (at truck_0 loc_0)))";
    let atoms_only = atoms_only.as_bytes().to_vec();
    let plain = Transpiler::new(HDDLProgram::from_hddl(&domain_bytes, Some(&atoms_only)).unwrap());
    assert!(!plain.to_json().contains("\"init_values\""));
    assert!(!plain.to_json().contains("\"metric\""));
    let reimported = HDDLProgram::from_json(&json).unwrap();
    reimported.verify().unwrap();
    assert_eq!(Transpiler::new(reimported).to_json(), json);
}

// the domain and problem requirement flags after a refresh
fn refreshed_requirements(
    domain: &str,
    problem: Option<&str>,
) -> (Vec<RequirementType>, Vec<RequirementType>) {
    let domain = domain.as_bytes().to_vec();
    let problem = problem.map(|p| p.as_bytes().to_vec());
    let mut transpiler = Transpiler::from_hddl(&domain, problem.as_ref()).unwrap();
    transpiler.refresh_requirements();
    let problem_requirements = transpiler
        .program
        .problem
        .as_ref()
        .map(|p| p.requirements.clone())
        .unwrap_or_default();
    (transpiler.program.domain.requirements, problem_requirements)
}

#[test]
pub fn refresh_adds_flags_for_used_constructs_test() {
    let (domain, _) = refreshed_requirements(
        "(define (domain d)
            (:requirements :typing)
            (:types t)
            (:predicates (p) (q ?x - t))
            (:task abstract :parameters ())
            (:method m
             :parameters ()
             :task (abstract)
             :precondition (not (p))
             :subtasks (act)
            )
            (:action act
             :parameters ()
             :precondition (and (or (p) (exists (?x - t) (q ?x))) (forall (?x - t) (q ?x)))
             :effect (and (when (p) (not (p))) (probabilistic 0.5 (p)))
            )
        )",
        None,
    );
    let expected = [
        RequirementType::TypedObjects,
        RequirementType::UniversalPreconditions,
        RequirementType::ExistentialPreconditions,
        RequirementType::NegativePreconditions,
        RequirementType::DisjunctivePreconditions,
        RequirementType::ConditionalEffects,
        RequirementType::ProbabilisticEffects,
        RequirementType::Hierarchy,
        RequirementType::MethodPreconditions,
    ];
    assert_eq!(domain, expected);
}

#[test]
pub fn refresh_drops_flags_for_absent_constructs_test() {
    // every declared flag but :typing is unused, in the domain and the problem
    let (domain, problem) = refreshed_requirements(
        "(define (domain d)
            (:requirements :typing :existential-preconditions :equality :fluents :action-costs :hierarchy :method-preconditions)
            (:predicates (p))
            (:action act :parameters () :precondition (p) :effect (not (p)))
        )",
        Some(
            "(define (problem p) (:domain d)
            (:requirements :typing :equality :action-costs)
            (:init (p))
        )",
        ),
    );
    assert_eq!(domain, [RequirementType::TypedObjects]);
    assert_eq!(problem, [RequirementType::TypedObjects]);
}

#[test]
pub fn refresh_picks_the_numeric_flag_test() {
    let cost_domain = "(define (domain d)
        (:requirements :typing)
        (:predicates (p))
        (:functions (total-cost) (toll))
        (:action act :parameters () :effect (and (p) (increase (total-cost) (toll))))
    )";
    let cost_problem = "(define (problem p) (:domain d)
        (:init (= (total-cost) 0) (= (toll) 3))
        (:metric minimize (total-cost))
    )";
    // increasing total-cost by constants or fluents, and minimising it, is action costs
    let (domain, _) = refreshed_requirements(cost_domain, Some(cost_problem));
    assert_eq!(
        domain,
        [RequirementType::TypedObjects, RequirementType::ActionCosts]
    );
    // a declared numeric flag that covers the usage stays as written
    let (domain, _) = refreshed_requirements(
        &cost_domain.replace(":typing", ":typing :numeric-fluents"),
        Some(cost_problem),
    );
    assert_eq!(
        domain,
        [
            RequirementType::TypedObjects,
            RequirementType::NumericFluents
        ]
    );
    // any other update, or any other metric, needs numeric fluents
    let (domain, _) = refreshed_requirements(
        &cost_domain.replace("(increase (total-cost) (toll))", "(decrease (toll) 1)"),
        Some(cost_problem),
    );
    assert_eq!(
        domain,
        [
            RequirementType::TypedObjects,
            RequirementType::NumericFluents
        ]
    );
    let (domain, _) = refreshed_requirements(
        cost_domain,
        Some(&cost_problem.replace("minimize (total-cost)", "minimize (+ (total-cost) (toll))")),
    );
    assert_eq!(
        domain,
        [
            RequirementType::TypedObjects,
            RequirementType::NumericFluents
        ]
    );
    // declared :action-costs is kept but no longer sufficient
    let (domain, _) = refreshed_requirements(
        &cost_domain
            .replace(":typing", ":typing :action-costs")
            .replace("(increase (total-cost) (toll))", "(decrease (toll) 1)"),
        Some(cost_problem),
    );
    assert_eq!(
        domain,
        [
            RequirementType::TypedObjects,
            RequirementType::ActionCosts,
            RequirementType::NumericFluents
        ]
    );
}
