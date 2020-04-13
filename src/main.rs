use ini::Ini;
use std::fs;
use std::path::PathBuf;
struct GitRepo {
    git_path: String,
    workdir_path: String,
    conf: Ini,
}

impl GitRepo {
    fn new(path: &str) -> Result<GitRepo, &str> {
        let mut gitdir = PathBuf::from(path);
        gitdir.push(".git");
        let gitdir = gitdir.as_path().to_str();
        let wkdir = path;
        if let Some(gd) = gitdir {
            let gitrepo = GitRepo {
                git_path: gd.to_string(),
                workdir_path: wkdir.to_string(),
                conf: Ini::new(),
            };
            return Ok(gitrepo);
        }
        return Err("PathBuf Creation error");
    }
}

fn repo_path(r: &GitRepo, path: &[&str]) -> PathBuf {
    let mut ret = PathBuf::from(&r.git_path);
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
    let r = GitRepo::new("sumit").expect("New Repo not possible");

    let v = vec!["refs", "remotes", "origins", "HEAD"];
    let s = repo_file(&r, &v, true);

    if let Some(p) = s {
        println!("this is = {:?}", p);
    }
}
