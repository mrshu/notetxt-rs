extern crate regex;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::path::Path;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;


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

#[derive(PartialEq, Debug)]
pub enum NoteErrors {
    NoteTitleMissingError,
}

impl fmt::Display for NoteErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NoteErrors::NoteTitleMissingError => write!(f, "The note title could not be parsed.")
        }
    }
}

impl Error for NoteErrors {
    fn description(&self) -> &str {
        match *self {
            NoteErrors::NoteTitleMissingError => "The provided string did not contain something like a note title."
        }
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl<'a> Note<'a> {
    pub fn title_from_string(text: String) -> Result<String, NoteErrors> {
        let re = Regex::new("^([A-Za-z0-9 -_:]+)\n-+\n").unwrap();
        let caps = match re.captures(text.as_str()) {
            Some(x) => x,
            None => return Err(NoteErrors::NoteTitleMissingError),
        };

        let title = caps.get(1).map_or("", |m| m.as_str());
        return Ok(String::from(title));
    }

    pub fn from_path(path: &'a Path, dir: String) -> Result<Note<'a>, Box<Error>> {
        let mut file = File::open(&path)?;//.expect(&format!("Failed to open file {}", display));
        let mut s = String::new();
        file.read_to_string(&mut s)?;//.expect(&format!("Failed to read file {}", display));

        // Only take the first 512 characters into account.
        let mut end_length = 512;
        if s.len() < end_length {
            end_length = s.len();
        }

        s = String::from(&s[..end_length]);

        let title = Self::title_from_string(s)?;

        if let Some(s) = path.parent().unwrap().to_str() {
            let mut remover = String::from("^");
            remover.push_str(dir.as_str());
            // Remove the last separator optionally
            remover.push_str(r"/?");
            let re = Regex::new(remover.as_str()).unwrap();

            let tag = re.replace(s, "").to_string();
            let tags: Vec<String> = vec![tag];
            Ok(
                Note {
                    title,
                    path,
                    tags
                }
            )
        } else {
            panic!("Tag path is not a valid UTF-8 sequence")
        }
    }
}


#[cfg(test)]
mod tests {
    use Note;

    #[allow(dead_code)]
    use NoteErrors;

    use std::path::Path;

    fn parse_title_or_return_placeholder(text: &str, placeholder: &str) -> String {
        let s = String::from(text);
        match Note::title_from_string(s) {
            Ok(string) => string,
            Err(_) => String::from(placeholder),
        }
    }

    #[test]
    fn test_title_from_string() {
        let n = parse_title_or_return_placeholder("Test\n----\n", "err");
        assert_eq!(n, "Test");
    }

    #[test]
    fn test_title_from_string_invalid() {
        let n = parse_title_or_return_placeholder("\n", "err");
        assert_eq!(n, "err");
    }

    #[test]
    fn test_note_from_path() {
        let p = Path::new("tests/notes/test-note.md");
        let dir = String::from("");
        let n = Note::from_path(p, dir).unwrap();
        assert_eq!(n.title, "Test note");
        assert_eq!(n.tags[0], "tests/notes");
        assert_eq!(n.path, p);
    }

    #[test]
    fn test_note_from_path_dir_removal() {
        // Test removal of beginning of the path
        let p = Path::new("./tests/notes/test-note.md");
        let dir = String::from("./");
        let n = Note::from_path(p, dir).unwrap();
        assert_eq!(n.title, "Test note");
        assert_eq!(n.tags[0], "tests/notes");
        assert_eq!(n.path, p);
    }

    #[test]
    fn test_note_from_path_dir_removal_with_default() {
        // Test removal of beginning of the path with optional forward slash
        let p = Path::new("./tests/notes/test-note.md");
        let dir = String::from(".");
        let n = Note::from_path(p, dir).unwrap();
        assert_eq!(n.title, "Test note");
        assert_eq!(n.tags[0], "tests/notes");
        assert_eq!(n.path, p);
    }

    #[test]
    fn test_very_long_note_from_path() {
        let p = Path::new("./tests/notes/test-note-very-long.md");
        let dir = String::from("./");
        let n = Note::from_path(p, dir);
        assert_eq!(n.is_err(), true);
        //assert_eq!(n.err(), Some(NoteErrors::NoteTitleMissingError));
    }

    #[test]
    fn test_note_from_path_pdf_file() {
        let p = Path::new("./tests/notes/otherfile.pdf");
        let dir = String::from("./");
        let n = Note::from_path(p, dir);
        assert_eq!(n.is_err(), true);
        //panic!("{:?}", n.err());
        //assert_eq!(n.err(), Some(NoteErrors::NoteTitleMissingError));
    }

    #[test]
    fn test_note_from_path_tex_file() {
        let p = Path::new("./tests/notes/otherfile.tex");
        let dir = String::from("./");
        let n = Note::from_path(p, dir);
        assert_eq!(n.is_err(), true);
        //panic!("{:?}", n.err());
        //assert_eq!(n.err(), Some(NoteErrors::NoteTitleMissingError));
    }

    #[test]
    fn test_note_from_nonexistent_path() {
        let p = Path::new("./tests/notes/test-note-nonexistent.md");
        let dir = String::from("./");
        let n = Note::from_path(p, dir);
        assert_eq!(n.is_err(), true);
        panic!("{:?}", n.err());
        //assert_eq!(n.err(), Some(NoteErrors::NoteTitleMissingError));
    }
}
