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

fn create_repo(rp: &str) -> Result<GitRepo, &str> {
    let r = GitRepo::new(rp, true).expect("Repo Creation Error");

    let workdir_path = PathBuf::from(&r.workdir_path[..]);

    if workdir_path.exists() {
        if !workdir_path.is_dir() {
            return Err("workdir is not a dir");
        }
        let dir = fs::read_dir(&workdir_path);
        for _ in dir {
            return Err("workdir is not empty");
        }
    } else {
        fs::create_dir_all(&workdir_path)
            .expect(&format!("create dir failed: {:?}", &workdir_path));
    }

    repo_dir(&r, &["branches"], true).expect("could not create branches folder");
    repo_dir(&r, &["objects"], true).expect("could not create objects folder");
    repo_dir(&r, &["refs", "tags"], true).expect("could not create refs/tags folder");
    repo_dir(&r, &["refs", "heads"], true).expect("could not create refs/heads folder");

    if let Some(s) = repo_file(&r, &["description"], true) {
        fs::write(
            s,
            "Unnamed repository; edit this file 'description' to name the repository.\n",
        )
        .expect("Cannot write description file");
    }

    if let Some(f) = repo_file(&r, &["HEAD"], true) {
        fs::write(f, "ref: refs/heads/master\n").expect("cannot write HEAD file");
    }

    if let Some(f) = repo_file(&r, &["config"], true) {
        repo_default_config()
            .write_to_file(f)
            .expect("failed to write config");
    }
    return Ok(r);
}

fn repo_default_config() -> Ini {
    let mut ret = Ini::new();
    for (k, v) in &[
        ("repositoryformatversion", "0"),
        ("filemode", "false"),
        ("bare", "false"),
    ] {
        ret.set_to(Some("core"), k.to_string(), v.to_string());
    }

    ret
}

fn repo_find(path: &str, required: bool) -> Option<GitRepo> {
    let path = PathBuf::from(path);

    if path.join(".git").exists() {
        return Some(GitRepo::new(path.to_str().unwrap(), false).unwrap());
    }

    if let Some(parent) = path.parent() {
        repo_find(parent.to_str().unwrap(), required)
    } else if required {
        panic!(".git dir not found !")
    } else {
        None
    }
}

fn main() {
    let matches = clap::App::new("Gust")
        .version("0.1")
        .about("Simple rust based git client")
        .subcommand(
            clap::SubCommand::with_name("init")
                .about("Create a git repo")
                .arg(
                    clap::Arg::with_name("path")
                        .help("Required path for repo")
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", Some(init)) => {
            let r = create_repo(init.value_of("path").unwrap()).expect("could not create repo");
            let v = vec!["refs", "remotes", "origins", "HEAD"];
            let s = repo_file(&r, &v, true);

            if let Some(p) = s {
                println!("this is = {:?}", p);
            }
        }
        (&_, _) => {}
    }
}
