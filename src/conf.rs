use std::collections::HashMap;
use std::fmt;
use ini::Ini;

use error::Error;

pub struct Conf {
    location: String,
    gitpath: String,
    projects: HashMap<String, Project>,
}

impl Conf {
    pub fn from_ini(ini: &Ini) -> Result<Conf, &str> {
        let s = try!(ini.section(None::<String>).ok_or("No general section found"));
        let location = try!(s.get("location").ok_or("No location found"));
        let gitpath = try!(s.get("gitpath").ok_or("No gitpath found"));
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
}

fn ini_to_projects(ini: &Ini) -> Result<HashMap<String, Project>, &str> {
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

pub struct Project {
    id: String,
    repo: String,
    branch: String,
    command: String,
}

impl Project {
    fn from_map(id: &str, props: &HashMap<String, String>) -> Result<Project, &'static str> {
        let repo = try!(props.get("repo").ok_or("No repo found"));
        let branch = try!(props.get("branch").ok_or("No branch found"));
        let command = try!(props.get("command").ok_or("No command found"));
        Ok(Project {
            id: id.to_owned(),
            repo: repo.to_owned(),
            branch: branch.to_owned(),
            command: command.to_owned(),
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
