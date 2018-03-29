use FileDescriptor;

use nom;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
pub enum ParserWithDependenciesError {
    Io(io::Error),
    Other(String),
    Nom(nom::IError),
}

impl From<io::Error> for ParserWithDependenciesError {
    fn from(e: io::Error) -> Self {
        ParserWithDependenciesError::Io(e)
    }
}

impl From<nom::IError> for ParserWithDependenciesError {
    fn from(e: nom::IError) -> Self {
        ParserWithDependenciesError::Nom(e)
    }
}

/// Convert OS path to protobuf path (with slashes)
/// Function is `pub(crate)` for test.
pub(crate) fn relative_path_to_protobuf_path(path: &Path) -> String {
    assert!(path.is_relative());
    let path = path.to_str().expect("not a valid UTF-8 name");
    if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.to_owned()
    }
}

pub struct FileDescriptorWithContext {
    /// Protobuf path of parsed file descriptor
    pub protobuf_path: String,
    /// File descriptor itself
    pub file_descriptor: FileDescriptor,
    /// Whether file was an input file
    pub input: bool,
}

struct ParserState<'a> {
    include_path: &'a [&'a Path],
    visited: HashSet<String>,
    parsed: HashMap<String, FileDescriptorWithContext>,
}

impl<'a> ParserState<'a> {
    /// Protobuf path to filesystem path
    fn resolve_protobuf_path(&self, protobuf_path: &str)
        -> Result<PathBuf, ParserWithDependenciesError>
    {
        // protobuf path is a valid path on both Unix and Windows
        let fs_path_relative = Path::new(protobuf_path);
        assert!(fs_path_relative.is_relative());
        for include_dir in self.include_path {
            let fs_path = include_dir.join(fs_path_relative);
            if fs_path.exists() {
                return Ok(fs_path);
            }
        }

        Err(ParserWithDependenciesError::Other(
            format!("protobuf include {:?} is not found in include path: {:?}",
                protobuf_path, self.include_path)))
    }

    /// Parse and store file and its dependencies
    fn process_file(&mut self, protobuf_path: &str, fs_path: &Path, input: bool)
        -> Result<(), ParserWithDependenciesError>
    {
        if !self.visited.insert(protobuf_path.to_owned()) {
            if input {
                if let Some(parsed_file) = self.parsed.get_mut(protobuf_path) {
                    // Make sure file is marked as input even if it was imported
                    parsed_file.input = true;
                }
            }
            return Ok(());
        }

        let mut content = Vec::new();
        File::open(fs_path)?.read_to_end(&mut content)?;

        let file_descriptor = FileDescriptor::parse(content)?;

        for import_path in &file_descriptor.import_paths {
            let import_path = import_path.to_str().expect("not a valid UTF-8");
            let import_fs_path = self.resolve_protobuf_path(import_path)?;
            self.process_file(import_path, &import_fs_path, false)?;
        }

        let prev = self.parsed.insert(protobuf_path.to_owned(), FileDescriptorWithContext {
            protobuf_path: protobuf_path.to_owned(),
            file_descriptor,
            input,
        });
        assert!(prev.is_none());

        Ok(())
    }

    /// Process file passed as input file
    fn process_input_file(&mut self, fs_path: &Path) -> Result<(), ParserWithDependenciesError> {
        let relative_path = self.include_path.iter()
            .filter_map(|include_dir| fs_path.strip_prefix(include_dir).ok())
            .next();
        match relative_path {
            Some(relative_path) => {
                let protobuf_path = relative_path_to_protobuf_path(relative_path);
                self.process_file(&protobuf_path, fs_path, true)?;
                Ok(())
            }
            None => Err(ParserWithDependenciesError::Other(format!(
                "file {:?} must reside in include path {:?}", fs_path, self.include_path)))
        }
    }
}

/// Parse given file and all dependencies.
/// All files must reside in `include_path`.
pub fn parse_with_dependencies(include_path: &[&Path], files: &[&Path])
    -> Result<Vec<FileDescriptorWithContext>, ParserWithDependenciesError>
{
    let mut parser_state = ParserState {
        include_path: include_path,
        visited: HashSet::new(),
        parsed: HashMap::new(),
    };

    for file in files {
        parser_state.process_input_file(file)?;
    }

    Ok(parser_state.parsed.into_iter().map(|(_, v)| v).collect())
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn test_relative_path_to_protobuf_path_windows() {
        assert_eq!("foo/bar.proto", relative_path_to_protobuf_path(&Path::new("foo\\bar.proto")));
    }

    #[test]
    fn test_relative_path_to_protobuf_path() {
        assert_eq!("foo/bar.proto", relative_path_to_protobuf_path(&Path::new("foo/bar.proto")));
    }

    fn parse_with_dependencies_helper(include_path: &[&str], input_files: &[&str])
        -> Vec<FileDescriptorWithContext> {
        let include_path: Vec<_> = include_path.iter().map(Path::new).collect();
        let input_files: Vec<_> = input_files.iter().map(Path::new).collect();
        let mut r = parse_with_dependencies(&include_path, &input_files)
            .expect("parse");
        r.sort_by(|a, b| a.protobuf_path.cmp(&b.protobuf_path));
        r
    }

    // Basic parser invocation
    #[test]
    fn test_parse_with_dependencies_simple() {
        let r = parse_with_dependencies_helper(
            &["test_data/parse_with_dependencies/simple"],
            &["test_data/parse_with_dependencies/simple/aa.proto"]);
        assert_eq!(1, r.len());
        assert_eq!("aa.proto", r[0].protobuf_path);
    }

    // Basic invocation with import
    #[test]
    fn test_parse_with_dependencies_import() {
        let r = parse_with_dependencies_helper(
            &["test_data/parse_with_dependencies/import"],
            &["test_data/parse_with_dependencies/import/aa.proto"]);
        assert_eq!(2, r.len());
        assert_eq!("aa.proto", r[0].protobuf_path);
        assert_eq!("imported.proto", r[1].protobuf_path);
    }

    // Invocation with same file imported twice
    #[test]
    fn test_parse_with_dependencies_import_2() {
        let r = parse_with_dependencies_helper(
            &["test_data/parse_with_dependencies/import_2"],
            &["test_data/parse_with_dependencies/import_2/aa.proto",
                "test_data/parse_with_dependencies/import_2/bb.proto"]);
        assert_eq!(3, r.len());
        assert_eq!("aa.proto", r[0].protobuf_path);
        assert_eq!("bb.proto", r[1].protobuf_path);
        assert_eq!("imported.proto", r[2].protobuf_path);
    }

    // Invocation with imported file also passed as input file
    #[test]
    fn test_parse_with_dependencies_and_input() {
        let r = parse_with_dependencies_helper(
            &["test_data/parse_with_dependencies/and_input"],
            &["test_data/parse_with_dependencies/and_input/imported.proto",
                "test_data/parse_with_dependencies/and_input/aa.proto"]);
        assert_eq!(2, r.len());
        assert_eq!("aa.proto", r[0].protobuf_path);
        assert_eq!("imported.proto", r[1].protobuf_path);
    }
}
