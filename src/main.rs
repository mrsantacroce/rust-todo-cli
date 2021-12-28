fn main() {
    load_list().unwrap();
}

// For JSON
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use std::error::Error;
use std::fs::File;
use std::fs::write;
use std::io::Read;

// For Saving File
extern crate app_dirs;
use app_dirs::*;
use app_dirs::{app_dir, get_app_dir};
use std::path::PathBuf;

const APP_INFO: AppInfo = AppInfo{name: "todo-cli", author: "mrsantacroce"};

// For CLI
use std::env;
use std::process;

// For Date & Time
extern crate chrono;
use chrono::prelude::*;

// For Styling
extern crate ansi_term;
use ansi_term::Colour::{Blue, Purple, Green, Yellow, Red, White};
use ansi_term::Style;

// All valid CLI commands for the App
enum Command {
    Help,
    Get,
    Add(String),
    Done(usize),
    Remove(usize)
}

fn load_list() -> Result<(), Box<Error>> {

    // Inputs
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() < 2 {
        help();
    }

    // Initialize an empty list
    let mut todo_list = TodoList::new();

    // Optionally load an existing file
    let save_directory = get_app_dir(AppDataType::UserData, &APP_INFO, "lists/saved");
    let mut path_to_save_file: PathBuf = save_directory.iter().collect();
    path_to_save_file.push("goals.json");

    let save_file_exists = std::path::Path::new(&path_to_save_file).exists();
    let mut file;
    let mut contents;
    if save_file_exists {
        file = File::open(&path_to_save_file)?;
        // Read the file to a string
        contents = String::new();
        file.read_to_string(&mut contents)?;

        // Deserialize the data structure
        let data: TodoList = serde_json::from_str(&contents)?;

        // Fill the empty list with this data
        for (index, item) in data.list.iter().enumerate() {
            todo_list.add_to_list(data.list[index].name.clone());
            if item.completed == true {
                todo_list.toggle_done(index);
            }
            // TODO load the streak
        }
    }
    else {
        let new_save_directory = app_dir(AppDataType::UserData, &APP_INFO, "lists/saved")
            .expect("Failed to create directory to save lists");
        println!("{}", new_save_directory.display());
    }

    let command = match arguments[1].as_str() {
        "get" => {
            Command::Get
        },
        "help" => {
            Command::Help
        },
        "add" => {
            if arguments.len() < 3 {
                let detail = "Try typing a goal name after the 'add' keyword, for example: 'add meditate'";
                help_detail("Missing Goal Name".to_string(), detail.to_string());
                return Ok(());
            }
            Command::Add(arguments[2].clone())
        },
        "done" => {
            if arguments.len() < 3 {
                let detail = "Try typing a goal ID after the 'done' keyword, for example: 'done 1'";
                help_detail("Missing Goal Name".to_string(), detail.to_string());
                return Ok(());
            }
            if !arguments[2].parse::<f64>().is_ok() {
                let detail = "A goal ID can only be a number";
                help_detail("Invalid Goal ID".to_string(), detail.to_string());
                return Ok(());
            }
            Command::Done(arguments[2].parse().expect("Error converting ID to an integer."))
        },
        "remove" => {
            if arguments.len() < 3 {
                let detail = "Try typing a goal ID after the 'remove' keyword, for example: 'remove 0'";
                help_detail("Missing Goal Name".to_string(), detail.to_string());
                return Ok(());
            }
            if !arguments[2].parse::<f64>().is_ok() {
                let detail = "A goal ID can only be a number";
                help_detail("Invalid Goal ID".to_string(), detail.to_string());
                return Ok(());
            }
            Command::Remove(arguments[2].parse().expect("Error converting ID to an integer."))
        },
        _ => panic!("Invalid command!".to_string())
    };

    match command {
        Command::Get => todo_list.print(),
        Command::Help => help(),
        Command::Add(task) => {
            todo_list.add_to_list(task);
            todo_list.print();
        },
        Command::Done(index) => {
            todo_list.toggle_done(index);
            todo_list.print();
        },
        Command::Remove(index) => {
            todo_list.remove_item(index);
            todo_list.print();
        }
    }

    // Serilaize it to a JSON string
    let data_to_write = serde_json::to_string(&todo_list)?;

    // Write to a file
    write(path_to_save_file, data_to_write).expect("Unable to write file!");

    Ok(())

}

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    name: String,
    completed: bool,
    streak: u32
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoList {
    list: Vec<TodoItem>
}

impl TodoItem {
    fn new(name: String) -> TodoItem {
        return TodoItem {
            name: name,
            completed: false,
            streak: 0,
        };
    }
}

impl TodoList {
    fn new() -> TodoList {
        return TodoList {
            list: Vec::new()
        };
    }

    fn add_to_list(&mut self, name: String) {
        let todo_item = TodoItem::new(name);
        self.list.push(todo_item);
    }

    fn print(&self) {
        let local_time = Local::now();
        println!("---------------------------");
        println!("{} for {}/{}/{}", Blue.bold().paint("Daily Goals"), Purple.bold().paint(local_time.month().to_string()), Purple.bold().paint(local_time.day().to_string()), Purple.bold().paint(local_time.year().to_string()));
        println!("---------------------------");
        for (index, item) in self.list.iter().enumerate() {
            if item.completed == true {
                println!("{} ~ [{}] {} - {}", Green.bold().paint(index.to_string()), Green.bold().paint("X".to_string()), Green.bold().paint(item.name.to_string()), Green.bold().paint(item.streak.to_string()));
            } else {
                println!("{} ~ [{}] {} - {}", Red.bold().paint(index.to_string()), Red.bold().paint(" ".to_string()), White.bold().paint(item.name.to_string()), Yellow.italic().paint(item.streak.to_string()));
            }
        }
        println!("---------------------------");
    }

    fn toggle_done(&mut self, index: usize) {
        if self.list.len() > index {
            if self.list[index].completed == true {
                self.list[index].completed = false;
                self.list[index].streak -= 1;
            } else {
                self.list[index].completed = true;
                self.list[index].streak += 1;
            }
        }
    }

    fn remove_item(&mut self, index: usize) {
        if index < self.list.len() {
            self.list.remove(index);
        }
    }
}

/* Utils */
fn help() {
    println!("----------");
    println!("Help");
    println!("----------");
    println!("List of valid commands:");
    help_get();
    help_add();
    help_done();
    help_remove();
    println!("----------");
    process::exit(0);
}

fn help_detail(command: String, detail: String) {
    println!("{} {} - {}", Red.bold().paint("Error"), White.bold().paint(command), Yellow.italic().paint(detail));
}

fn help_get() {
    println!("get");
}

fn help_add() {
    println!("add");
}

fn help_done() {
    println!("done");
}

fn help_remove() {
    println!("remove");
}
