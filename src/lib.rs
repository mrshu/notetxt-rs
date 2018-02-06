extern crate regex;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::path::Path;
use regex::Regex;
use std::collections::HashMap;


#[allow(dead_code)]
pub struct Notes<'a> {
    dir: &'a str,
    map: HashMap<&'a Path, &'a Note<'a>>,
    notes: Vec<&'a Note<'a>>
}

impl<'a> Notes<'a> {
    pub fn from_dir(dir: &'a str) -> Notes<'a> {
        let map = HashMap::new();
        let notes = vec![];

        match fs::read_dir(dir) {
            Err(why) => println!("Error reading directory {:?} ({:?})",
                                 dir, why.kind()),
            Ok(paths) => for path in paths {
                println!("> {:?}", path.unwrap().path());
            },
        }

        Notes {
            dir,
            map,
            notes
        }

    }
}

#[allow(dead_code)]
pub struct Note<'a> {
    title: String,
    path: &'a Path,
    tags: Vec<String>
}

impl<'a> Note<'a> {
    pub fn title_from_string(text: String) -> String {
        let re = Regex::new("^([^\n]+)\n-+\n").unwrap();
        let caps = re.captures(text.as_str()).unwrap();
        let title = caps.get(1).map_or("", |m| m.as_str());
        return String::from(title);
    }

    pub fn from_path(path: &'a Path, dir: String) -> Note<'a> {
        let display = path.display();
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display,
                                                       why.description()),
            Ok(file) => file,
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display,
                                                       why.description()),
            Ok(_) => (),
        }

        let title = Self::title_from_string(s);

        if let Some(s) = path.parent().unwrap().to_str() {
            let mut remover = String::from("^");
            remover.push_str(dir.as_str());
            // Remove the last separator optionally
            remover.push_str(r"/?");
            let re = Regex::new(remover.as_str()).unwrap();

            let tag = re.replace(s, "").to_string();
            let tags: Vec<String> = vec![tag];
            Note {
                title,
                path,
                tags
            }
        } else {
            panic!("Tag path is not a valid UTF-8 sequence")
        }
    }
}


#[cfg(test)]
mod tests {
    use Note;
    use std::path::Path;

    #[test]
    fn test_title_from_string() {
        let s = String::from("Test\n----\n");
        let n = Note::title_from_string(s);
        assert_eq!(n, "Test");
    }

    #[test]
    fn test_note_from_path() {
        let p = Path::new("tests/notes/test-note.md");
        let dir = String::from("");
        let n = Note::from_path(p, dir);
        assert_eq!(n.title, "Test note");
        assert_eq!(n.tags[0], "tests/notes");
        assert_eq!(n.path, p);
    }

    #[test]
    fn test_note_from_path_dir_removal() {
        // Test removal of beginning of the path
        let p = Path::new("./tests/notes/test-note.md");
        let dir = String::from("./");
        let n = Note::from_path(p, dir);
        assert_eq!(n.title, "Test note");
        assert_eq!(n.tags[0], "tests/notes");
        assert_eq!(n.path, p);
    }

    #[test]
    fn test_note_from_path_dir_removal_with_default() {
        // Test removal of beginning of the path with optional forward slash
        let p = Path::new("./tests/notes/test-note.md");
        let dir = String::from(".");
        let n = Note::from_path(p, dir);
        assert_eq!(n.title, "Test note");
        assert_eq!(n.tags[0], "tests/notes");
        assert_eq!(n.path, p);
    }

}
