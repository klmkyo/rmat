use graph::{Propabilities, Graph};
use clap::Parser;
use std::time::Instant;

mod graph;

/// Program do generacji grafów
#[derive(Parser, Debug)]
#[command(author = "Marcin Klimek", version, about, long_about = None)]
struct Args {
    /// Czy graf ma być skierowany
    #[clap(short, long)]
    directed: bool,

    /// Czy graf może mieć pętle
    #[clap(short, long)]
    self_connections_allowed: bool,

    /// Liczba wierzchołków
    #[clap(short)]
    n: u32,

    /// Prawdopodobieństwo wybrania ćwiartki (format: [0.15, 0.2, 0.20, 0.45])
    #[clap(short, long)]
    propabilities: String,

    /// Docelowa gęstość grafu
    #[clap(short = 'g', long)]
    dest_density: f64,
}

fn main() {
    // let propabilities = Propabilities::new([0.15, 0.2, 0.20, 0.45]);
    // let mut graph = Graph::new(false, true, 9, propabilities, 0.40);

    // graph.fill();

    let args = Args::parse();
    // działa za pomocą FromStr trait
    let propabilities = args.propabilities.parse::<Propabilities>().unwrap();
    let mut graph = Graph::new(args.directed, args.self_connections_allowed, args.n, propabilities, args.dest_density);

    // let start = Instant::now();
    graph.fill();
    // println!("{}", start.elapsed().as_micros());

    graph.print();

    // get graph stats and print them
    let stats = graph.get_stats();
    println!("Density: {:.4}%", stats.density * 100.0);
    println!("Edges: {}", stats.edges);
    println!("Vertices: {}", stats.vertices);
    // since degrees are stored in a vector, which index is the vertex and the value is the degree,
    // print the vertex and its degree
    // stats.degrees.iter().enumerate().for_each(|(vertex, degree)| {
    //     println!("Vertex {} has degree {}", vertex, degree);
    // });

    // print the vertex with the highest degree
    let (vertex, degree) = stats.degrees.iter().enumerate().max_by(|(_, a), (_, b)| a.cmp(b)).unwrap();

    println!("Vertex {} has the highest degree with {}", vertex, degree);
}