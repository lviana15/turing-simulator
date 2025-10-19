#![allow(clippy::module_name_repetitions)]

use std::collections::HashSet;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub enum ConversionError {
    Io(std::io::Error),
    InvalidHeader(String),
    ParseError(ParseTransitionError),
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::Io(e) => write!(f, "I/O error: {e}"),
            ConversionError::InvalidHeader(s) => write!(f, "Invalid machine type header: {s}"),
            ConversionError::ParseError(e) => write!(f, "Failed to parse transition line: {e}"),
        }
    }
}

impl std::error::Error for ConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConversionError::Io(e) => Some(e),
            ConversionError::ParseError(e) => Some(e),
            ConversionError::InvalidHeader(_) => None,
        }
    }
}

impl From<std::io::Error> for ConversionError {
    fn from(err: std::io::Error) -> Self {
        ConversionError::Io(err)
    }
}

impl From<ParseTransitionError> for ConversionError {
    fn from(err: ParseTransitionError) -> Self {
        ConversionError::ParseError(err)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseTransitionError {
    Empty,
    InvalidPartCount(usize),
    InvalidDirection(String),
    InvalidSymbol(String),
}

impl Display for ParseTransitionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseTransitionError::Empty => write!(f, "Line is empty"),
            ParseTransitionError::InvalidPartCount(count) => {
                write!(f, "Invalid number of parts, expected 5, got {count}")
            }
            ParseTransitionError::InvalidDirection(dir) => {
                write!(f, "Invalid direction: {dir}")
            }
            ParseTransitionError::InvalidSymbol(sym) => {
                write!(f, "Invalid symbol, must be a single char: '{sym}'")
            }
        }
    }
}

impl std::error::Error for ParseTransitionError {}

mod constants {
    pub const LEFT_WALL: char = '#';
    pub const RIGHT_WALL: char = '$';
    pub const BLANK: char = '_';
    pub const ANY: char = '*';
    pub const HALT_PREFIX: &str = "halt";
    pub const SIM_PREFIX: &str = "sim_";
    pub const START_STATE: &str = "0";
}
use constants::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
    Stay,
}

