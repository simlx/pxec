extern crate ncurses;
extern crate rand;
use ncurses::*;
use rand::Rng;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::fs::Permissions;
use std::fs::{self};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::ops::Deref;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::ExitStatus;
use std::process::{Command, Stdio};
use std::ptr::null;
use std::ptr::null_mut;

#[derive(Clone)]
struct MapEntry {
    name: String,
    category: String,
    filehash: String,
}

struct Config {
    editor: String,
}

fn help() {
    println!("pxc help:");
    println!("");
    println!("lsc                    -> List all categories.");
    println!("(ls | list)            -> List all commands.");
    println!("(ls | list) <name>     -> List all commands in category <name>.");
    println!("edit <name>            -> Edit the command <name>.");
    println!("add <name>             -> Add a new command with the name <name>.");
    println!("print <name>           -> Print the content of the command <name>.");
    println!("ext | external         -> Export the command <name>.");
    println!("rm | remove            -> Remove the command <name>.");
}

fn gen_char_sequence() -> String {
    const CHARSET: &[u8] = b"ABCDEF0123456789";
    return (0..8)
        .map(|_| {
            let idx = rand::thread_rng().gen_range(0, CHARSET.len());
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
            "h" | "help" | "--help" => help(),
            "print" => {
                let entry_name: String;
                if let Some(arg1) = args.next() {
                    entry_name = arg1;
                } else {
                    println!("[print] no name supplied, exiting.");
                    return;
                }
                if !check_entry_exists(&entry_name, &entries) {
                    println!("[print] item with name '{}' doesn't exist", entry_name);
                    return;
                }
                print_cmd(&entry_name, entries);
            }

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
                    edit(&config, &entry_name, &mut entries, "no-new-category");
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
                    println!(
                        "filehash {} already existed!, generating again..",
                        &char_sequence
                    );
                    char_sequence = gen_char_sequence();
                }

                add(
                    MapEntry {
                        name: entry_name.to_string(),
                        category: entry_category.clone(),
                        filehash: char_sequence,
                    },
                    &mut entries,
                );

                ext(&entry_name, &mut entries);

                edit(&config, &entry_name, &mut entries, &entry_category);
            }
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

                edit(&config, &entry_name, &mut entries, &entry_category);
            }
            "ext" => {
                let entry_name: String;
                if let Some(arg1) = args.next() {
                    entry_name = arg1;
                } else {
                    println!("[ext] no name supplied, exiting.");
                    return;
                }

                ext(&entry_name, &mut entries);
            }
            "rm" => {
                remove(&mut args, &mut entries);
            }
            "ls" | "list" => {
                let category: String = if let Some(arg1) = args.next() {
                    arg1
                } else {
                    "".to_string()
                };
                list(&entries, &category);
            }
            "lsc" => {
                list_categories(&entries);
            }
            "interactive" | "int" => {
                /* Setup ncurses. */
                initscr();
                raw();

                /* Allow for extended keyboard (like F1). */
                keypad(stdscr(), true);
                noecho();

                /* Invisible cursor. */
                curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

                let mut search_word = "".to_string();

                mvprintw(LINES() - 1, 0, "Press Escape to exit").unwrap();
                refresh();

                /* Get the screen bounds. */
                let mut max_x = 0;
                let mut max_y = 0;
                getmaxyx(stdscr(), &mut max_y, &mut max_x);

                let mut in_loop = true;

                let mut key_e = 'e' as i32;

                let mut last_max_y = 0;

                while in_loop == true {
                    let mut ch = getch();

                    match ch {
                        27 => {
                            // escape
                            in_loop = false;
                        }
                        263 => {
                            // backspace
                            let mut chars = search_word.chars();
                            chars.next_back();
                            search_word = chars.as_str().to_string();
                            let mut search_word_display = "".to_string();
                            search_word_display.push_str("search: '");
                            search_word_display.push_str(&search_word.clone());
                            search_word_display.push_str("'");
                            mvprintw(2, 0, "                                                                  ").unwrap();

                            mvprintw(2, 0, &search_word_display.as_str()).unwrap();

                            let mut print_y = 3;

                            for i in 3..last_max_y + 1 {
                                mvprintw(i, 0, "                                                                  ").unwrap();
                            }
                            for entry in find_entries_containing(&entries, search_word.clone()) {
                                print_y += 1;
                                mvprintw(print_y, 0, "                                                                  ").unwrap();
                                if print_y == 1 {
                                    mvprintw(print_y, 0, &("-> ".to_owned() + &entry.as_str()))
                                        .unwrap();
                                } else {
                                    mvprintw(print_y, 0, &entry.as_str()).unwrap();
                                }
                            }
                            last_max_y = print_y;
                        }
                        10 => {
                            //enter
                            //endwin();
                            //in_loop = false;
                            // Run command if exact match

                            let mut found_entries =
                                find_entries_containing(&entries, search_word.clone());
                            found_entries.sort_by(|a, b| a.len().cmp(&b.len()));

                            run_cmd(found_entries.get(0).unwrap(), &mut args, &entries);

                            in_loop = false;
                        }
                        _ => {
                            //search_word += ch.to_string().as_str();
                            search_word +=
                                std::char::from_u32(ch as u32).unwrap().to_string().as_str();

                            let mut search_word_display = "".to_string();
                            search_word_display.push_str("search: '");
                            search_word_display.push_str(&search_word.clone());
                            search_word_display.push_str("'");

                            mvprintw(2, 0, &search_word_display.as_str()).unwrap();

                            let mut print_y = 3;

                            for i in 3..last_max_y + 1 {
                                mvprintw(i, 0, "                                                                  ").unwrap();
                            }

                            let mut found_entries =
                                find_entries_containing(&entries, search_word.clone());
                            found_entries.sort_by(|a, b| a.len().cmp(&b.len()));

                            for entry in found_entries {
                                print_y += 1;
                                mvprintw(print_y, 0, "                                                                  ").unwrap();
                                if print_y == 1 {
                                    mvprintw(print_y, 0, &("-> ".to_owned() + &entry.as_str()))
                                        .unwrap();
                                } else {
                                    mvprintw(print_y, 0, &entry.as_str()).unwrap();
                                }
                            }
                            last_max_y = print_y;
                        }
                    }
                }

                endwin();
            }
            _ => {
                let cmd = arg;

                // Run command if exact match
                if check_entry_exists(&cmd, &entries) {
                    run_cmd(&cmd, &mut args, &entries);
                    return;
                }

                let possible_cmds = find_entries_containing(&entries, cmd);

                if possible_cmds.len() == 0 {
                    println!("Command not found");
                    return;
                }

                if possible_cmds.len() > 1 {
                    println!("Did you mean one of:");
                }

                let mut choice_entries = Vec::new();
                let mut counter = 0;
                for cmd in possible_cmds {
                    counter += 1;
                    println!("{}. ->{}", counter, cmd);
                    choice_entries.push(cmd);
                }
                let mut input_text = String::new();

                if choice_entries.len() > 1 {
                    println!("select: ");
                } else {
                    println!("Press Enter to run {}", choice_entries.get(0).unwrap());
                }

                io::stdin()
                    .read_line(&mut input_text)
                    .expect("failed to read from stdin");

                let trimmed = input_text.trim();
                match trimmed.parse::<u32>() {
                    Ok(i) => run_cmd(
                        &choice_entries.get(i as usize - 1).unwrap(),
                        &mut args,
                        &entries,
                    ),
                    Err(..) => {
                        if trimmed == "" && choice_entries.len() == 1 {
                            run_cmd(&choice_entries.get(0).unwrap(), &mut args, &entries)
                        } else {
                            println!("invalid option: {}", &trimmed);
                        }
                    }
                };
            }
        }
    } else {
        help();
    }
}

