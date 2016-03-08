use std::collections::HashMap;
use std::fmt;
use ini::Ini;

use error::Error;

const DEFAULT_BRANCH: &'static str = "master";
const DEFAULT_GIT_PATH: &'static str = "/usr/bin/git";

pub type Projects = HashMap<String, Project>;

#[derive(Clone)]
pub struct Conf {
    pub location: String,
    pub gitpath: String,
    pub projects: Projects,
}

impl Conf {
    pub fn from_ini(ini: &Ini) -> Result<Conf, &str> {
        let default_gitpath = DEFAULT_GIT_PATH.to_owned();
        let s = try!(ini.section(None::<String>).ok_or("No general section found"));
        let location = try!(s.get("location").ok_or("No location found"));
        let gitpath = s.get("gitpath").unwrap_or(&default_gitpath);
        let projects = try!(ini_to_projects(ini));
        Ok(Conf {
            location: location.to_owned(),
            gitpath: gitpath.to_owned(),
            projects: projects,
        })
    }

    pub fn from_file(path: &str) -> Result<Conf, Error> {
        Ini::load_from_file(path)
            .map_err(Error::from)
            .and_then(|ini| Conf::from_ini(&ini).map_err(Error::from))
    }

    pub fn get_project(&self, repo: &str) -> Option<&Project> {
        self.projects.get(repo)
    }
}

fn ini_to_projects(ini: &Ini) -> Result<Projects, &str> {
    ini.iter()
       .filter_map(|pair| {
           let (key, vs) = pair;
           key.as_ref().map(|k| (k, vs))
       })
       .map(|pair| {
           let (key, vs) = pair;
           Project::from_map(key, vs)
       })
       .collect::<Result<Vec<_>, &str>>()
       .map(|projects| {
           projects.into_iter()
                   .map(|p| (p.repo.to_owned(), p))
                   .collect::<HashMap<_, _>>()
       })
}

impl fmt::Display for Conf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = write!(f,
                             "Conf(location = {}, gitpath = {}, projects = [",
                             self.location,
                             self.gitpath);
        for (_, v) in self.projects.iter() {
            if res.is_ok() {
                res = write!(f, "{}, ", v);
            } else {
                break;
            }
        }
        res.and_then(|_| write!(f, "])"))
    }
}

#[derive(Clone)]
pub struct Project {
    pub id: String,
    pub repo: String,
    pub branch: String,
    pub command: String,
    pub key: String,
}

impl Project {
    fn from_map(id: &str, props: &HashMap<String, String>) -> Result<Project, &'static str> {
        let default_branch = DEFAULT_BRANCH.to_owned();
        let repo = try!(props.get("repo").ok_or("No repo found"));
        let branch = props.get("branch").unwrap_or(&default_branch);
        let command = try!(props.get("command").ok_or("No command found"));
        let key = try!(props.get("key").ok_or("No key found"));
        Ok(Project {
            id: id.to_owned(),
            repo: repo.to_owned(),
            branch: branch.to_owned(),
            command: command.to_owned(),
            key: key.to_owned(),
        })
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Project(id = {}, repo = {}, branch = {}, command = {})",
               self.id,
               self.repo,
               self.branch,
               self.command)
    }
}
