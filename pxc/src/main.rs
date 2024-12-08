extern crate ncurses;
extern crate rand;
use ncurses::*;
use rand::Rng;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::Permissions;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::ops::Deref;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
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

static WINDOW_HEIGHT: i32 = 3;
static WINDOW_WIDTH: i32 = 10;

fn create_win(start_y: i32, start_x: i32) -> WINDOW {
    let win = newwin(WINDOW_HEIGHT, WINDOW_WIDTH, start_y, start_x);
    box_(win, 0, 0);
    wrefresh(win);
    win
}

fn destroy_win(win: WINDOW) {
    let ch = ' ' as chtype;
    wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
    wrefresh(win);
    delwin(win);
}
fn main() {
    let mut entries: Vec<MapEntry> = read_map_file();
    let config = read_config();
    let mut args = env::args().skip(1);

    if let Some(arg) = args.next() {
        match &arg[..] {
            "h" => help(),
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

                edit(&config, &entry_name, entries, &entry_category);
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

                edit(&config, &entry_name, entries, &entry_category);
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
                        27 => { // escape
                            in_loop = false;
                        }
                        263 => { // backspace
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
                                    mvprintw(print_y, 0, &("-> ".to_owned() + &entry.as_str())).unwrap();
                                } else {
                                    mvprintw(print_y, 0, &entry.as_str()).unwrap();
                                }
                            }
                            last_max_y = print_y;

                        }
                        10 => { //enter
                            //endwin();
                            //in_loop = false;
                            // Run command if exact match


                            let mut found_entries = find_entries_containing(&entries, search_word.clone());
                            found_entries.sort_by(|a, b| a.len().cmp(&b.len()));


                            run_cmd(
                                found_entries
                                    .get(0)
                                    .unwrap(),
                                &mut args,
                                &entries,
                            );

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

                            let mut found_entries = find_entries_containing(&entries, search_word.clone());
                            found_entries.sort_by(|a, b| a.len().cmp(&b.len()));

                            for entry in found_entries {
                                print_y += 1;
                                mvprintw(print_y, 0, "                                                                  ").unwrap();
                                if print_y == 1 {
                                    mvprintw(print_y, 0, &("-> ".to_owned() + &entry.as_str())).unwrap();
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
            println!("running command '{}', filehash: {}", arg, ent.filehash);
            let cmdpath = format!("{}/cmd/{}", get_pxc_path(), ent.filehash);

            // Collect all arguments in a vector, and join them into a single string
            let cmdargs = args.collect::<Vec<_>>().join(" ");

            println!("cargs: {}", cmdargs);
            Command::new("sh")
                .arg("-c")
                .arg(format!("{} {}", cmdpath, cmdargs))
                .status()
                .expect("failed to execute process");
        }
        None => println!("command '{}' not found", arg),
    };
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
        if let Err(e) = File::create(&config_filepath).and_then(|mut file| {
            write!(file, "editor;{}\n", config.editor)
        }) {
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
    if !check_entry_exists(&entry_name, &entries) {
        println!("[ext] map entry with name '{}' doesn't exist!", entry_name);
        return;
    }

    if !cfg!(unix) {
        return;
    }

    for entry in entries {
        if entry.name == entry_name {
            let extcmdpath = get_ext_path() + entry_name + ".!";
            let cmdfilepath = get_pxc_path() + "/cmd/" + &entry.filehash;

            let file_buffer = File::create(&extcmdpath).expect("unable to create file");
            let mut file_buffer = BufWriter::new(file_buffer);

            write!(
                file_buffer,
                "{}\n",
                "exec ".to_owned() + &cmdfilepath + " \"$@\""
            )
            .expect("unable to write");
            std::fs::set_permissions(&extcmdpath, Permissions::from_mode(0o777));
        }
    }
    println!("[ext] exported command '{}.!'", entry_name);
}

// Removes a pxc command from the map file
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
    let pxcpath = get_pxc_path() + "/cmd/" + &file_hash;
    if Path::new(&pxcpath).exists() {
        fs::remove_file(&pxcpath);
    }

    entries.remove(
        entries
            .iter()
            .position(|x| *&x.name == entry_name.to_string())
            .expect("not found"),
    );

    // remove external command if exists.
    let extpath = get_ext_path() + &entry_name + ".pxc";
    if Path::new(&extpath).exists() {
        fs::remove_file(&extpath);
    }

    save_map(&entries);

    println!("[rm] removed {}!", entry_name);
}

// Return <true> if the entry_name exists in the map file
fn check_entry_exists(entry_name: &str, entries: &Vec<MapEntry>) -> bool {
    for entry in entries {
        if entry.name == entry_name {
            return true;
        }
    }
    return false;
}

fn add(mut new_entry: MapEntry, entries: &mut Vec<MapEntry>) {
    if check_entry_exists(&new_entry.name, entries) {
        println!("[add] map entry with this name already exists, this should not happen!");
        return;
    }
    if new_entry.category == "" {
        new_entry.category = "default".to_string();
    }

    std::fs::File::create(get_pxc_path() + "/cmd/" + &new_entry.filehash);
    std::fs::set_permissions(
        get_pxc_path() + "/cmd/" + &new_entry.filehash,
        Permissions::from_mode(0o777),
    );

    entries.push(new_entry);
    save_map(entries);
}

fn get_pxc_path() -> String {
    match home::home_dir() {
        Some(path) if !path.as_os_str().is_empty() => {
            return path.as_os_str().to_str().unwrap().to_string() + "/.pxc"
        }
        _ => {
            println!("Unable to get pxc path!");
            return "".to_string();
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

    let newfilepath = get_pxc_path() + "/map/pxc";

    if !Path::new(&newfilepath).exists() {
        println!("[save] map file doesn't exist!");
        return;
    }

    let file_buffer = File::create(&newfilepath).expect("unable to create file");
    let mut file_buffer = BufWriter::new(file_buffer);

    for entry in entries {
        let entry_line =
            "".to_owned() + &entry.name + ";" + &entry.category + ";" + &entry.filehash;
        write!(file_buffer, "{}\n", entry_line).expect("unable to write");
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

fn edit(config: &Config, entry_name: &str, mut entries: Vec<MapEntry>, category_name: &str) {
    for entry in &mut entries {
        if entry.name == entry_name {
            if category_name != "no-new-category" {
                entry.category = category_name.to_string();
            }

            println!(
                "[edit] editing command '{}', file: {}",
                entry_name, entry.filehash
            );
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
    let mut cat_list: Vec<String> = Vec::new();

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
    let cat_list: Vec<String> = get_categories(entries);

    println!("CATEGORIES");
    println!("🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶");
    for cat in cat_list {
        println!("{}", cat);
    }
}

fn list(entries: &Vec<MapEntry>, category_name: &str) {
    println!("NAME\t\tCATEGORY\tFILE");
    println!("🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶🭶");

    if category_name != "" {
        for entry in entries.iter() {
            if entry.category == category_name {
                println!(
                    "{: <16}{: <16}{: <16}",
                    entry.name, entry.category, entry.filehash
                );
            }
        }
        println!();
    } else {
        for category in get_categories(entries) {
            for entry in entries.iter() {
                if entry.category == category {
                    println!(
                        "{: <16}{: <16}{: <16}",
                        entry.name, entry.category, entry.filehash
                    );
                }
            }
            println!();
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<std::io::Lines<BufReader<File>>> 
where
    P: AsRef<std::path::Path>,
{
    let file = File::open(filename)?;  // Open the file
    let reader = BufReader::new(file);  // Create a BufReader from the file
    Ok(reader.lines())  // Return the Lines iterator
}

fn init() -> std::io::Result<()> {
    println!("[init] initializing pxc..");

    if cfg!(windows) {
        // windows todo
        return Ok(());
    } else if cfg!(unix) {
        let pxcpath = get_pxc_path();

        // pxc directory: root pxec directory
        match fs::create_dir_all(&pxcpath) {
            Ok(()) => {}
            Err(dir) => {
                println!("error when creating dir {}", dir)
            }
        }
        // map directory: stores the mapping of command to script
        match fs::create_dir_all(pxcpath.to_owned() + "/map/") {
            Ok(()) => {}
            Err(dir) => {
                println!("error when creating dir {}", dir)
            }
        }
        // commands directory: stores all script files
        match fs::create_dir_all(pxcpath.to_owned() + "/cmd/") {
            Ok(()) => {}
            Err(dir) => {
                println!("error when creating dir {}", dir)
            }
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