fn find_entries_containing(mut entries: &Vec<MapEntry>, mut chars: String) -> Vec<String> {
    return entries
        .clone()
        .into_iter()
        .filter(|entry| entry.name.contains(&chars))
        .map(|entry| entry.name)
        .collect();
}

fn run_cmd(arg: &str, args: &mut core::iter::Skip<crate::env::Args>, entries: &[MapEntry]) {
    match get_entry_by_name(arg, entries) {
        Some(ent) => {
            println!("Running command '{}' with filehash: {}", arg, ent.filehash);

            let cmdpath = format!("{}/cmd/{}", get_pxc_path(), ent.filehash);

            let cmdargs = args.collect::<Vec<_>>().join(" ");
            println!("Command arguments: {}", cmdargs);

            let status = execute_command(&cmdpath, &cmdargs);

            match status {
                Ok(status_code) => {
                    if !status_code.success() {
                        println!("Command execution failed with status: {}", status_code);
                    }
                }
                Err(e) => {
                    println!("Failed to run command: {}", e);
                }
            }
        }
        None => println!("Command '{}' not found", arg),
    }
}

fn execute_command(cmdpath: &str, cmdargs: &str) -> Result<ExitStatus, String> {
    let command_str = format!("{} {}", cmdpath, cmdargs);

    // Use sh -c for flexibility, but consider possible security risks
    let status = Command::new("sh")
        .arg("-c")
        .arg(command_str.clone()) // Clone here to avoid moving it
        .status();

    match status {
        Ok(status_code) => Ok(status_code),
        Err(e) => Err(format!(
            "Failed to execute command '{}': {}",
            command_str, e
        )),
    }
}

