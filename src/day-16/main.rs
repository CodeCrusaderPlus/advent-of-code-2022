use itertools::Itertools;
use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::character::complete as cc;
use nom::combinator::{all_consuming, map_res};
use nom::multi::separated_list1;
use nom::sequence::{separated_pair, tuple};
use nom::{bytes::complete::tag, Finish, IResult};
use petgraph::algo::floyd_warshall;
use petgraph::{prelude::*, Directed, Graph};

#[derive(Debug)]
struct Valve {
  name: String,
  flow_rate: i32,
  tunnel_valves: Vec<String>,
}

fn main() {
  let valves = all_consuming(parse_all_valves)(include_str!("input.txt"))
    .finish()
    .unwrap()
    .1;

  let graph = build_graph_and_weight_map(&valves).0;
  let weight_map = build_graph_and_weight_map(&valves).1;
  let inf = i32::MAX;

  let res = floyd_warshall(&graph, |edge| {
    if let Some(weight) = weight_map.get(&(edge.source(), edge.target())) {
      *weight
    } else {
      inf
    }
  })
  .unwrap();

  println!(
    "The maximum flow rate is {:#?}",
    res.iter().clone().collect_vec()
  );
}

fn build_graph_and_weight_map(
  valves: &Vec<Valve>,
) -> (Graph<(), ()>, HashMap<(NodeIndex, NodeIndex), i32>) {
  let mut graph: Graph<(), (), Directed> = Graph::new();
  let mut valves_graph: Vec<(NodeIndex, NodeIndex)> = Vec::new();
  let mut valves_graph_weight: HashMap<(NodeIndex, NodeIndex), i32> = HashMap::new();

  let nodes: HashMap<&String, NodeIndex> =
    HashMap::from_iter(valves.iter().map(|valve| (&valve.name, graph.add_node(()))));

  valves.iter().for_each(|valve| {
    valve.tunnel_valves.iter().for_each(|tunnel_valve_name| {
      valves_graph.push((
        *nodes.get(&valve.name).unwrap(),
        *nodes.get(&tunnel_valve_name).unwrap(),
      ));
    })
  });
  graph.extend_with_edges(&valves_graph);

  valves.iter().for_each(|valve| {
    valves_graph_weight.insert(
      (
        *nodes.get(&valve.name).unwrap(),
        *nodes.get(&valve.name).unwrap(),
      ),
      0,
    );
    valve.tunnel_valves.iter().for_each(|tunnel_valve_name| {
      valves_graph_weight.insert(
        (
          *nodes.get(&valve.name).unwrap(),
          *nodes.get(&tunnel_valve_name).unwrap(),
        ),
        valve.flow_rate,
      );
    })
  });
  println!("{:#?}", valves_graph_weight);
  return (graph, valves_graph_weight);
}

fn parse_all_valves(i: &str) -> IResult<&str, Vec<Valve>> {
  return separated_list1(cc::newline, parse_valve)(i);
}

fn parse_valve(input: &str) -> IResult<&str, Valve> {
  // Valve QJ has flow rate=11; tunnels lead to valves HB, GL
  let (input, result) = separated_pair(
    tuple((
      tag("Valve "),
      map_res(take_while(|c: char| c.is_alphabetic()), |s: &str| {
        s.parse::<String>()
      }),
      tag(" has flow rate="),
      map_res(take_while(|c: char| c.is_numeric()), |s: &str| {
        s.parse::<i32>()
      }),
    )),
    tag("; "),
    tuple((
      alt((tag("tunnels "), tag("tunnel "))),
      alt((tag("lead "), tag("leads "))),
      alt((tag("to valves "), tag("to valve "))),
      separated_list1(
        tag(", "),
        map_res(take_while(|c: char| c.is_alphabetic()), |s: &str| {
          s.parse::<String>()
        }),
      ),
    )),
  )(input)?;

  Ok((
    input,
    Valve {
      #[rustfmt::skip]
      name: result.0.1,
      #[rustfmt::skip]
      flow_rate: result.0.3,
      #[rustfmt::skip]
      tunnel_valves: result.1.3,
    },
  ))
}