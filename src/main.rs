use flate2::bufread::{ZlibDecoder, ZlibEncoder};
use ini::Ini;
use sha1;
use sha1::Digest;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::{Read, Write};
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

fn repo_find(path: Option<&str>, required: Option<bool>) -> Option<GitRepo> {
    let required = required.unwrap_or(false);
    let path = PathBuf::from(path.unwrap_or("."));

    if path.join(".git").exists() {
        return Some(GitRepo::new(path.to_str().unwrap(), false).unwrap());
    }

    if let Some(parent) = path.parent() {
        repo_find(parent.to_str(), Some(required))
    } else if required {
        panic!(".git dir not found !")
    } else {
        None
    }
}

struct GitCommit<'a> {
    repo: &'a GitRepo,
    okv: OrderedHM,
}
struct GitTree(String);
struct GitTag(String);
struct GitBlob<'a>(String, &'a GitRepo);

impl<'a> GitBlob<'a> {
    fn new(r: &'a GitRepo, data: &str) -> Result<GitBlob<'a>, String> {
        let s = data.to_string();
        return Ok(GitBlob(s, r));
    }
}

trait ReadWrite {
    fn serialize(&self) -> Result<String, String>;
    fn deserialize(&mut self, data: String);
    fn fmt(&self) -> &str;
    fn repo(&self) -> &GitRepo;
}

impl<'a> ReadWrite for GitBlob<'a> {
    fn repo(&self) -> &GitRepo {
        self.1
    }
    fn fmt(&self) -> &str {
        "blob"
    }
    fn serialize(&self) -> Result<String, String> {
        Ok((&self.0[..]).to_string())
    }
    fn deserialize(&mut self, data: String) {
        self.0 = data
    }
}

fn decode_bufreader(bytes: Vec<u8>) -> io::Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}

fn object_read<'a>(r: &GitRepo, sha: &str) -> (String, String) {
    let path = repo_file(r, &["objects", &sha[0..2], &sha[2..]], false)
        .expect("object path doesnt Exists");

    let raw: String = decode_bufreader(fs::read(path).unwrap()).unwrap();

    let x = raw
        .find(" ")
        .expect("Cannot read format from object header");
    let (fmt, raw) = raw.split_at(x);

    let y = raw
        .find("\x00")
        .expect("Cannot read size from object header");

    let (size, raw) = raw.split_at(y);

    let size = &size[1..]; //drop the space
    let raw = &raw[1..]; //drop the null

    if let Ok(size) = size.parse::<usize>() {
        if size != (raw.len()) {
            panic!(
                "Malformed Object, \nstored size:{}\nactual size:{}",
                size,
                raw.len()
            );
        } else {
            //println!("size is {} bytes", size);
        }
    } else {
        panic!("invalid object file: size")
    }

    match fmt {
        "blob" => (fmt.to_string(), raw.to_string()),
        "commit" => (fmt.to_string(), raw.to_string()),
        &_ => panic!("cannot parse object"),
    }
}

fn object_find<'a>(
    r: &GitRepo,
    name: &'a str,
    objtype: Option<&str>,
    follow: Option<bool>,
) -> &'a str {
    name
}

