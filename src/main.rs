#![allow(unused_imports)]
fn main() {
    println!("Hello, world!");
    let token = std::fs::read_to_string("token.txt").expect("Error reading token.txt");
    autograder::init(&token[..]);

    let courses = autograder::list_courses();
    for i in 0..courses.len() {
        println!("{:?}", courses[i]);
    }

    let assignments = autograder::list_assignments(courses[0].id);
    for i in 0..assignments.len() {
        if assignments[i].submission_types.iter().any(|v| v=="online_upload")
            && assignments[i].published {
            println!("{:?}", assignments[i]);
        }
    }

    let students = autograder::list_students(courses[0].id);
    for i in 0..students.len() {
        println!("{:?}", students[i]);
    }

    autograder::download_submissions(133, 199001);
}

mod autograder {
#[derive(Deserialize, Debug)]
pub struct Course {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Assignment {
    pub id: i32,
    pub name: String,
    pub submission_types: Vec<String>,
    pub published: bool,
}

#[derive(Deserialize, Debug)]
pub struct Student {
    pub id: i32,
    pub name: String,
    pub sortable_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Submission {
    pub user_id: i32,
    pub submission_type: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Deserialize, Debug)]
pub struct Attachment {
    pub filename: String,
    pub url: String,
}

    use curl::easy::{Easy, List};
    use serde::{Deserialize};
    use serde_json::{Result, Value};
    use std::io::{stdout, Write};

    const API: &str = "https://sit.instructure.com/api/v1";
    static mut OAUTH: String = String::new();

    pub fn init(token: &str) {
        unsafe {
            OAUTH = format!("Authorization: Bearer {}", token);
        }
    }

    /*
     * Download submissions for a given course and assignment
     */
    pub fn download_submissions(course_id: i32, assignment_id: i32) {
        let students = list_students(course_id);
        for i in 0..students.len() {
            let path = format!("/courses/{}/assignments/{}/submissions/{}", course_id, assignment_id, students[i].id);
            let buf = fetch(&path[..]);
            if let Ok(json) = std::str::from_utf8(&buf) {
                // Turbofish
                if let Ok(tmp) = serde_json::from_str::<Value>(json) {
                    if tmp["submission_type"].is_null() {
                        continue;
                    }
                    let submission: Submission = match serde_json::from_str(json) {
                        Ok(v) => v,
                        Err(e) => panic!("Panic on: {}\n{}", json, e),
                    };
                    println!("{:?}", submission);
                }
            }
        }
    }

    /*
     * Get student listing for a given course for display
     */
    pub fn list_students(course_id: i32) -> Vec<Student> {
        let path = format!("/courses/{}/users?enrollment_type=student&sort=sis_id&per_page=1000", course_id);
        let buf = fetch(&path[..]);
        if let Ok(json) = std::str::from_utf8(&buf) {
            let students: Vec<Student> = match serde_json::from_str(json) {
                Ok(v) => v,
                Err(e) => panic!("{}", e),
            };
            return students;
        }
        panic!("Couldn't find course with id {}", course_id);
    }


    /*
     * Get assignment listing for a given course for display
     */
    pub fn list_assignments(course_id: i32) -> Vec<Assignment> {
        let path = format!("/courses/{}/assignments?per_page=1000", course_id);
        let buf = fetch(&path[..]);
        if let Ok(json) = std::str::from_utf8(&buf) {
            let assignments: Vec<Assignment> = match serde_json::from_str(json) {
                Ok(v) => v,
                Err(e) => panic!("{}", e),
            };
            return assignments;
        }
        panic!("Couldn't find course with id {}", course_id);
    }

    /*
     * Get course listing for display
     */
    pub fn list_courses() -> Vec<Course> {
        let buf = fetch("/courses?enrollment_type=teacher");
        let json = match std::str::from_utf8(&buf) {
            Ok(v)  => v,
            Err(_) => panic!("Invalid UTF-8 sequence."),
        };

        let courses: Vec<Course> = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(_) => panic!("Couldn't parse JSON"),
        };

        courses
    }

    fn fetch(path: &str) -> Vec<u8> {
        let mut handle = Easy::new();
        let mut list = List::new();

        let mut buf = Vec::new();

        // Set up URL
        let url = API.to_owned() + path;
        handle.url(&url[..]).unwrap();

        // Set up headers
        list.append("Content-Type: application/json").unwrap();
        list.append("Charset: UTF-8").unwrap();
        unsafe {
            list.append(&OAUTH[..]).unwrap();
        }
        handle.http_headers(list).unwrap();
        {
            // Callback
            let mut transfer = handle.transfer();
            transfer.write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();

            // Perform
            transfer.perform().unwrap();
        }
        buf
    }
}
