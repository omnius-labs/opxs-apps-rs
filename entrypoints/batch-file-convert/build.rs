use std::env;

fn main() {
    if let Ok(git_tag) = env::var("GIT_TAG") {
        println!("cargo:rustc-env=GIT_TAG={git_tag}");
    }
}
