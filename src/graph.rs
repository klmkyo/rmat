use std::{fmt::{Debug, Formatter}, ops::Deref, fs::File, io::Write, str::FromStr};
use colored::Colorize;


// 4 f64's in an array
pub struct Propabilities([f64; 4]);

impl Deref for Propabilities {
    type Target = [f64; 4];
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Propabilities {
    pub fn new(vals: [f64; 4]) -> Propabilities {
        // sprawdzenie czy prawdopodobieństwa sumują się do 1
        if vals.iter().sum::<f64>() != 1.0 {
            panic!("Propabilities must sum to 1 (they sum to {})", vals.iter().sum::<f64>());
        }
        Propabilities(vals)
    }

    pub fn get_random_quarter(&self) -> u8 {
        let r = fastrand::f64();
        if r < self[0] {
            0
        } else if r < self[0] + self[1] {
            1
        } else if r < self[0] + self[1] + self[2] {
            2
        } else {
            3
        }
    }
}

impl FromStr for Propabilities {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals = s
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<f64>, _>>()
            .map_err(|_| "Error parsing propabilites, the format is: [0.1, 0.2, 0.3, 0.4]".to_string())?;
        
        if vals.len() != 4 {
            return Err("Expected 4 propabilites".to_string());
        }
        
        Ok(Propabilities::new([vals[0], vals[1], vals[2], vals[3]]))
    }
}

pub struct GraphStats {
    pub density: f64,
    pub edges: usize,
    pub vertices: usize,
    pub degrees: Vec<u32>,
}

pub struct Graph {
	directed: bool,
	self_connections_allowed: bool,

    // tablica 2d reprezentująca połączenia między wierzchołkami
    // false oznacza brak połączenia, true oznacza połączenie
    connections: Vec<Vec<bool>>,

    n: u32,

    // prawdobodobieństwa ćwiartek
    propabilities: Propabilities,

    // docelowa gęstość grafu
    dest_density: f64,
}

impl Graph {

    // metoda tworząca nowy graf o podanej liczbie wierzchołków
    pub fn new(directed: bool, self_connections_allowed: bool, n: u32, propabilities: Propabilities, dest_density: f64) -> Graph {
        // tablica 2^n wypełniona zerami
        let connections = vec![vec![false; 2_usize.pow(n)]; 2_usize.pow(n)];
        Graph { directed, self_connections_allowed, connections, propabilities, n, dest_density }
    }

    // fills the graph with connections according to the R-Mat algorithm
    // the desired density of the graph is given in the dest_density parameter
    pub fn fill(&mut self) {
        // how many fields need to be filled
        // all fields besides the diagonal from top left to bottom right
        // (2^n * 2^n) - 2^n = 4^n - 2^n

		let mut total_fillable = 4_usize.pow(self.n) - 2_usize.pow(self.n);

		if !self.directed {
			// if is undirected, the graph is symmetrical
			// so we only need to fill half of the graph
			total_fillable /= 2;
		}

        if self.self_connections_allowed {
            // if self connections are allowed, we need to add the diagonal
            total_fillable += 2_usize.pow(self.n);
        }

        println!("Total fillable: {}", total_fillable);

        let to_fill = (total_fillable as f64 * self.dest_density) as usize;

        // how many fields have been filled
        let mut filled = 0;

        // while loop to fill the graph
        while self.poke() {
            filled += 1;
            if filled >= to_fill {
                break;
            }
        }

        println!("Filled {} fields", filled);
    }

    // sets random bool in the array to true
    // it wil select the random bool using the propabilities
    // if the found bool is already true, it will find another one
    pub fn poke(&mut self) -> bool {
        let size = self.connections.len();
        self.poke_range(0, 0, size, !self.directed)
    }

    // sets random bool in the array to true
    // call itself recursively until it finds a false bool
    // each call it shrinks the range to a quarter selected by propability
    // a: top left, b: top right, c: bottom left, d: bottom right
    // returns true if it found a false bool and set it to true
    pub fn poke_range(&mut self, start_x: usize, start_y: usize, size: usize, ignore_b: bool) -> bool {
        // check if range is actually just one element
        // the difference between start and end for both x and y should be the same
        // so we can just check one of them
        if size == 1 {
			// if self connections are not allowed, ignore the diagonal
			if !self.self_connections_allowed && start_x == start_y {
				return false;
			}

            // check if it is already true
            if self.connections[start_y][start_x] {
                // if it is true, return false
                return false;
            } else {
                // if it is false, set it to true and return true
                self.connections[start_y][start_x] = true;
                return true;
            }
        }

        // keep track of what quarters have been searched
        let mut searched = [false; 4];
		// if the graph is undirected, the b and c quarters are the same, so we only need to search one of them
		if ignore_b {
			// set the b quarter to true for it to be ignored
			searched[1] = true;
		}

        // while loop to find a false bool
        loop {
			// self.print();

            // select the quarter
            // a (0): top left, b (1): top right, c (2): bottom left, d (3): bottom right
            let mut quarter = self.propabilities.get_random_quarter();

			// if the graph is undirected, the b and c quarters are the same
			// so if quarter is b, we need to set it to c
			// 
			// REPORT NOTE: this could be optimized further by making different functions for directed and undirected graphs
			// 				but for readability's sake I decided to keep it like this
			
			if ignore_b && (quarter == 1) {
				quarter = 2;
			}

            // check if the quarter has been searched
            // quarters are represented by numbers
            if searched[quarter as usize] {
                // if it has been searched, continue the loop
                continue;
            } else {
                // if it has not been searched, set it to true
                searched[quarter as usize] = true;
            }

			let size = size / 2;
			let mut start_x = start_x;
			let mut start_y = start_y;

			if quarter % 2 != 0 {
				start_x += size;
			}
			if quarter > 1 {
				start_y += size;
			}

            // based on the quarter, calculate the start and end of the range
            // let (start_x, start_y) = match quarter {
            //     0 => {
			// 		// a: top left
            //         let start_x = start_x;
			// 		let start_y = start_y;
			// 		(start_x, start_y)
			// 	},
			// 	1 => {
			// 		// b: top right
			// 		let start_x = start_x + size;
			// 		let start_y = start_y;
			// 		(start_x, start_y)
			// 	},
			// 	2 => {
			// 		// c: bottom left
			// 		let start_x = start_x;
			// 		let start_y = start_y + size;
			// 		(start_x, start_y)
			// 	},
			// 	3 => {
			// 		// d: bottom right
			// 		let start_x = start_x + size;
			// 		let start_y = start_y + size;
			// 		(start_x, start_y)
			// 	},
            //     _ => panic!("Invalid quarter"),
            // };

			let ignore_b = match ignore_b {
				// keep ignoring b unless you've gone to c (bottom left)
				true => quarter != 2,

				// if b is not ignored, keep it that way
				false => false,
			};

            // call itself on the reduced range
            if self.poke_range(start_x, start_y, size, ignore_b) {
                // if it flipped a false bool, return true
                return true;
            }

            // if all quarters have been tried, return false
            // check if there is any false bool in the searched array
            if searched.iter().all(|&x| x == true) {
                return false;
            }
        }
    }