impl FromStr for Direction {
    type Err = ParseTransitionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "l" => Ok(Direction::Left),
            "r" => Ok(Direction::Right),
            "*" => Ok(Direction::Stay),
            _ => Err(ParseTransitionError::InvalidDirection(s.to_string())),
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "l"),
            Direction::Right => write!(f, "r"),
            Direction::Stay => write!(f, "*"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MachineType {
    Infinite,
    Sipser,
}

impl FromStr for MachineType {
    type Err = ConversionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ";S" => Ok(MachineType::Sipser),
            ";I" => Ok(MachineType::Infinite),
            _ => Err(ConversionError::InvalidHeader(s.to_string())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Transition {
    current_state: String,
    current_symbol: char,
    new_symbol: char,
    direction: Direction,
    new_state: String,
}

impl Display for Transition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let new_symbol_str = if self.new_symbol == self.current_symbol {
            ANY.to_string()
        } else {
            self.new_symbol.to_string()
        };
        let new_state_str = if self.new_state == self.current_state {
            ANY.to_string()
        } else {
            self.new_state.clone()
        };
        write!(
            f,
            "{} {} {} {} {}",
            self.current_state, self.current_symbol, new_symbol_str, self.direction, new_state_str
        )
    }
}

#[inline]
fn is_halt_state(state: &str) -> bool {
    state.starts_with(HALT_PREFIX)
}

fn get_next_state(original_new_state: &str, check_state: String) -> String {
    if is_halt_state(original_new_state) {
        original_new_state.to_string()
    } else {
        check_state
    }
}

fn parse_line(line: &str) -> Result<Transition, ParseTransitionError> {
    let line = line.split(';').next().unwrap_or("").trim();
    if line.is_empty() {
        return Err(ParseTransitionError::Empty);
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(ParseTransitionError::InvalidPartCount(parts.len()));
    }
    let current_symbol = parts[1]
        .chars()
        .next()
        .ok_or_else(|| ParseTransitionError::InvalidSymbol(parts[1].to_string()))?;
    let new_symbol = parts[2]
        .chars()
        .next()
        .ok_or_else(|| ParseTransitionError::InvalidSymbol(parts[2].to_string()))?;
    Ok(Transition {
        current_state: parts[0].to_string(),
        current_symbol,
        new_symbol,
        direction: parts[3].parse::<Direction>()?,
        new_state: parts[4].to_string(),
    })
}

fn rename_original_states(original_transitions: &[Transition], prefix: &str) -> Vec<Transition> {
    let rename = |state: &str| {
        if is_halt_state(state) {
            state.to_string()
        } else {
            format!("{}{}", prefix, state)
        }
    };
    original_transitions
        .iter()
        .map(|t| Transition {
            current_state: rename(&t.current_state),
            new_state: rename(&t.new_state),
            current_symbol: t.current_symbol,
            new_symbol: t.new_symbol,
            direction: t.direction,
        })
        .collect()
}

fn generate_carry_logic(carry_0_state: &str, carry_1_state: &str) -> Vec<Transition> {
    vec![
        Transition {
            current_state: carry_0_state.to_string(),
            current_symbol: '0',
            new_symbol: '0',
            direction: Direction::Right,
            new_state: carry_0_state.to_string(),
        },
        Transition {
            current_state: carry_0_state.to_string(),
            current_symbol: '1',
            new_symbol: '0',
            direction: Direction::Right,
            new_state: carry_1_state.to_string(),
        },
        Transition {
            current_state: carry_1_state.to_string(),
            current_symbol: '0',
            new_symbol: '1',
            direction: Direction::Right,
            new_state: carry_0_state.to_string(),
        },
        Transition {
            current_state: carry_1_state.to_string(),
            current_symbol: '1',
            new_symbol: '1',
            direction: Direction::Right,
            new_state: carry_1_state.to_string(),
        },
    ]
}

fn generate_return_head_logic(return_state: &str, target_state: &str) -> Vec<Transition> {
    vec![
        Transition {
            current_state: return_state.to_string(),
            current_symbol: ANY,
            new_symbol: ANY,
            direction: Direction::Left,
            new_state: return_state.to_string(),
        },
        Transition {
            current_state: return_state.to_string(),
            current_symbol: LEFT_WALL,
            new_symbol: LEFT_WALL,
            direction: Direction::Right,
            new_state: target_state.to_string(),
        },
    ]
}

fn generate_check_logic(
    check_state: &str,
    on_any_state: &str,
    on_symbol: char,
    on_symbol_new_symbol: char,
    on_symbol_direction: Direction,
    on_symbol_new_state: &str,
) -> Vec<Transition> {
    vec![
        Transition {
            current_state: check_state.to_string(),
            current_symbol: ANY,
            new_symbol: ANY,
            direction: Direction::Stay,
            new_state: on_any_state.to_string(),
        },
        Transition {
            current_state: check_state.to_string(),
            current_symbol: on_symbol,
            new_symbol: on_symbol_new_symbol,
            direction: on_symbol_direction,
            new_state: on_symbol_new_state.to_string(),
        },
    ]
}

fn generate_setup_transitions(renamed_start_state: &str) -> Vec<Transition> {
    let write_end_marker_state = "q_write_end_marker";
    let return_head_state = "q_return_head";
    let q_carry_0 = "q_carry_0";
    let q_carry_1 = "q_carry_1";

    vec![
        Transition {
            current_state: START_STATE.to_string(),
            current_symbol: '0',
            new_symbol: LEFT_WALL,
            direction: Direction::Right,
            new_state: q_carry_0.to_string(),
        },
        Transition {
            current_state: START_STATE.to_string(),
            current_symbol: '1',
            new_symbol: LEFT_WALL,
            direction: Direction::Right,
            new_state: q_carry_1.to_string(),
        },
    ]
    .into_iter()
    .chain(generate_carry_logic(q_carry_0, q_carry_1))
    .chain(vec![
        Transition {
            current_state: q_carry_0.to_string(),
            current_symbol: BLANK,
            new_symbol: '0',
            direction: Direction::Right,
            new_state: write_end_marker_state.to_string(),
        },
        Transition {
            current_state: q_carry_1.to_string(),
            current_symbol: BLANK,
            new_symbol: '1',
            direction: Direction::Right,
            new_state: write_end_marker_state.to_string(),
        },
        Transition {
            current_state: write_end_marker_state.to_string(),
            current_symbol: BLANK,
            new_symbol: RIGHT_WALL,
            direction: Direction::Left,
            new_state: return_head_state.to_string(),
        },
    ])
    .chain(generate_return_head_logic(
        return_head_state,
        renamed_start_state,
    ))
    .chain(vec![
        Transition {
            current_state: START_STATE.to_string(),
            current_symbol: BLANK,
            new_symbol: LEFT_WALL,
            direction: Direction::Right,
            new_state: "q_write_end_marker_empty".to_string(),
        },
        Transition {
            current_state: "q_write_end_marker_empty".to_string(),
            current_symbol: BLANK,
            new_symbol: RIGHT_WALL,
            direction: Direction::Left,
            new_state: renamed_start_state.to_string(),
        },
    ])
    .collect()
}

fn generate_check_right_logic(state: &str) -> Vec<Transition> {
    let check_right_state = format!("check_right_{}", state);
    let expand_right_state = format!("expand_right_{}", state);

    generate_check_logic(
        &check_right_state,
        state,
        RIGHT_WALL,
        BLANK,
        Direction::Right,
        &expand_right_state,
    )
    .into_iter()
    .chain(std::iter::once(Transition {
        current_state: expand_right_state,
        current_symbol: BLANK,
        new_symbol: RIGHT_WALL,
        direction: Direction::Left,
        new_state: state.to_string(),
    }))
    .collect()
}

fn generate_shift_sub_logic(state_suffix: &str, shift_start_state: &str) -> Vec<Transition> {
    let carry_0 = &format!("shift_carry_0_{}", state_suffix);
    let carry_1 = &format!("shift_carry_1_{}", state_suffix);
    let write_end = &format!("shift_write_end_{}", state_suffix);
    let return_s = &format!("shift_return_{}", state_suffix);

    vec![
        Transition {
            current_state: shift_start_state.to_string(),
            current_symbol: '0',
            new_symbol: BLANK,
            direction: Direction::Right,
            new_state: carry_0.clone(),
        },
        Transition {
            current_state: shift_start_state.to_string(),
            current_symbol: '1',
            new_symbol: BLANK,
            direction: Direction::Right,
            new_state: carry_1.clone(),
        },
        Transition {
            current_state: shift_start_state.to_string(),
            current_symbol: BLANK,
            new_symbol: BLANK,
            direction: Direction::Stay,
            new_state: state_suffix.to_string(),
        },
    ]
    .into_iter()
    .chain(generate_carry_logic(carry_0, carry_1))
    .chain(vec![
        Transition {
            current_state: carry_0.clone(),
            current_symbol: BLANK,
            new_symbol: '0',
            direction: Direction::Right,
            new_state: write_end.clone(),
        },
        Transition {
            current_state: carry_1.clone(),
            current_symbol: BLANK,
            new_symbol: '1',
            direction: Direction::Right,
            new_state: write_end.clone(),
        },
        Transition {
            current_state: carry_0.clone(),
            current_symbol: RIGHT_WALL,
            new_symbol: '0',
            direction: Direction::Right,
            new_state: write_end.clone(),
        },
        Transition {
            current_state: carry_1.clone(),
            current_symbol: RIGHT_WALL,
            new_symbol: '1',
            direction: Direction::Right,
            new_state: write_end.clone(),
        },
        Transition {
            current_state: shift_start_state.to_string(),
            current_symbol: RIGHT_WALL,
            new_symbol: BLANK,
            direction: Direction::Right,
            new_state: write_end.clone(),
        },
        Transition {
            current_state: write_end.clone(),
            current_symbol: BLANK,
            new_symbol: RIGHT_WALL,
            direction: Direction::Left,
            new_state: return_s.clone(),
        },
    ])
    .chain(generate_return_head_logic(return_s, state_suffix))
    .collect()
}

fn generate_check_left_logic(state: &str) -> Vec<Transition> {
    let check_left_state = format!("check_left_{}", state);
    let shift_start_state = format!("shift_start_{}", state);

    generate_check_logic(
        &check_left_state,
        state,
        LEFT_WALL,
        LEFT_WALL,
        Direction::Right,
        &shift_start_state,
    )
    .into_iter()
    .chain(generate_shift_sub_logic(state, &shift_start_state))
    .collect()
}

fn convert_simulation_transitions(original_transitions: &[Transition]) -> Vec<Transition> {
    let mut target_states = HashSet::new();
    let mut new_transitions: Vec<Transition> = original_transitions
        .iter()
        .map(|t| {
            if !is_halt_state(&t.new_state) {
                target_states.insert(t.new_state.clone());
            }
            if is_halt_state(&t.current_state) {
                return t.clone();
            }
            match t.direction {
                Direction::Stay => t.clone(),
                Direction::Right => Transition {
                    new_state: get_next_state(&t.new_state, format!("check_right_{}", t.new_state)),
                    ..t.clone()
                },
                Direction::Left => Transition {
                    new_state: get_next_state(&t.new_state, format!("check_left_{}", t.new_state)),
                    ..t.clone()
                },
            }
        })
        .collect();

    for state in target_states {
        new_transitions.extend(generate_check_right_logic(&state));
        new_transitions.extend(generate_check_left_logic(&state));
    }

    new_transitions
}

fn generate_wall_setup_transitions(renamed_start_state: &str) -> Vec<Transition> {
    vec![
        Transition {
            current_state: START_STATE.to_string(),
            current_symbol: ANY,
            new_symbol: ANY,
            direction: Direction::Left,
            new_state: "q_write_wall".to_string(),
        },
        Transition {
            current_state: "q_write_wall".to_string(),
            current_symbol: BLANK,
            new_symbol: LEFT_WALL,
            direction: Direction::Right,
            new_state: renamed_start_state.to_string(),
        },
    ]
}

fn convert_sipser_to_infinite(original_transitions: &[Transition]) -> Vec<Transition> {
    let mut target_states = HashSet::new();
    let mut new_transitions: Vec<Transition> = original_transitions
        .iter()
        .map(|t| {
            if t.direction == Direction::Left && !is_halt_state(&t.new_state) {
                target_states.insert(t.new_state.clone());
            }
            if t.direction == Direction::Left {
                Transition {
                    new_state: get_next_state(
                        &t.new_state,
                        format!("check_left_wall_{}", t.new_state),
                    ),
                    ..t.clone()
                }
            } else {
                t.clone()
            }
        })
        .collect();

    for state in target_states {
        let check_state = format!("check_left_wall_{}", state);
        new_transitions.extend(generate_check_logic(
            &check_state,
            &state,
            LEFT_WALL,
            LEFT_WALL,
            Direction::Stay,
            HALT_PREFIX,
        ));
    }

    new_transitions
}

fn run_converter(input_path: &str, output_path: &str) -> Result<MachineType, ConversionError> {
    let content = fs::read_to_string(input_path)?;
    let mut lines = content.lines();
    let machine_type = lines
        .next()
        .ok_or_else(|| ConversionError::InvalidHeader("File is empty".to_string()))?
        .parse::<MachineType>()?;
    let original_transitions = lines
        .map(parse_line)
        .filter(|r| !matches!(r, Err(ParseTransitionError::Empty)))
        .collect::<Result<Vec<Transition>, _>>()?;

    let mut output_file = fs::File::create(output_path)?;
    let renamed_start_state = format!("{}{}", SIM_PREFIX, START_STATE);

    let (header, all_transitions) = match machine_type {
        MachineType::Infinite => {
            let header =
                format!("; --- Infinite-to-Sipser Simulation ---\n; Start state: {START_STATE}\n");
            let renamed = rename_original_states(&original_transitions, SIM_PREFIX);
            let transitions = generate_setup_transitions(&renamed_start_state)
                .into_iter()
                .chain(convert_simulation_transitions(&renamed))
                .collect::<Vec<_>>();
            (header, transitions)
        }
        MachineType::Sipser => {
            let header =
                format!("; --- Sipser-to-Infinite Simulation ---\n; Start state: {START_STATE}\n");
            let renamed = rename_original_states(&original_transitions, SIM_PREFIX);
            let transitions = generate_wall_setup_transitions(&renamed_start_state)
                .into_iter()
                .chain(convert_sipser_to_infinite(&renamed))
                .collect::<Vec<_>>();
            (header, transitions)
        }
    };

    write!(output_file, "{}", header)?;

    for t in all_transitions {
        writeln!(output_file, "{}", t)?;
    }

    Ok(machine_type)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).map_or("example.in", |s| s.as_str());

    let path = Path::new(input_path);
    if path.extension().and_then(|s| s.to_str()) != Some("in") {
        eprintln!("Error: Input file name must end with '.in': {}", input_path);
        std::process::exit(1);
    }

    let output_path = path.with_extension("out");
    let output_path_str = match output_path.to_str() {
        Some(s) => s,
        None => {
            eprintln!("Error: Could not create a valid UTF-8 output path.");
            std::process::exit(1);
        }
    };

    match run_converter(input_path, output_path_str) {
        Ok(machine_type) => {
            let model_name = match machine_type {
                MachineType::Infinite => "Sipser",
                MachineType::Sipser => "Infinite",
            };
            println!(
                "✅ Successfully converted to {} model.\n Input: {}\n Output: {}",
                model_name, input_path, output_path_str
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