fn get_entry_by_name<'a>(entry_name: &str, entries: &'a [MapEntry]) -> Option<&'a MapEntry> {
    entries.iter().find(|entry| entry.name == entry_name)
}

fn read_config() -> Config {
    let pxcpath = get_pxc_path();
    let config_path = format!("{}/config", pxcpath);
    let config_filepath = format!("{}/config/config", pxcpath);

    // Default config values
    let mut config = Config {
        editor: "vim".to_string(),
    };

    // Check if config directory exists, if not, create it
    if !Path::new(&config_path).exists() {
        if let Err(e) = fs::create_dir_all(&config_path) {
            println!("Error when creating dir: {}", e);
        }
    }

    // Read the config file if it exists
    if let Ok(map_lines) = read_lines(&config_filepath) {
        for line in map_lines.flatten() {
            if let Some((key, value)) = line.split_once(';') {
                match key {
                    "editor" => {
                        config.editor = value.to_string();
                    }
                    _ => {}
                }
            }
        }
    }

    // Write the editor value to the config file if it has been modified
    if !config.editor.is_empty() {
        if let Err(e) = File::create(&config_filepath)
            .and_then(|mut file| write!(file, "editor;{}\n", config.editor))
        {
            println!("Unable to write to config file: {}", e);
        }
    }

    config
}

fn read_map_file() -> Vec<MapEntry> {
    let map_file = format!("{}/map/pxc", get_pxc_path());

    // Try reading the file and processing the lines
    match read_lines(map_file) {
        Ok(map_lines) => {
            let mut result = Vec::new();
            for line in map_lines.flatten() {
                let parts = line.split(';').collect::<Vec<_>>();
                result.push(MapEntry {
                    name: parts[0].to_string(),
                    category: parts[1].to_string(),
                    filehash: parts[2].to_string(),
                });
            }
            result
        }
        Err(_) => {
            // In case of an error, return an empty Vec instead of returning a wrong type
            println!("Error reading map file.");
            Vec::new()
        }
    }
}

fn ext(entry_name: &str, entries: &mut Vec<MapEntry>) {
    if let Some(entry) = entries.iter().find(|e| e.name == entry_name) {
        // Entry found
        if !cfg!(unix) {
            return;
        }

        // Construct paths using Path::join to avoid string concatenation
        let extcmdpath = Path::new(&get_ext_path()).join(format!("{}.!", entry_name));
        let cmdfilepath = Path::new(&get_pxc_path()).join("cmd").join(&entry.filehash);

        // Try to create and write to the file
        match File::create(&extcmdpath) {
            Ok(file) => {
                let mut file_buffer = BufWriter::new(file);
                if let Err(e) = write!(file_buffer, "exec \"{}\" \"$@\"\n", cmdfilepath.display()) {
                    println!(
                        "[ext] failed to write to file '{}': {}",
                        extcmdpath.display(),
                        e
                    );
                    return;
                }

                // Set permissions on the file
                if let Err(e) = fs::set_permissions(&extcmdpath, Permissions::from_mode(0o777)) {
                    println!(
                        "[ext] failed to set permissions on '{}': {}",
                        extcmdpath.display(),
                        e
                    );
                    return;
                }

                println!("[ext] exported command '{}.!'", entry_name);
            }
            Err(e) => {
                println!(
                    "[ext] failed to create file '{}': {}",
                    extcmdpath.display(),
                    e
                );
            }
        }
    } else {
        println!("[ext] map entry with name '{}' doesn't exist!", entry_name);
    }
}

