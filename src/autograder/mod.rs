use serde::{Deserialize, de};
use serde_json::{Result, Value, json};
use std::io::{stdout, Write, Read, ErrorKind};

mod connection;
use connection::*;

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

pub const API: &str = "https://sit.instructure.com/api/v1";
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
 * Getter for OAUTH token
 */
pub fn get_token() -> &'static str {
    unsafe {
        &OAUTH
    }
}

pub fn get_submission_dir(course_id: i32, assignment_id: i32) -> String {
    format!("submissions/{}/{}/", course_id, assignment_id)
}

pub fn get_submission_files(dir: &str) -> std::fs::ReadDir {
    let paths = std::fs::read_dir(&dir).unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            std::fs::read_dir(&dir).expect("Problem reading submissions dir")
        } else {
            panic!("Problem reading submissions dir: {:?}", error);
        }
    });

    paths
}

/*
 * Compile submissions for a given course and assignment
 */
pub fn compile_submissions(course_id: i32, assignment_id: i32) {
    let dir = get_submission_dir(course_id, assignment_id);
    let paths = get_submission_files(&dir);

    // Remove all previous execs
    // Use .exe extension for Windows compatibility
    let command = format!("rm -rf {}/out/", dir);
    std::process::Command::new("sh").arg("-c").arg(&command).output().unwrap();

    let command = format!("mkdir -p {}/out/", dir);
    std::process::Command::new("sh").arg("-c").arg(&command).output().unwrap();

    for path in paths {
        let filepath = path.unwrap().path(); // FilePath
        let ext = filepath.extension();
        if ext == None || ext.unwrap() != "cc" {
            continue;
        }

        let parent_dir = filepath.parent().unwrap().to_str().unwrap();
        let filename = filepath.file_name().unwrap().to_str().unwrap();
        let command = format!("g++ -Wall -Wextra -fsanitize=address -o {}/out/{}.exe {}", &parent_dir, &filename[..filename.len() - 3], filepath.to_str().unwrap());
        println!("command: {}", command);
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .expect("Failed to execute process");

        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
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

        let json = std::str::from_utf8(&buf).unwrap();
        // Turbofish
        let tmp = serde_json::from_str::<Value>(json).unwrap();
        if tmp["submission_type"].is_null() {
            // No submission
            continue;
        }
        let submission: Submission = serde_json::from_str(json).unwrap();
        println!("{:?}", submission);
        download_url_to_ident(course_id, assignment_id, &submission.user_id, &submission.attachments[0].url);
    }
}

/*
 * Grade a single submission
 */
pub fn grade_submission(course_id: i32, assignment_id: i32, user_id: i32, percent: f32) {
    let path = format!("/courses/{}/assignments/{}/submissions/{}", course_id, assignment_id, user_id);
    let data = json!({
        "submission": {
            "posted_grade": format!("{}%", percent),
        },
    });
    put_json(&path, &data.to_string());
}

/*
 * Get student listing for a given course id
 */
pub fn list_students(course_id: i32) -> Vec<Student> {
    let path = format!("/courses/{}/users?enrollment_type=student&sort=sis_id&per_page=300", course_id);
    let buf = fetch_api(&path);
    raw_to_vec::<Student>(buf)
}

/*
 * Get assignment listing for a given course id
 */
pub fn list_assignments(course_id: i32) -> Vec<Assignment> {
    let path = format!("/courses/{}/assignments?per_page=200", course_id);
    let buf = fetch_api(&path);
    raw_to_vec::<Assignment>(buf)
}

/*
 * Get course listing
 */
pub fn list_courses() -> Vec<Course> {
    let buf = fetch_api("/courses?enrollment_type=ta");
    raw_to_vec::<Course>(buf)
}

/*
 * Convert raw Vec<u8> to Vec<T>, where T is Deserialize
 */
fn raw_to_vec<T: de::DeserializeOwned>(buf: Vec<u8>) -> Vec<T> {
    let json = std::str::from_utf8(&buf).unwrap();
    let data = serde_json::from_str::<Vec<T>>(json).unwrap();
    data
}

/*
 * Download file at URL to "./assignments/{assignment_id}/{user_id}.cc"
 */
fn download_url_to_ident<T: std::fmt::Display>(course_id: i32, assignment_id: i32, ident: &T, url: &String) {
    let buf = fetch_file(url);
    let dir = format!("submissions/{}/{}", course_id, assignment_id);
    let path = format!("submissions/{}/{}/{}.cc", course_id, assignment_id, ident);
    std::fs::create_dir_all(&dir).unwrap();
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(&buf).unwrap();
}
