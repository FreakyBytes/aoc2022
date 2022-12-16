use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Error};
use petgraph::prelude::GraphMap;
use regex::Regex;

type ValveGraph<'a> = GraphMap<&'a String, u32, petgraph::Directed>;
type ValveMap = HashMap<String, Valve>;
static MAX_TIME: usize = 30;

#[derive(Debug, Clone)]
struct Valve {
    // id: String,
    flow_rate: u32,
    targets: Vec<String>,
}

#[derive(Debug, Clone)]
enum WalkStep {
    Tunnel(String),
    OpenedValve(String),
}

#[derive(Debug, Clone)]
struct GraphWalk {
    path: Vec<WalkStep>,
    opened_valves: HashSet<String>,
    flow_rate: u32,
    current_node: String,
}

impl GraphWalk {
    fn new(start_node: &str) -> Self {
        Self {
            path: Vec::new(),
            opened_valves: HashSet::new(),
            flow_rate: 0,
            current_node: start_node.to_string(),
        }
    }

    fn get_minute(&self) -> usize {
        self.path.len()
    }

    fn move_to_tunnel(&mut self, tunnel: String) {
        self.path.push(WalkStep::Tunnel(self.current_node.clone()));
        self.current_node = tunnel;
    }

    fn open_valve(&mut self, valves: &ValveMap) {
        if let Some(valve) = valves.get(&self.current_node) {
            self.path
                .push(WalkStep::OpenedValve(self.current_node.clone()));
            self.opened_valves.insert(self.current_node.clone());
            self.flow_rate += valve.flow_rate;
        } else {
            panic!("Trying to open valve that does not exist: {self:?}");
        }
    }

    fn generate_candidates(&self, graph: &ValveGraph, valves: &ValveMap) -> Vec<(WalkStep, i64)> {
        let mut candidates = graph
            .neighbors_directed(&self.current_node, petgraph::Direction::Outgoing)
            .map(|neighbor| {
                if self.opened_valves.contains(neighbor) {
                    (WalkStep::Tunnel(neighbor.to_owned()), 0)
                } else {
                    (
                        WalkStep::Tunnel(neighbor.to_owned()),
                        valves
                            .get(neighbor)
                            .map_or(0, |v| v.flow_rate as i64 - self.flow_rate as i64),
                    )
                }
            })
            // TODO add look ahead here?
            .collect::<Vec<_>>();

        // add self, if not already opened valve
        if let Some(node_flow_rate) = valves.get(&self.current_node).map(|v| v.flow_rate) {
            if node_flow_rate > 0 && !self.opened_valves.contains(&self.current_node) {
                candidates.push((
                    WalkStep::OpenedValve(self.current_node.clone()),
                    node_flow_rate as i64,
                ));
            }
        }

        candidates.sort_by(|(_, a), (_, b)| (*b).cmp(a));
        candidates
    }

    fn walk(self, graph: &ValveGraph, valves: &ValveMap) -> (u32, Vec<Self>) {
        if self.get_minute() >= MAX_TIME {
            // println!("Reached {MAX_TIME}! {self:?}");
            return (self.flow_rate, vec![self]);
        }

        let mut walks = Vec::new();
        let mut max: u32 = 0;
        let candidates = self.generate_candidates(graph, valves);
        // println!(
        //     "Candidates for {:?} + {:?}: {candidates:?}",
        //     self.path, self.current_node,
        // );
        for (node, _) in candidates {
            let mut current_walk = self.clone();
            let (current_max, mut resulting_walks) = match node {
                WalkStep::OpenedValve(_) => {
                    current_walk.open_valve(&valves);
                    current_walk.walk(graph, valves)
                }
                WalkStep::Tunnel(target) => {
                    current_walk.move_to_tunnel(target);
                    current_walk.walk(graph, valves)
                }
            };

            walks.append(&mut resulting_walks);
            if current_max < max {
                // Bound! It has no sense to try further candidates - according to the heuristic from `generate_candidates`
                println!("Bound!");
                break;
            } else if max == 0 || current_max > max {
                max = current_max;
            }
        }

        (max, walks)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");

    let valve_regex = Regex::new(
        r#"Valve\s+(?P<valve>[A-Z]{2}).*flow\s+rate=(?P<fr>[0-9]+).*to valves? (?P<targets>[A-Z, ]{2}(, [A-Z]{2})*)"#,
    )?;
    let valves = BufReader::new(File::open(&file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok())
        .map(|line| {
            let cap = valve_regex
                .captures(&line)
                .ok_or_else(|| Error::msg(format!("Did not match! '{line}'")))?;
            if let (Some(valve), Some(flow_rate), Some(targets)) =
                (cap.name("valve"), cap.name("fr"), cap.name("targets"))
            {
                Ok((
                    valve.as_str().to_string(),
                    Valve {
                        flow_rate: flow_rate
                            .as_str()
                            .parse()
                            .context("Failed to parse flow rate")?,
                        targets: targets
                            .as_str()
                            .split(",")
                            .map(|s| s.trim().to_string())
                            .collect(),
                    },
                ))
            } else {
                Err(Error::msg(format!(
                    "Line does not follow the format! '{line}'"
                )))
            }
        })
        .collect::<Result<ValveMap, _>>()?;

    let start_node = "AA";
    let graph: ValveGraph = GraphMap::<_, u32, petgraph::Directed>::from_edges(
        valves.iter().flat_map(|(id, valve)| {
            valve
                .targets
                .iter()
                .map(move |t| (id, t, (*valve).flow_rate))
        }), // .collect::<Vec<_>>(),
    );

    // {
    //     let dot = petgraph::dot::Dot::new(&graph);
    //     std::fs::write(format!("{file_name}.dot"), dot.to_string())?;
    // }

    // let floyd =
    //     petgraph::algo::floyd_warshall(&graph, |(_src, _dest, flow_rate)| u32::MAX - *flow_rate)
    //         .map_err(|_negative_cycles| Error::msg("Failed to calculates Floyd-Warshall"))?;

    // dbg!(&floyd);

    // let bf = petgraph::algo::bellman_ford(&graph, start_node)?;

    let initial_walk = GraphWalk::new(start_node);
    // dbg!(
    //     &initial_walk,
    //     initial_walk.generate_candidates(&graph, &valves)
    // );

    let (max, walks) = initial_walk.walk(&graph, &valves);
    dbg!(max, walks.first());

    Ok(())
}
