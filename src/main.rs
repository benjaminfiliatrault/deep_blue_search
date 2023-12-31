mod read_lines;

use std::{
    fs::{self, DirEntry, File},
    path::{Path, PathBuf}, collections::HashMap, sync::Mutex, io::{self, stdin},
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

    fn slice_while<P>(&mut self, mut predicate: P) -> &'a [char] where P: FnMut(&char) -> bool {
        let mut idx = 0;
        while idx < self.content.len() && predicate(&self.content[idx]) {
            idx += 1;
        }
        self.slice(idx)
    }

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();

        if self.content.len() == 0 {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self.slice_while(|x| x.is_numeric()))
        }

        if self.content[0].is_alphabetic() {
            return Some(self.slice_while(|x| x.is_alphanumeric()))
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

fn get_file_content(path: &Path) -> io::Result<String> {
    let mut content = String::new();

    if let Ok(lines) = read_lines(path) {
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

type TermFreq = HashMap<String, usize>;
type DocIdx = HashMap<PathBuf, TermFreq>;

fn main() -> io::Result<()> {
    let dir_path = Path::new("react.dev/src/content");
    let document_index_path = "index.json";
    let document_index: DocIdx;

    if Path::new(document_index_path).exists() {
   
        println!("Reading from Index");

        let index_file  = File::open(document_index_path)?;

        document_index = serde_json::from_reader(index_file).unwrap();

    } else {
        let doc_idx = Mutex::new(DocIdx::new());
        
        let _ = visit_dirs(dir_path, &|path| {
            let mut term_frequency = TermFreq::new();
            let file_path = path.path();    


            if let Ok(content) = get_file_content(&file_path) {
                let content = content.chars().collect::<Vec<_>>();

                for chars in Lexer::new(&content) {

                    let term = chars.iter().map(|c| c.to_ascii_uppercase()).collect::<String>();

                    if let Some(frequency) = term_frequency.get_mut(&term) {
                        *frequency += 1;
                    } else {
                        term_frequency.insert(term, 1);
                    }
                }

                let mut stats = term_frequency.iter().collect::<Vec<_>>();

                stats.sort_by_key(|(_, f)| *f);
                stats.reverse();

                doc_idx.lock().unwrap().insert(file_path, term_frequency);
            }
        });

        document_index = doc_idx.lock().unwrap().clone();

        let index_file = File::create(document_index_path)?;

        serde_json::to_writer(index_file, &document_index).expect("Cannot write document_index to disk"); 

    };


    println!("Index file contains {file_count}", file_count = document_index.len());

    
    let mut query_input = String::new();
    println!("Search: ");

    let query = stdin().read_line(&mut query_input).unwrap();

    println!("{query_input}");

    

    Ok(())
}