fn remove(args: &mut core::iter::Skip<crate::env::Args>, entries: &mut Vec<MapEntry>) {
    let entry_name = match args.next() {
        Some(arg1) => arg1,
        None => {
            println!("[rm] no name supplied, exiting.");
            return;
        }
    };

    // Check if the entry exists in the map
    if !check_entry_exists(&entry_name, &entries) {
        println!("[rm] map entry with name '{}' doesn't exist!", entry_name);
        return;
    }

    // Find the filehash of the entry
    let file_hash = entries
        .iter()
        .find(|entry| entry.name == entry_name)
        .map(|entry| entry.filehash.clone())
        .unwrap_or_else(|| {
            println!("[rm] Unable to find file hash for entry '{}'", entry_name);
            return "8723478546982389235".to_string(); // Default fallback value
        });

    // Remove the corresponding command file if it exists
    let pxcpath = format!("{}/cmd/{}", get_pxc_path(), file_hash);
    if Path::new(&pxcpath).exists() {
        if let Err(e) = fs::remove_file(&pxcpath) {
            println!("[rm] Failed to remove command file '{}': {}", pxcpath, e);
        }
    }

    // Remove the entry from the map
    if let Some(pos) = entries.iter().position(|x| x.name == entry_name) {
        entries.remove(pos);
    } else {
        println!("[rm] Entry '{}' not found in map", entry_name);
    }

    // Remove the external command file if it exists
    let extpath = format!("{}/{}.pxc", get_ext_path(), entry_name);
    if Path::new(&extpath).exists() {
        if let Err(e) = fs::remove_file(&extpath) {
            println!("[rm] Failed to remove external file '{}': {}", extpath, e);
        }
    }

    // Save the updated map to file
    save_map(entries);

    println!("[rm] Removed '{}' successfully!", entry_name);
}

// Return true if the entry_name exists in the map file
fn check_entry_exists(entry_name: &str, entries: &[MapEntry]) -> bool {
    entries.iter().any(|entry| entry.name == entry_name)
}

fn add(mut new_entry: MapEntry, entries: &mut Vec<MapEntry>) {
    if check_entry_exists(&new_entry.name, entries) {
        println!("[add] map entry with this name already exists, this should not happen!");
        return;
    }

    if new_entry.category.is_empty() {
        new_entry.category = "default".to_string();
    }

    let cmd_path = Path::new(&get_pxc_path())
        .join("cmd")
        .join(&new_entry.filehash);

    match File::create(&cmd_path) {
        Ok(_) => {
            if let Err(e) = fs::set_permissions(&cmd_path, fs::Permissions::from_mode(0o777)) {
                println!(
                    "[add] Error setting permissions for file '{}': {}",
                    cmd_path.display(),
                    e
                );
            }
        }
        Err(e) => {
            println!("[add] Error creating file '{}': {}", cmd_path.display(), e);
            return;
        }
    }

    entries.push(new_entry);

    save_map(entries);
}

fn get_pxc_path() -> String {
    match home::home_dir() {
        Some(path) => {
            let pxc_path = path.join(".pxc");
            pxc_path
                .to_str()
                .unwrap_or_else(|| {
                    println!("Error converting path to string");
                    ""
                })
                .to_string()
        }
        None => {
            println!("Unable to get home directory!");
            "".to_string()
        }
    }
}

fn get_ext_path() -> String {
    // path for externalized commands
    return "/usr/local/bin/".to_string();
}

