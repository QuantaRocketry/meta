#[cfg(not(feature = "std"))]
use defmt::println;

fn main() {
    println!("Hello, world!");
}
