use std::env;
use std::fs::File;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::io::{BufWriter, Write};
use rand::Rng;
use std::process::Command;

struct MapEntry {
	name: String,
	category: String,
	filehash: String
}

fn help() {
	println!("pxc commands:");
	println!("-> list,ls");
	println!("-> edit");
	println!("-> rm");
}

fn gen_char_sequence() -> String {

    const CHARSET: &[u8] = b"ABCDEF0123456789";
    const PASSWORD_LEN: usize = 8;
    let mut rng = rand::thread_rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    println!("{:?}", password);

	return password;
}

fn check_sequence_exists(sequence: &str, entries: &mut Vec<MapEntry>) -> bool {
	for entry in entries {
		if sequence == entry.filehash {
			return true;
		}
	}
	return false;
}

fn main() {

	let mut entries: Vec<MapEntry> = read_map_file();
	let mut args = env::args().skip(1);

	if let Some(arg) = args.next() {
		match &arg[..] {
			"h" => help(),
			"edit" => {
				let entry_name: String;
				let entry_category: String;

				if let Some(arg1) = args.next() {
					entry_name = arg1;
				} else {
					println!("[add] no name supplied, exiting.");
					return;
				}

				if let Some(arg1) = args.next() {
					entry_category = arg1;
				} else {
					println!("[add] adding {} with default category", entry_name);
					entry_category = "default".to_string();
				}
					
				let mut char_sequence = gen_char_sequence();
				while check_sequence_exists(&char_sequence, &mut entries) {
					println!("filehash {} already existed!, generating again..", &char_sequence);
					char_sequence = gen_char_sequence();
				}

				add( MapEntry {name: entry_name.to_string(),
						category: entry_category,filehash: char_sequence}, &mut entries);
				edit(&entry_name, entries);
			},
			"rm" => {
				remove(&mut args, &mut entries);
			},
			"ls" | "list" => list(&entries), // pxc ls category(optional)
			"pkg" => {
				if let Some(arg1) = args.next() {
					match &arg1[..] {
						"install" | "in" | "i" => println!("pkg not yet implemented"),
						"remove" | "rm" | "r" => println!("pkg not yet implemented"),
						_ => println!("cpkg: invalid arg {}", arg1)
					}
				} else {
					println!("[pkg] specify install or remove");
				}
			},
			_ => {
					run_cmd(&arg, &mut args, entries);
			}
		}
	} else {
		help();
	}
}


fn run_cmd(arg: &str, args: &mut core::iter::Skip<crate::env::Args>,  entries: Vec<MapEntry>) {
	match get_entry_by_name(&arg, entries) {
		Some(ent) => {
			println!("running command '{}', filehash: {}", arg,  ent.filehash);
			let cmdpath = get_pxc_path().to_string() + "/cmd/" + &ent.filehash;
			println!("cmd path: '{}'", cmdpath);

			Command::new("chmod")
			.arg("777")
			.arg(&cmdpath)
			.status()
			.expect("failed to execute process");

			let mut cmdargs = "".to_owned();

			loop {
				match args.next() {
					Some(carg) => {
						cmdargs.push_str(&carg.to_owned());
						cmdargs.push_str(" ");
						
					},
					None => {break;}
				};
			}

			println!("cargs: {}", cmdargs);
			Command::new("sh")
			.arg("-c")
			.arg(cmdpath + " " + &cmdargs)
			.status()
			.expect("failed to execute process");
		},
		None  => println!("command '{}' not found", arg)
	};
}

fn read_map_file() -> Vec<MapEntry> {

	let mut result: Vec<MapEntry> = Vec::new();
	let map_file = get_pxc_path() + "/map/pxc";

	if let Ok(map_lines) = read_lines(map_file) {
		for line in map_lines.flatten() {
			let parts = line.split(";").collect::<Vec<_>>();
			result.push(MapEntry 
				{
					name:     parts[0].to_string(),
					category: parts[1].to_string(),
					filehash: parts[2].to_string()
				}
			);
		}
	} else {
		println!("pxc not initialized!");
		match init() {
			Ok(_) => {},
			Err(e) => {println!("Could not initialize!: {}", e)}
		}
	}

	return result;
}

