#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use std::process::{Command, Stdio};
use std::io::{self, Write};

static COURSE_ID: i32 = 88888;
static ASSIGNMENT_ID: i32 = 888888;

// WARNING: DOES NOT WORK WITH BAD INPUTS AND cin LOOPS
static INPUTS: &[&'static str] = &["10", "12"];

mod autograder;
fn main() {
    println!("Hello, world!");
    let mut token = std::fs::read_to_string("token.txt").expect("Error reading token.txt");
    token.pop();
    autograder::init(&token);

    autograder::download_submissions(COURSE_ID, ASSIGNMENT_ID);
    autograder::compile_submissions(COURSE_ID, ASSIGNMENT_ID);

    // Source code
    let source_dir = autograder::get_submission_dir(COURSE_ID, ASSIGNMENT_ID);

    // Executables
    let dir = format!("{}/out", source_dir);
    let paths = autograder::get_submission_files(&dir);

    for path in paths {
        let exe = path.unwrap().path();

        // Student ID + ".exe"
        let elf = exe.file_name().unwrap().to_str().unwrap();
        // Student ID
        let id = elf[..elf.len() - 4].parse::<i32>().unwrap();

        for input in INPUTS.iter() {
            println!("======================\nRunning {} with input '{}'\n", elf, input);
            let mut proc = Command::new(&exe)
                .stdin(Stdio::piped())
                .spawn()
                .expect("Failed to execute process");

            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(&input.as_bytes()).unwrap();
            drop(stdin);

            let output = proc.wait_with_output().unwrap();
            println!("");
        }
        println!("======================");

        // View the code
        // Make sure to `M-x server-start` in an existing emacs window
        // Otherwise, we won't have a single emacs frame
        // to use for everything
        let cmd = format!("{}/{}.cc", source_dir, id);
        Command::new("emacsclient")
            .arg("-n")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .expect("Failed to start emacs");

        loop {
            print!("> ");
            let _ = io::stdout().flush();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).expect("Failed to read input, terminating autograder.");
            choice.pop();

            match choice.as_str() {
                "skip" | "s" => {
                    println!("skipping student");
                    break;
                }
                "help" | "h" => {
                    println!("skip | s: Skip grading this student\n<float %>: percentage to give student");
                }
                _ => {
                    if let Ok(percent) = choice.parse::<f32>() {
                        autograder::grade_submission(COURSE_ID, ASSIGNMENT_ID, id, percent);
                        println!("Done.\n");
                        break;
                    }
                    println!("Received bad input: {}", choice.as_str());
                }
            }
        }
    }
}

fn get_info() {
    let courses = autograder::list_courses();
    for i in 0..courses.len() {
        println!("{:?}", courses[i]);
    }

    let assignments = autograder::list_assignments(courses[1].id);
    for i in 0..assignments.len() {
        if assignments[i].submission_types.iter().any(|v| v == "online_upload")
            && assignments[i].published {
            println!("{:?}", assignments[i]);
        }
    }

    let students = autograder::list_students(courses[1].id);
    for i in 0..students.len() {
        println!("{:?}", students[i]);
    }
}