    // prints the vector of vectors with numbered rows and columns, 1 for true and 0 for false
    // row and column labels are separated from 1s and 0s by a | and -
    // 1 is colored red, 0 is white
    pub fn print(&self) {

        // print the row labels and the 1s and 0s
		if self.directed {
			for i in 0..self.connections.len() {
				for j in 0..self.connections.len() {

					// if is on the diagonal and self_connections are not allowed, print it as gray (1 or 0). otherwise if 1, print it as red, otherwise print it as white
					if !self.self_connections_allowed && i == j {
						print!("{} ", (self.connections[i][j] as u8).to_string().truecolor(120, 120, 120));
					} else if self.connections[i][j] {
						print!("{} ", (self.connections[i][j] as u8).to_string().red());
					} else {
						print!("{} ", (self.connections[i][j] as u8));
					}

				}
                println!();
			}
		} else {
			for i in 0..self.connections.len() {
				for j in 0..self.connections.len() {
					// if is on the diagonal or after that, print it as gray (1 or 0). otherwise if 1, print it as red, otherwise print it as white
					if (if self.self_connections_allowed {i < j} else {i <= j}) {
						print!("{} ", (self.connections[i][j] as u8).to_string().truecolor(120, 120, 120));
					} else if self.connections[i][j] {
						print!("{} ", (self.connections[i][j] as u8).to_string().red());
					} else {
						print!("{} ", (self.connections[i][j] as u8));
					}

				}
                println!();
			}
		}
    }

    pub fn get_stats(&self) -> GraphStats {
        // count the number of edges (true values in the array)
        let edges = self.connections
            .iter()
            .map(
                |row| row.iter().filter(|&x| *x).count()
            )
            .sum::<usize>();
        let vertices = self.connections.len();
        // calculate the density
        let mut density = {
            let vertices = vertices as f64;
            if self.self_connections_allowed {
                // if self connections are allowed, divide by vertices * vertices
                if self.directed {
                    // if directed, divide by vertices * vertices
                    edges as f64 / (vertices * vertices)
                } else {
                    // if undirected, divide by vertices * (vertices - 1)
                    edges as f64 / (vertices * (vertices + 1.0))
                }
            } else {
                // if self connections are not allowed, you cannot connect to yourself, so there
                // is one less vertex to connect to
                edges as f64 / (vertices * (vertices - 1.0))
            }
        };

        if !self.directed {
            // if undirected, multiply by 2
            density *= 2.0;
        }

        // count the number of edges for each vertex
        let mut degrees: Vec<u32> = vec![0; vertices];

        if self.directed {
            for y in 0..self.connections.len() {
                for x in 0..self.connections.len() {
                    let vertex = y;

                    if self.connections[y][x] {
                        degrees[vertex] += 1;
                    }
                }
            }
        } else {
            for mut y in 0..self.connections.len() {
                for mut x in 0..self.connections.len() {
                    let vertex = y;
                    // map the top right triangle to the bottom left triangle
                    if x > y {
                        // swap x and y
                        (x, y) = (y, x);
                    }

                    if self.connections[y][x] {
                        degrees[vertex] += 1;
                    }
                }
            }
        }

        GraphStats {
            density,
            edges,
            vertices,
            degrees,
        }
    }

	// saves the graph to a file using its string representation
	pub fn save(&self, filename: &str) {
		let mut file = File::create(filename).unwrap();
		file.write_all(self.to_string().as_bytes()).unwrap();
	}
}

// debug display
impl Debug for Graph {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for i in 0..self.connections.len() {
            for j in 0..self.connections.len() {
                if self.connections[i][j] {
                    write!(f, "1 ")?;
                } else {
                    write!(f, "0 ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

// to string
impl ToString for Graph {
	fn to_string(&self) -> String {
		let mut string = String::new();
		for i in 0..self.connections.len() {
			for j in 0..self.connections.len() {
				if self.connections[i][j] {
					string.push_str("1 ");
				} else {
					string.push_str("0 ");
				}
			}
			string.push('\n');
		}
		string
	}
}