fn remove(args: &mut core::iter::Skip<crate::env::Args>, entries: &mut Vec<MapEntry>) {

	let entry_name;

	if let Some(arg1) = args.next() {
		entry_name = arg1;
	} else {
		println!("[rm] no name supplied, exiting.");
		return;
	}

	if !check_entry_exists(&entry_name, &entries) {
		println!("[rm] map entry with name '{}' doesn't exist!", entry_name);
		return;
	}

	entries.remove(entries.iter().position(|x| *&x.name == entry_name.to_string())
			.expect("not found"));

	save_map(&entries);

	println!("[rm] removed {}!", entry_name);
}

fn check_entry_exists(entry_name: &str, entries: &Vec<MapEntry>) -> bool {
	for entry in entries {
		if entry.name == entry_name {
			return true;
		}
	}
	return false;
}

fn get_entry_by_name(entry_name: &str, entries: Vec<MapEntry>) -> Option<MapEntry> {
	for entry in entries {
		if entry.name == entry_name {
			return Some(entry);
		}
	}
	return None;
}


fn add(mut new_entry: MapEntry, entries: &mut Vec<MapEntry>) {
	if check_entry_exists(&new_entry.name, entries) {
		println!("[add] map entry with this name already exists!");
		return;
	}
	if new_entry.category == "" {
		new_entry.category = "default".to_string();
	}

	entries.push(new_entry);
	save_map(entries);
}

fn get_pxc_path() -> String {
	match home::home_dir() {
		Some(path) if !path.as_os_str().is_empty() => 
			return  path.as_os_str().to_str().unwrap().to_string() + "/.pxc",
			_ => { 
				println!("Unable to get pxc path!");
				return "".to_string(); 
			}
	}
}

fn save_map(entries: &Vec<MapEntry>) {
	if !cfg!(unix) {return; }

	let newfilepath = get_pxc_path() + "/map/pxc";

	if !Path::new(&newfilepath).exists() {
		println!("[save] map file doesn't exist!");
		return;
	}

    let file_buffer = File::create(&newfilepath).expect("unable to create file");
    let mut file_buffer = BufWriter::new(file_buffer);

    for entry in entries {
		let entry_line = "".to_owned() + &entry.name + ";" +
										 &entry.category + ";" + 
										 &entry.filehash;
        write!(file_buffer, "{}\n", entry_line).expect("unable to write");
    }

	println!("[save] file saved!");
}

fn edit(entry_name: &str, entries: Vec<MapEntry>) {
	match get_entry_by_name(&entry_name, entries) {
		Some(ent) => {
			println!("[edit] editing command '{}', filehash: {}", entry_name,  ent.filehash);
			let cmdpath = get_pxc_path().to_string() + "/cmd/" + &ent.filehash;
			println!("cmd path: '{}'", cmdpath);

			Command::new("vim")
			.arg(cmdpath)
			.status()
			.expect("failed to execute process");
		},
		None  => println!("command '{}' not found", entry_name)
	};
}

fn list(entries: &Vec<MapEntry>) {
	println!("NAME\t\tCATEGORY\tFILE");
	println!("----------------------------------------");
	for entry in entries.iter() {
		println!("{}\t\t{}\t\t{}",entry.name,entry.category,entry.filehash);
	}
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> where P: AsRef<Path>, {
	let file = File::open(filename)?;
	Ok(io::BufReader::new(file).lines())
}

fn init() -> std::io::Result<()> {

	println!("[init] initializing pxc..");

	if cfg!(windows) {
		// windows todo
		return Ok(());

	} else if cfg!(unix) {

		let pxcpath = get_pxc_path();

		// pxc directory: root pxec directory
		match fs::create_dir_all(&pxcpath){
			Ok(()) => {},
			Err(dir) => {println!("error when creating dir {}", dir)}
		}
		// map directory: stores the mapping of command to script
		match fs::create_dir_all(pxcpath.to_owned() + "/map/"){
			Ok(()) => {},
			Err(dir) => {println!("error when creating dir {}", dir)}
		}
		// commands directory: stores all script files
		match fs::create_dir_all(pxcpath.to_owned() + "/cmd/"){
			Ok(()) => {},
			Err(dir) => {println!("error when creating dir {}", dir)}
		}

		let newfilepath: String = pxcpath.to_owned() + "/map/pxc";
		if Path::new(&newfilepath).exists() {
			println!("[init] file already exists");
			return Ok(());
		}

		let mut file = File::create(&newfilepath)?;
		file.write_all(b"test;test;00000000!")?;

		println!("[init] init successful!");
	}

	Ok(())
}

