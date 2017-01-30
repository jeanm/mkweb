extern crate mkweb;

use std::env;
use std::io::Write;
use mkweb::Renderer;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let renderer = match Renderer::new(&args[1]) {
            Ok(r) => r,
            Err(e) => panic!("{}", e)
        };
        match renderer.render() {
            Err(e) => panic!("{}", e),
            Ok(()) => (),
        }
    } else {
        writeln!(
            &mut std::io::stderr(),
            "syntax: mkweb <root_directory>"
        ).unwrap(); 
    }
}