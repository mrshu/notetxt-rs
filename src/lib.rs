extern crate regex;
extern crate glob;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use glob::glob;
use std::path::PathBuf;
use regex::Regex;
use std::fmt;


#[allow(dead_code)]
pub struct Notes {
    dir: String,
    notes: Vec<Note>
}

impl Notes {
    pub fn from_dir(dir: String) -> Notes {
        let dir_buf = {
            let buf = PathBuf::from(dir.clone());
            buf.canonicalize().unwrap()
        };

        let files: Vec<PathBuf> = glob(format!("{}/**/*", dir_buf.to_str().unwrap()).as_str())
                                       .expect("Failed to read glob pattern")
                                       .filter_map(Result::ok)
                                       .filter(|p| !p.symlink_metadata().unwrap()
                                                     .file_type().is_symlink())
                                       .collect();

        let symlinks: Vec<PathBuf> = glob(format!("{}/**/*", dir_buf.to_str().unwrap()).as_str())
                                          .expect("Failed to read glob pattern")
                                          .filter_map(Result::ok)
                                          .filter(|p| p.symlink_metadata().unwrap()
                                                       .file_type().is_symlink())
                                          .collect();

        let mut notes: Vec<Note> = files.iter()
                                    .map(|path: &PathBuf| Note::from_path(path, &dir_buf))
                                    .filter_map(Result::ok)
                                    .collect();

        for link in symlinks.iter() {
            for note in notes.iter_mut() {
                let read_link = link.canonicalize().unwrap();
                if note.path.to_str() == read_link.to_str() {
                    let bare_file = link.strip_prefix(dir_buf.to_str().unwrap()).unwrap();
                    let bare_directory = bare_file.parent().unwrap();
                    let tag = String::from(bare_directory.to_str().unwrap());

                    note.tags.push(tag);
                }
            }
        }

        Notes {
            dir,
            notes
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Note {
    title: String,
    path: PathBuf,
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

impl Note {
    pub fn title_from_string(text: String) -> Result<String, NoteErrors> {
        let re = Regex::new("^([A-Za-z0-9 -_:]+)\n-+\n").unwrap();
        let caps = match re.captures(text.as_str()) {
            Some(x) => x,
            None => return Err(NoteErrors::NoteTitleMissingError),
        };

        let title = caps.get(1).map_or("", |m| m.as_str());
        return Ok(String::from(title));
    }

    pub fn from_path(path: &PathBuf, note_dir: &PathBuf) -> Result<Note, Box<Error>> {
        let mut file = File::open(path.as_path())?;//.expect(&format!("Failed to open file {}", display));
        let mut s = String::new();
        file.read_to_string(&mut s)?;//.expect(&format!("Failed to read file {}", display));

        // Only take the first 512 characters into account.
        let mut end_length = 512;
        if s.len() < end_length {
            end_length = s.len();
        }

        s = String::from(&s[..end_length]);

        let title = Self::title_from_string(s)?;

        let dir_pathbuf = {
            let mut new_path = path.clone();
            new_path.pop();
            new_path.canonicalize().unwrap()
        };

        let dir = note_dir.canonicalize().unwrap();

        let tag = String::from(dir_pathbuf.strip_prefix(dir.to_str().unwrap()).unwrap()
                               .to_str().unwrap());
        let tags: Vec<String> = vec![tag];
        let mut path = path.clone();
        path = path.canonicalize().unwrap();
        Ok(
            Note {
                title,
                path,
                tags
            }
        )
    }
}


#[cfg(test)]
mod tests {
    use Note;
    use Notes;
    use std::error::Error;
    use std::path::PathBuf;


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

    fn note_from_path(path: &str, dir: &str) -> Result<Note, Box<Error>> {
        let p = PathBuf::from(path);
        let d = PathBuf::from(dir);
        return Note::from_path(&p, &d);
    }

    #[test]
    fn test_note_from_path() {
        let path = "./tests/notes/test-note.md";
        let n = note_from_path(path, "./").unwrap();
        assert_eq!(n.title, "Test note");
        assert_eq!(n.tags[0], "tests/notes");
        assert_eq!(n.path, PathBuf::from(path).canonicalize().unwrap());
    }

    #[test]
    fn test_very_long_note_from_path() {
        let n = note_from_path("./tests/notes/test-note-very-long.md", "./");
        assert_eq!(n.is_err(), true);
    }

    #[test]
    fn test_note_from_path_pdf_file() {
        let n = note_from_path("./tests/notes/otherfile.pdf", "./");
        assert_eq!(n.is_err(), true);
    }

    #[test]
    fn test_note_from_path_tex_file() {
        let n = note_from_path("./tests/notes/otherfile.tex", "./");
        assert_eq!(n.is_err(), true);
    }

    #[test]
    fn test_note_from_nonexistent_path() {
        let n = note_from_path("./tests/notes/test-note-nonexistent.md", "./");
        assert_eq!(n.is_err(), true);
    }

    #[test]
    fn test_notes_form_dir() {
        let notes = Notes::from_dir(String::from("./tests/"));
        assert_eq!(notes.notes.len(), 1);
        assert_eq!(notes.notes[0].tags.len(), 2);
    }
}
