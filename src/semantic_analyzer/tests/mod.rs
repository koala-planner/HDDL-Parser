mod duplicate_tests;
mod undefined_tests;
mod cycle_detection_tests;
mod type_checking_tests;
mod tdg_tests;
mod problem_test;
mod warning_tests;

use super::*;
use crate::syntactic_analyzer::*;
use crate::lexical_analyzer::*;
use analyzers::*;