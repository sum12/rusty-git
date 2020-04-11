struct GitRepo<'a> {
    git_path: &'a str,
    workdir_path: &'a str,
}

fn repo_path<'a>(r: &'a GitRepo, path: Vec<&str>) -> String {
    let mut ret = r.git_path.clone().to_string();

    for p in &path {
        ret = format!("{}/{}", ret, p)
    }
    ret
}

fn main() {
    println!(
        "repo_path = {}",
        repo_path(
            &GitRepo {
                git_path: ".git",
                workdir_path: "."
            },
            vec!["refs"]
        )
    )
}
