use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::fs::read_to_string;
use std::env;
/*
* TODO:
*
*   read fn
*   save fn
*
*   - run in  tui
*   - run by argument
*   - edit command
*   - help command
*   - version command
*   - list command (ls)
*
*/

fn main() {
    println!("init");

    let args: Vec<String> = env::args().skip(1).collect();
    for arg in &args {
        match arg.as_str() {
            "help" => println!("x <alias>"),
            "edit" => edit_mapfile(),
            _ => execute(arg.to_string()),
        }
    }
    
    let lines = load();

    // print test file content 
        println!("example content: ");
    for line in lines {
        println!("{}", line);
    }

}

fn execute(arg: String) {
    println!("executing {}", arg);
}

fn edit_mapfile() {}

fn save() {}

fn load() -> Vec<String> {
    read_lines("test.test")
}

fn read_lines(filename: &str) -> Vec<String> {
    read_to_string(filename) 
    .unwrap()                    // panic on possible file-reading errors
    .lines()                     // split the string into an iterator of string slices
    .map(String::from)           // make each slice into a string
    .collect()                   // gather them together into a vector
}

