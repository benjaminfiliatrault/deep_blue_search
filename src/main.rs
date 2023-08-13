mod read_lines;

use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};

use read_lines::read_lines;

#[derive(Debug)]
struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn slice(&mut self, idx: usize) -> &'a [char] {    
            let token = &self.content[0..idx];
            self.content = &self.content[idx..];
            return token
    }

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();

        if self.content.len() == 0 {
            return None;
        }

        if self.content[0].is_numeric() {
            let mut idx = 0;
            while idx < self.content.len() && self.content[idx].is_numeric() {
                idx += 1;
            }

            return Some(self.slice(idx));
        }

        if self.content[0].is_alphabetic() {
            let mut idx = 0;
            while idx < self.content.len() && self.content[idx].is_alphanumeric() {
                idx += 1;
            }
            return Some(self.slice(idx))
        }

        return Some(self.slice(1))
    }
}
impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn get_file_content(path: &DirEntry) -> io::Result<String> {
    let mut content = String::new();

    if let Ok(lines) = read_lines(path.path()) {
        for line in lines {
            if let Ok(ip) = line {
                content.push_str(&ip);
            }
        }
    }

    Ok(content)
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}


fn main() {
    let dir_path = Path::new("react.dev/src/content");

    let _ = visit_dirs(dir_path, &|path| {
        if let Ok(content) = get_file_content(path) {
            let content = content.chars().collect::<Vec<_>>();

            for chars in Lexer::new(&content) {
                println!("{token:?}", token = chars.iter().map(|c| c.to_ascii_uppercase()).collect::<String>());
            }
        }
    });

    // let content = get_file_content(file_path).expect("TODO");

    // print!("{content}");
}
