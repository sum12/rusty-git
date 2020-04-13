use std::path::Path;
struct GitRepo<'a> {
    git_path: &'a str,
    workdir_path: &'a str,
}

fn repo_path<'a>(r: &'a GitRepo, path: Vec<&str>) -> &'a Path {
    let p = Path::new(r.git_path);
    let mut ret = p.to_path_buf();
    for p in &path {
        ret.push(Path::new(p));
    }
    p
}

fn main() {
    let r = &GitRepo {
        git_path: ".git",
        workdir_path: ".",
    };

    let s = repo_path(r, vec!["refs"]).to_str();
    //println!("{:?}", s);

    if let Some(p) = s {
        println!("{}", p);
    }
}
