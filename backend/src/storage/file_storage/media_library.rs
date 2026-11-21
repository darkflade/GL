use crate::domain::model::{FileMeta, FileType, TagCategory};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

const ROOTS: [&str; 2] = ["/media/old", "/media/new"];

#[derive(Serialize)]
pub struct MediaEntry {
    pub path: String,
    pub name: String,
    pub directory: bool,
}

pub struct MediaFile {
    pub path: String,
    pub hash: String,
    pub media_type: FileType,
    pub meta: FileMeta,
}

pub struct ImportFile {
    pub media: MediaFile,
    pub title: String,
    pub tags: BTreeSet<(i16, String)>,
    pub notes: Vec<ImportNote>,
}

#[derive(Deserialize)]
pub struct ImportNote {
    pub text: String,
    #[serde(alias = "x")]
    pub pos_x: f32,
    #[serde(alias = "y")]
    pub pos_y: f32,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Notes {
    List(Vec<ImportNote>),
    One(ImportNote),
}

fn media_root(path: &Path) -> Result<&'static Path> {
    ensure!(
        path.is_absolute()
            && !path
                .components()
                .any(|part| matches!(part, Component::ParentDir)),
        "Choose a path under /media/old or /media/new"
    );
    ROOTS
        .iter()
        .map(Path::new)
        .find(|root| path.starts_with(root))
        .context("Choose a path under /media/old or /media/new")
}

fn display_path(path: &Path) -> Result<String> {
    let value = path.to_str().context("Path is not valid UTF-8")?;
    ensure!(
        !value.chars().any(char::is_control),
        "Path contains control characters"
    );
    Ok(value.to_owned())
}

fn media_path(path: &Path) -> Result<PathBuf> {
    let root = media_root(path)?;
    ensure!(
        !path.starts_with(root.join("thumb")),
        "Choose a media path outside the thumbnail directory"
    );
    let resolved = path
        .canonicalize()
        .with_context(|| format!("Cannot access {}", path.display()))?;
    ensure!(resolved == path, "Symbolic links are not supported");
    ensure!(
        !resolved.ancestors().any(is_sidecar_dir),
        "Choose a media path outside the notes folders"
    );
    display_path(&resolved)?;
    Ok(resolved)
}

fn is_sidecar_dir(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "notes")
}

fn media_kind(path: &Path) -> Option<(FileType, String)> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let media_type = match extension.as_str() {
        "png" | "webp" | "jpg" | "jpeg" | "avif" | "gif" => FileType::Picture,
        "mp4" | "webm" | "wmv" | "mkv" | "mov" | "m4v" | "avi" => FileType::Video,
        "mp3" | "wav" | "aac" | "m4a" | "flac" | "ogg" => FileType::Audio,
        _ => return None,
    };
    Some((media_type, extension))
}

