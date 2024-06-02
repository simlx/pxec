use std::env;
use std::fs::File;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::io::{BufWriter, Write};
use rand::Rng;
use std::process::Command;

#[derive(Clone)]
struct MapEntry {
    name: String,
    category: String,
    filehash: String
}

struct Config {
    editor: String
}

fn help() {
    println!("pxc help:");
    println!("list            -> ls [category]");
    println!("list categories -> lsc");
    println!("edit entry      -> edit [name] [category]");
    println!("add entry       -> add [name]");
    println!("export command  -> ext [name]");
    println!("remove entry    -> rm [name]");
}

fn gen_char_sequence() -> String {
    const CHARSET: &[u8] = b"ABCDEF0123456789";
    return (0..8)
        .map(|_| {
            let idx = rand::thread_rng().gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
    .collect();
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
    let config = read_config();
    let mut args = env::args().skip(1);

    if let Some(arg) = args.next() {
        match &arg[..] {
            "h" => help(),
            "add" => {
                let entry_name: String;
                if let Some(arg1) = args.next() {
                    entry_name = arg1;
                } else {
                    println!("[add] no name supplied, exiting.");
                    return;
                }

                if check_entry_exists(&entry_name, &entries) {
                    println!("[add] map entry with this name already exists, editing");
                    edit(&config, &entry_name, entries, "no-new-category");
                    return;
                }

                let entry_category: String;
                if let Some(arg1) = args.next() {
                    entry_category = arg1;
                } else {
                    entry_category = "default".to_string();
                    println!("[add] adding '{}' with default category", entry_name);
                }

                let mut char_sequence = gen_char_sequence();
                while check_sequence_exists(&char_sequence, &mut entries) {
                    println!("filehash {} already existed!, generating again..", &char_sequence);
                    char_sequence = gen_char_sequence();
                }

                add( MapEntry {name: entry_name.to_string(),
                category: entry_category.clone(), filehash: char_sequence}, &mut entries);

                ext(&entry_name, &mut entries);

                edit(&config,&entry_name, entries, &entry_category);
            },
            "edit" => {

                let entry_name: String;
                if let Some(arg1) = args.next() {
                    entry_name = arg1;
                } else {
                    println!("[edit] no name supplied, exiting.");
                    return;
                }

                let entry_category: String;
                if let Some(arg1) = args.next() {
                    entry_category = arg1;
                    println!("[edit] changing category to '{}'", entry_category);
                } else {
                    entry_category = "no-new-category".to_string();
                }

                edit(&config,&entry_name, entries, &entry_category);
            },
            "ext" => {

                let entry_name: String;
                if let Some(arg1) = args.next() {
                    entry_name = arg1;
                } else {
                    println!("[ext] no name supplied, exiting.");
                    return;
                }

                ext(&entry_name, &mut entries);
            },
            "rm" => {
                remove(&mut args, &mut entries);
            },
            "ls" | "list" => {
                let category: String = if let Some(arg1) = args.next() { arg1 } else {"".to_string() };
                list(&entries, &category);
            },
            "lsc" => {
                list_categories(&entries);
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

fn read_config() -> Config {

    let pxcpath = get_pxc_path();
    let config_path: String = pxcpath.to_owned() + "/config";
    let config_filepath: String = pxcpath.to_owned() + "/config/config";

    //default config values here
    let mut config: Config = Config{editor:"vim".to_string()};

    let mut editor_exists = false;

    if Path::new(&config_path).exists() {
        if let Ok(map_lines) = read_lines(&config_filepath) {
            for line in map_lines.flatten() {
                let parts = line.split(";").collect::<Vec<_>>();
                match parts[0] {
                    "editor" => {config.editor = parts[1].to_string(); editor_exists = true;}
                    _ => {}
                }
            }
        }
    } else {
        // config directory: stores config files
        match fs::create_dir_all(pxcpath.to_owned() + "/config/"){
            Ok(()) => {
            },
            Err(dir) => {println!("error when creating dir {}", dir)}
        }
    }

    if editor_exists { // only write, when a config is missing. add && for future config entries
        let mut file_buffer = File::create(&config_filepath).expect("unable to create file");
        write!(file_buffer, "{}\n", "editor;".to_owned() + &config.editor).expect("unable to write");
    }
    return config;
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

fn ext(entry_name: &str,  entries: &mut Vec<MapEntry>) {

    if !check_entry_exists(&entry_name, &entries) {
        println!("[ext] map entry with name '{}' doesn't exist!", entry_name);
        return;
    }

    if !cfg!(unix) {return; }

    for entry in entries {
        if entry.name == entry_name {

            let extcmdpath = get_ext_path() + entry_name + ".pxc";
            let cmdfilepath = get_pxc_path() + "/cmd/" + &entry.filehash;

            let file_buffer = File::create(&extcmdpath).expect("unable to create file");
            let mut file_buffer = BufWriter::new(file_buffer);

            write!(file_buffer, "{}\n", "exec ".to_owned() + &cmdfilepath + " \"$@\"").expect("unable to write");

            Command::new("chmod")
                .arg("777")
                .arg(&extcmdpath)
                .status()
                .expect("failed to execute process");

        }
    }
    println!("[ext] exported command '{}.pxc'", entry_name);
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

    let mut file_hash: String = "8723478546982389235".to_string();
    for entry in entries.clone() {
        if entry.name == entry_name {
            file_hash = entry.filehash.to_string();
        }
    }

    // remove command if exists.
    let extpath = get_pxc_path() + "/cmd/" + &file_hash;

    if Path::new(&extpath).exists() {
        Command::new("rm")
            .arg(&extpath)
            .status()
            .expect("failed to execute process");
    }

    entries.remove(entries.iter().position(|x| *&x.name == entry_name.to_string())
                   .expect("not found"));

    // remove external command if exists.
    let extpath = get_ext_path() + &entry_name + ".pxc";

    if Path::new(&extpath).exists() {
        Command::new("rm")
            .arg(&extpath)
            .status()
            .expect("failed to execute process");
    }

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
        println!("[add] map entry with this name already exists, this should not happen!");
        return;
    }
    if new_entry.category == "" {
        new_entry.category = "default".to_string();
    }

    Command::new("touch")
        .arg(get_pxc_path() + "/cmd/" +&new_entry.filehash)
        .status()
        .expect("failed to execute process");

    Command::new("chmod")
        .arg("777")
        .arg(get_pxc_path() + "/cmd/" +&new_entry.filehash)
        .status()
        .expect("failed to execute process");

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

// path for externalized commands

fn get_ext_path() -> String {
    return "/usr/local/bin/".to_string();
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

fn edit(config: &Config, entry_name: &str, mut entries: Vec<MapEntry>, category_name: &str) {

    for entry in &mut entries {
        if entry.name == entry_name {

            if category_name != "no-new-category" {
                entry.category = category_name.to_string();
            }

            println!("[edit] editing command '{}', filehash: {}", entry_name,  entry.filehash);
            let cmdpath = get_pxc_path().to_string() + "/cmd/" + &entry.filehash;

            Command::new(config.editor.to_string())
                .arg(cmdpath)
                .status()
                .expect("failed to execute process");
        }
    }

    save_map(&entries);
}

fn get_categories(entries: &Vec<MapEntry>) -> Vec<String> {

    let mut cat_list: Vec<String> =  Vec::new();

    for entry in entries {
        let mut found = false;
        for cat in &cat_list {
            if cat == &entry.category {
                found = true;
            }
        }
        if found == false {
            cat_list.push(entry.category.to_string());
        }
    }
    return cat_list;
}

fn list_categories(entries: &Vec<MapEntry>) {

    let cat_list: Vec<String> =  get_categories(entries);

    println!("categories:");
    for cat in cat_list {
        println!("{}", cat);
    }
}

fn list(entries: &Vec<MapEntry>, category_name: &str) {
    println!("NAME\t\tCATEGORY\tFILE");
    println!("----------------------------------------");

    if category_name != "" {
        for entry in entries.iter() {
            if entry.category == category_name {
                println!("{: <16}{: <16}{: <16}",entry.name,entry.category,entry.filehash);
            }
        }
        println!();
    }
    else {
        for category in get_categories(entries) {
            for entry in entries.iter() {
                if entry.category == category {
                    println!("{: <16}{: <16}{: <16}",entry.name,entry.category,entry.filehash);
                }
            }
            println!();
        }   
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
        file.write_all(b"test;test;00000000")?;

        println!("[init] init successful!");
    }

    Ok(())
}

