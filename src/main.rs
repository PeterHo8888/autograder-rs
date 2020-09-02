#![allow(unused_imports)]
mod autograder;
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