pub fn browse(path: Option<&str>, directories_only: bool) -> Result<Vec<MediaEntry>> {
    let Some(path) = path else {
        return Ok(ROOTS
            .iter()
            .map(|root| MediaEntry {
                path: root.to_string(),
                name: root.to_string(),
                directory: true,
            })
            .collect());
    };
    let path = media_path(Path::new(path))?;
    let thumbnail_dir = media_root(&path)?.join("thumb");
    let mut entries = Vec::new();
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if path == thumbnail_dir || is_sidecar_dir(&path) {
            continue;
        }
        if !kind.is_dir() && (directories_only || !kind.is_file() || media_kind(&path).is_none()) {
            continue;
        }
        if let (Ok(path), Ok(name)) = (display_path(&path), entry.file_name().into_string()) {
            entries.push(MediaEntry {
                path,
                name,
                directory: kind.is_dir(),
            });
        }
    }
    entries.sort_unstable_by(|a, b| {
        b.directory
            .cmp(&a.directory)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(entries)
}

pub fn import_directory(path: &str) -> Result<PathBuf> {
    let path = media_path(Path::new(path))?;
    ensure!(path.is_dir(), "Choose a directory");
    Ok(path)
}

pub fn walk_media(path: &Path) -> impl Iterator<Item = walkdir::Result<PathBuf>> {
    let thumbnails = media_root(path)
        .expect("validated import directory")
        .join("thumb");
    WalkDir::new(path)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(move |entry| entry.path() != thumbnails && !is_sidecar_dir(entry.path()))
        .filter_map(|entry| match entry {
            Ok(entry) if entry.file_type().is_file() && media_kind(entry.path()).is_some() => {
                Some(Ok(entry.into_path()))
            }
            Ok(_) => None,
            Err(err) => Some(Err(err)),
        })
}

pub fn availability(path: &str) -> Result<()> {
    let path = media_path(Path::new(path))?;
    ensure!(path.is_file(), "Path does not point to a regular file");
    File::open(path)?;
    Ok(())
}

pub fn inspect_file(path: &Path) -> Result<MediaFile> {
    let path = media_path(path)?;
    let (media_type, extension) = media_kind(&path).context("Choose a supported media file")?;
    ensure!(path.is_file(), "Choose a file");
    let mut file = File::open(&path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let dimensions = if matches!(media_type, FileType::Picture) {
        imagesize::size(&path).ok()
    } else {
        None
    };
    Ok(MediaFile {
        path: display_path(&path)?,
        hash: format!("{:x}", hasher.finalize()),
        media_type,
        meta: FileMeta {
            width: dimensions
                .as_ref()
                .and_then(|size| size.width.try_into().ok()),
            height: dimensions
                .as_ref()
                .and_then(|size| size.height.try_into().ok()),
            duration_ms: None,
            extension: Some(extension),
        },
    })
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    name.into()
}

fn optional_kind(path: &Path) -> Result<Option<fs::FileType>> {
    match path.symlink_metadata() {
        Ok(meta) if meta.is_symlink() => {
            bail!("Symbolic links are not supported: {}", path.display())
        }
        Ok(meta) => Ok(Some(meta.file_type())),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn read_tags(path: &Path, tags: &mut BTreeSet<(i16, String)>) -> Result<()> {
    if !optional_kind(path)?.is_some_and(|kind| kind.is_file()) {
        return Ok(());
    }
    let data =
        fs::read_to_string(path).with_context(|| format!("Cannot read {}", path.display()))?;
    let mut category = TagCategory::General as i16;
    for line in data
        .trim_start_matches('\u{feff}')
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if let Some(header) = line
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
        {
            category = match header.trim().to_ascii_lowercase().as_str() {
                "artist" => TagCategory::Artist,
                "copyright" => TagCategory::Copyright,
                "character" => TagCategory::Character,
                _ => TagCategory::General,
            } as i16;
        } else {
            ensure!(!line.contains('\0'), "Invalid tag in {}", path.display());
            tags.insert((category, line.to_owned()));
        }
    }
    Ok(())
}

fn read_notes(path: &Path) -> Result<Vec<ImportNote>> {
    let dir = sidecar(path, ".notes");
    if !optional_kind(&dir)?.is_some_and(|kind| kind.is_dir()) {
        return Ok(Vec::new());
    }
    let mut notes = Vec::new();
    for entry in WalkDir::new(&dir).sort_by_file_name() {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type().is_file()
            || !path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let parsed = (|| -> Result<Notes> {
            let data = fs::read_to_string(path)?;
            Ok(serde_json::from_str(data.trim_start_matches('\u{feff}'))?)
        })()
        .with_context(|| format!("Invalid notes JSON in {}", path.display()))?;
        match parsed {
            Notes::One(note) => notes.push(note),
            Notes::List(mut list) => notes.append(&mut list),
        }
    }
    ensure!(
        notes.iter().all(|note| !note.text.contains('\0')
            && note.pos_x.is_finite()
            && note.pos_y.is_finite()),
        "Invalid note data in {}",
        dir.display()
    );
    Ok(notes)
}

pub fn prepare_import(path: &Path) -> Result<ImportFile> {
    let media = inspect_file(path)?;
    let root = media_root(path)?;
    let title = path
        .file_stem()
        .and_then(|name| name.to_str())
        .context("Invalid file name")?
        .to_string();
    let mut tags = BTreeSet::new();
    for directory in path
        .ancestors()
        .skip(1)
        .take_while(|dir| dir.starts_with(root))
    {
        read_tags(&directory.join("directory.txt"), &mut tags)?;
    }
    read_tags(&path.with_extension("txt"), &mut tags)?;
    read_tags(&sidecar(path, ".txt"), &mut tags)?;
    let media_tag = match media.media_type {
        FileType::Picture => "image",
        FileType::Video => "video",
        FileType::Audio => "audio",
    };
    tags.insert((TagCategory::General as i16, media_tag.to_string()));
    Ok(ImportFile {
        media,
        title,
        tags,
        notes: read_notes(path)?,
    })
}
