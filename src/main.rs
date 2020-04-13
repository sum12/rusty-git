use std::fs;
use std::path::PathBuf;
struct GitRepo<'a> {
    git_path: &'a str,
    //workdir_path: &'a str,
}

fn repo_path(r: &GitRepo, path: &[&str]) -> PathBuf {
    let mut ret = PathBuf::from(r.git_path);
    for p in path {
        ret.push(p);
    }
    ret
}

fn repo_dir(r: &GitRepo, path: &[&str], mkdir: bool) -> Option<PathBuf> {
    let rp = repo_path(r, path);

    if rp.exists() && rp.is_dir() {
        return Some(rp);
    }
    if mkdir {
        if let Some(rd) = rp.to_str() {
            fs::create_dir_all(rd).expect(&format!("create dir failed: {}", rd));
            return Some(rp);
        }
    }
    return None;
}

fn repo_file(r: &GitRepo, path: &[&str], mkdir: bool) -> Option<PathBuf> {
    if let Some((_, dir_path)) = path.split_last() {
        if let Some(_) = repo_dir(r, &dir_path.to_vec(), mkdir) {
            return Some(repo_path(r, path));
        }
    }
    return None;
}

fn main() {
    let r = &GitRepo {
        git_path: ".git",
        //workdir_path: ".",
    };
    let v = vec!["refs", "remotes", "origins", "HEAD"];
    let s = repo_file(r, &v, true);

    if let Some(p) = s {
        println!("this is = {:?}", p);
    }
}