impl<'a> GitCommit<'a> {
    fn new(r: &'a GitRepo, data: &str) -> Result<GitCommit<'a>, String> {
        let s = data.to_string();
        let mut gc = GitCommit {
            repo: r,
            okv: OrderedHM::new(),
        };
        gc.deserialize(s);
        Ok(gc)
    }
}

struct OrderedHM {
    order: Vec<String>,
    kv: HashMap<String, Vec<String>>,
}

impl OrderedHM {
    fn new() -> OrderedHM {
        OrderedHM {
            order: Vec::new(),
            kv: HashMap::new(),
        }
    }
}

fn kvlm_parse(raw: String, okv: Option<OrderedHM>) -> OrderedHM {
    let mut okv = okv.unwrap_or(OrderedHM::new());

    let spc = raw.find(' ');
    let nl = raw.find('\n');

    //println!("raw ===> \n{}", raw);
    //if (spc < 0) || (nl < spc) {
    if spc.is_none() || (nl < spc) {
        //println!("{:?}", nl);
        //println!("{:?}", spc);
        assert!(nl.expect("Malformed Object: no new line at end") == 0);
        okv.order.push("".to_string());
        okv.kv.insert("".to_string(), vec![raw]);
        return okv;
    }

    //println!("{}", spc);
    //println!("{}", raw);
    let spc = spc.unwrap();

    let (key, raw) = raw.split_at(spc);
    let raw = raw[1..].to_string();
    let mut end = 0 as usize;

    //println!("{}", key);
    for (count, b) in raw.chars().enumerate() {
        if end != 0 && end == count - 1 && b != ' ' {
            break;
        }
        if b == '\n' {
            end = count
        }
    }

    //println!("{}", end);
    //println!("---------{}", raw.as_bytes()[end - 1] == ' ' as u8);
    //println!("---------{}", &raw[..end]);

    let (value, raw) = raw.split_at(end);
    let value = value.replace("\n ", "\n");
    let raw = raw[1..].to_string();
    //println!("---------{}..", value);

    if okv.kv.contains_key(key) {
        if let Some(val) = okv.kv.get_mut(key) {
            val.push(value);
        }
    } else {
        //println!("..{}..", key);
        //println!("..{}..", value);
        okv.kv.insert(key.to_string(), vec![value]);
        okv.order.push(key.to_string());
    }
    //println!("---------{}", raw.as_bytes()[end] == '9' as u8);

    kvlm_parse(raw, Some(okv))
}

fn kvlm_serialize(okv: &OrderedHM) -> String {
    let mut ret = String::new();
    //println!("{:?}", okv.order);
    for key in okv.order.iter() {
        //println!("{}", key);
        if key == "" {
            continue;
        };
        let val = okv.kv.get(key).expect("key not found !");
        for v in val.iter() {
            //println!("{}", v);
            ret += format!("{} {}\n", key, v.replace("\n", "\n ")).as_str();
        }
    }
    //ret.push('\n');
    //println!("{}", &okv.kv.get("").unwrap()[0]);
    ret += &okv.kv.get("").unwrap_or(&vec!["".to_string()])[0];
    //.replace("\n", "\n ")
    //.as_str();
    ret
}

impl<'a> ReadWrite for GitCommit<'a> {
    fn repo(&self) -> &GitRepo {
        self.repo
    }
    fn fmt(&self) -> &str {
        "commit"
    }
    fn serialize(&self) -> Result<String, String> {
        Ok(kvlm_serialize(&self.okv))
    }
    fn deserialize(&mut self, data: String) {
        self.okv = kvlm_parse(data, None);
    }
}

fn cat_file(r: &GitRepo, objname: &str, objtype: Option<&str>) {
    let (fmt, contents) = object_read(r, object_find(r, objname, objtype, None));
    match fmt.as_str() {
        "blob" => {
            let obj = GitBlob::new(r, contents.as_str()).unwrap().serialize();
            io::stdout().write(obj.expect("Could not serialize").as_bytes());
        }
        "commit" => {
            let obj = GitCommit::new(r, contents.as_str()).unwrap().serialize();
            io::stdout().write(obj.expect("Could not serialize").as_bytes());
        }
        _ => panic!("unknown format"),
    }
}

fn cat_file_cmd(objname: &str, objtype: Option<&str>) {
    let r = repo_find(None, None).expect("Repo not found");
    cat_file(&r, objname, objtype);
}

fn object_write(obj: impl ReadWrite, actually_write: bool) -> Result<String, String> {
    let data = obj.serialize()?;
    //println!("{}", data.len());
    //println!("{}", data);
    let data = format!("{} {}\x00{}", obj.fmt(), data.len(), data);

    //println!("{:?}\n\n", data.as_bytes());

    let mut hasher = sha1::Sha1::new();
    hasher.input(data.as_bytes());

    let mut sha = String::new();
    for code in hasher.result() {
        {
            use std::fmt::Write;
            let _ = write!(&mut sha, "{:x}", code);
        }
    }

    if actually_write {
        let r = obj.repo();
        if let Some(path) = repo_file(&r, &["object", &sha[0..2], &sha[2..]], true) {
            let result = io::BufReader::new(data.as_bytes());
            let mut encoder = ZlibEncoder::new(result, flate2::Compression::fast());
            let mut sha = String::new();
            let _ = encoder.read_to_string(&mut sha);
            fs::write(path, sha).expect("write failed !");
        }
    }
    Ok(sha.to_string())
}
fn object_hash(data: String, fmt: &str, write: bool) -> String {
    let r = repo_find(None, None).expect("Repo not found");
    match fmt {
        "blob" => {
            let obj = GitBlob::new(&r, data.as_str()).expect("invalid blob");
            object_write(obj, write).expect("blob hash failed !!")
        }
        "commit" => {
            let obj = GitCommit::new(&r, data.as_str()).expect("invalid commit !!");
            object_write(obj, write).expect("commit hash failed !!")
        }
        &_ => panic!("cannot parse object"),
    }
}

fn hash_object_cmd(hashobject: &clap::ArgMatches) {
    let write = hashobject.is_present("write");

    let fmt = hashobject.value_of("type").unwrap();
    let path = hashobject.value_of("path").unwrap();
    let data = String::from_utf8(fs::read(path).unwrap()).unwrap();
    let sha = object_hash(data, fmt, write);
    let _ = io::stdout().write(sha.as_bytes());
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
        .subcommand(
            clap::SubCommand::with_name("cat-file")
                .about("Provide the content of a repo object")
                .arg(
                    clap::Arg::with_name("type")
                        .help("Specify type of object")
                        .possible_values(&["blob", "commit"])
                        .required(true),
                )
                .arg(
                    clap::Arg::with_name("name")
                        .help("The object to display")
                        .required(true),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("hash-object")
                .about("Compute object ID and optionally create a blob from a file")
                .arg(
                    clap::Arg::with_name("type")
                        .help("Specify type of object")
                        .short("t")
                        .possible_values(&["blob", "commit"])
                        .default_value("blob"),
                )
                .arg(
                    clap::Arg::with_name("write")
                        .help("Actually write the object into the database")
                        .short("w")
                        .long("write")
                        .takes_value(false),
                )
                .arg(
                    clap::Arg::with_name("path")
                        .help("Read object from file")
                        .takes_value(true)
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
        ("cat-file", Some(catfile)) => cat_file_cmd(
            catfile.value_of("name").unwrap(),
            Some(catfile.value_of("type").unwrap()),
        ),
        ("hash-object", Some(hashobject)) => hash_object_cmd(hashobject),
        (&_, _) => {}
    }
}