fn save_map(entries: &Vec<MapEntry>) {
    if !cfg!(unix) {
        return;
    }

    let newfilepath = Path::new(&get_pxc_path()).join("map").join("pxc");

    if !newfilepath.exists() {
        println!("[save] map file doesn't exist!");
        return;
    }

    // Open the file in write mode (create or truncate it)
    let file_buffer = File::create(&newfilepath).expect("unable to create file");
    let mut writer = BufWriter::new(file_buffer);

    for entry in entries {
        let entry_line = format!("{};{};{}", entry.name, entry.category, entry.filehash);
        write!(writer, "{}\n", entry_line).expect("unable to write to file");
    }

    println!("[save] file saved!");
}

fn print_cmd(entry_name: &str, entries: Vec<MapEntry>) {
    let position_of_entry = entries.iter().position(|entry| entry.name == entry_name);
    let cmdpath =
        get_pxc_path().to_string() + "/cmd/" + &entries[position_of_entry.unwrap()].filehash;

    if Path::new(&cmdpath).exists() {
        if let Ok(map_lines) = read_lines(&cmdpath) {
            for line in map_lines.flatten() {
                println!("{}", line);
            }
        }
    }
}

fn edit(config: &Config, entry_name: &str, entries: &mut Vec<MapEntry>, category_name: &str) {
    if let Some(entry) = entries.iter_mut().find(|e| e.name == entry_name) {
        if category_name != "no-new-category" {
            entry.category = category_name.to_string();
        }

        println!(
            "[edit] editing command '{}', file: {}",
            entry_name, entry.filehash
        );

        let cmdpath = Path::new(&get_pxc_path()).join("cmd").join(&entry.filehash);

        if let Err(e) = Command::new(&config.editor).arg(cmdpath).status() {
            eprintln!("Failed to execute editor: {}", e);
        }

        save_map(entries);
    } else {
        eprintln!("Entry '{}' not found.", entry_name);
    }
}

fn get_categories(entries: &Vec<MapEntry>) -> Vec<String> {
    let mut unique_categories = HashSet::new();

    for entry in entries {
        unique_categories.insert(entry.category.to_string());
    }

    unique_categories.into_iter().collect()
}

fn list_categories(entries: &Vec<MapEntry>) {
    let categories = get_categories(entries);

    println!("CATEGORIES");
    println!("{}", "🭶".repeat(22));
    for cat in categories {
        println!("{}", cat);
    }
}

fn list(entries: &Vec<MapEntry>, category_name: &str) {
    println!("NAME\t\tCATEGORY\tFILE");
    println!("🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶");

    // If category_name is empty, we need to group entries by category
    let categories_to_process: Vec<String> = if category_name.is_empty() {
        get_categories(entries) // Presumably returns a vector of unique categories
    } else {
        vec![category_name.to_string()]
    };

    // Iterate over the selected categories
    for category in categories_to_process {
        for entry in entries.iter() {
            if entry.category == category {
                println!(
                    "{: <16}{: <16}{: <16}",
                    entry.name, entry.category, entry.filehash
                );
            }
        }
        // Only print a newline between categories if we're listing multiple categories
        if category_name.is_empty() {
            println!();
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<std::io::Lines<BufReader<File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = File::open(filename)?; // Open the file
    let reader = BufReader::new(file); // Create a BufReader from the file
    Ok(reader.lines()) // Return the Lines iterator
}

fn init() -> std::io::Result<()> {
    println!("[init] initializing pxc..");

    if cfg!(windows) {
        // windows todo
        return Ok(());
    } else if cfg!(unix) {
        let pxcpath = get_pxc_path();

        // pxc directory: root pxec directory
        let pxcpath = Path::new(&pxcpath); // Convert String to Path
        fs::create_dir_all(pxcpath).map_err(|e| {
            println!("error when creating dir {}", e);
            e
        })?;

        // map directory: stores the mapping of command to script
        fs::create_dir_all(pxcpath.join("map")).map_err(|e| {
            println!("error when creating map directory: {}", e);
            e
        })?;

        // commands directory: stores all script files
        fs::create_dir_all(pxcpath.join("cmd")).map_err(|e| {
            println!("error when creating cmd directory: {}", e);
            e
        })?;

        let newfilepath = pxcpath.join("map").join("pxc");
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
