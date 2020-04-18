use ini::Ini;
use std::fs;
use std::path::PathBuf;
struct GitRepo {
    git_path: String,
    workdir_path: String,
    conf: Ini,
}

impl GitRepo {
    fn new(path: &str, force: bool) -> Result<GitRepo, &str> {
        let mut gitdir = PathBuf::from(path);
        gitdir.push(".git");
        let wkdir = path;

        if !(force || (gitdir.exists() && gitdir.is_dir())) {
            return Err(".git dir is missing");
        }

        let gitdir = gitdir.as_path().to_str();
        if !force && gitdir.is_none() {
            return Err("Gitdir not resolable");
        }
        let gitdir = gitdir.unwrap_or("");

        let mut gitrepo = GitRepo {
            git_path: gitdir.to_string(),
            workdir_path: wkdir.to_string(),
            conf: Ini::new(),
        };

        let cnf_pth = repo_file(&gitrepo, &["config"], false);
        if !force && cnf_pth.is_none() {
            return Err("Gitdir not resolable");
        }

        if let Some(cnf_pth) = cnf_pth {
            if cnf_pth.is_file() && cnf_pth.exists() {
                gitrepo.conf = Ini::load_from_file(cnf_pth.as_path()).unwrap();
            } else if !force {
                return Err("config file missing !");
            }
        } else if !force {
            return Err("config file missing !");
        }

        if !force {
            let repover = gitrepo
                .conf
                .get_from(Some("core"), "repositoryformatversion");

            if let Some(repover) = repover {
                let repover = repover.trim().parse();
                if let Ok(0) = repover {
                    return Ok(gitrepo);
                } else {
                    println!("repositoryformatversion {:?}", repover);
                    return Err("repositoryformatversion not supported");
                }
            } else {
                return Err("repositoryformatversion not available");
            }
        }

        return Ok(gitrepo);
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
    let r = GitRepo::new("sumit", false).expect("New Repo not possible");

    let v = vec!["refs", "remotes", "origins", "HEAD"];
    let s = repo_file(&r, &v, true);

    if let Some(p) = s {
        println!("this is = {:?}", p);
    }
}
