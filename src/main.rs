#![allow(unused_imports)]
fn main() {
    println!("Hello, world!");
    let token = std::fs::read_to_string("token.txt").expect("Error reading token.txt");
    autograder::init(&token);

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
    use curl::easy::{Easy, List};
    use serde::{Deserialize, de};
    use serde_json::{Result, Value};
    use std::io::{stdout, Write};

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

    const API: &str = "https://sit.instructure.com/api/v1";
    static mut OAUTH: String = String::new();

    /*
     * Set up token. Not calling first or with invalid token
     * will result in undefined behavior (unauthorized)
     */
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
            // Get submission for student
            let path = format!("/courses/{}/assignments/{}/submissions/{}", course_id, assignment_id, students[i].id);
            let buf = fetch_api(&path);

            if let Ok(json) = std::str::from_utf8(&buf) {
                // Turbofish
                if let Ok(tmp) = serde_json::from_str::<Value>(json) {
                    if tmp["submission_type"].is_null() {
                        // No submission
                        continue;
                    }
                    let submission: Submission = match serde_json::from_str(json) {
                        Ok(v) => v,
                        Err(e) => panic!("{}", e),
                    };
                    println!("{:?}", submission);
                    download_url_to_id(assignment_id, submission.user_id, &submission.attachments[0].url);
                }
            }
        }
    }

    /*
     * Get student listing for a given course id
     */
    pub fn list_students(course_id: i32) -> Vec<Student> {
        let path = format!("/courses/{}/users?enrollment_type=student&sort=sis_id&per_page=1000", course_id);
        let buf = fetch_api(&path);
        return raw_to_vec::<Student>(buf);
    }


    /*
     * Get assignment listing for a given course id
     */
    pub fn list_assignments(course_id: i32) -> Vec<Assignment> {
        let path = format!("/courses/{}/assignments?per_page=1000", course_id);
        let buf = fetch_api(&path);
        return raw_to_vec::<Assignment>(buf);
    }

    /*
     * Get course listing
     */
    pub fn list_courses() -> Vec<Course> {
        let buf = fetch_api("/courses?enrollment_type=teacher");
        return raw_to_vec::<Course>(buf);
    }

    /*
     * Convert raw Vec<u8> to Vec<T>, where T is Deserialize
     */
    fn raw_to_vec<T: de::DeserializeOwned>(buf: Vec<u8>) -> Vec<T> {
        let json = match std::str::from_utf8(&buf) {
            Ok(v)  => v,
            Err(_) => panic!("Invalid UTF-8 sequence."),
        };

        let data = match serde_json::from_str::<Vec<T>>(json) {
            Ok(v) => v,
            Err(_) => panic!("Couldn't parse JSON."),
        };
        data
    }

    /*
     * Download file at URL to "./assignments/{assignment_id}/{user_id}.cc"
     */
    fn download_url_to_id(assignment_id: i32, user_id: i32, url: &String) {
        let buf = fetch_file(url);
        let dir = format!("submissions/{}/", assignment_id);
        let path = format!("submissions/{}/{}.cc", assignment_id, user_id);
        std::fs::create_dir_all(&dir).unwrap();
        let mut file = match std::fs::File::create(path) {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };
        match file.write_all(&buf) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
    }

    /*
     * Retrieve file from URL
     */
    fn fetch_file(url: &str) -> Vec<u8> {
        fetch(url, None)
    }

    /*
     * Helper function to retrieve raw JSON from Canvas LMS REST API
     */
    fn fetch_api(path: &str) -> Vec<u8> {
        let mut list = List::new();
        list.append("Content-Type: application/json").unwrap();
        list.append("Charset: UTF-8").unwrap();
        unsafe {
            list.append(&OAUTH).unwrap();
        }
        // Set up URL
        let url = format!("{}{}", API, path);
        fetch(&url, Some(list))
    }

    /*
     * Retrieve data from given URL and optional headers
     */
    fn fetch(url: &str, list: Option<List>) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut handle = Easy::new();

        handle.url(url).unwrap();
        handle.follow_location(true).unwrap(); // 3xx redirects

        if let Some(header) = list {
            handle.http_headers(header).unwrap();
        }

        {
            // Callback
            let mut transfer = handle.transfer();
            transfer.write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();

            transfer.perform().unwrap();
        }
        buf
    }
}
