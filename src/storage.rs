use crate::models::{Config, ImageFormat, LocalConfig, Output, PageList, Pagination, TargetFile};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;
use std::error::Error;
use std::fs;
use std::fs::{create_dir_all, read_dir, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

/// Key: the resolve's name, for example: xs, s, m, origin.
/// Value: The url of a resolve.
type Pictures = HashMap<String, String>;

#[derive(Deserialize, Serialize)]
pub struct Scheme {
    pub id: String,
    pub thumbnail: String,
    pub pictures: Pictures,
}

pub trait Storage {
    /// Store the compressed output to a storage, an error will be returned if it fails.
    fn store(&mut self, output: Output) -> Result<Scheme, Box<dyn Error>>;

    /// Find a image, if everything goes well, the first element is the bytes Vec, the second element is
    /// the mime type of this file.
    fn get_picture(
        &self,
        partition: &str,
        hash: &str,
        scheme: &str,
    ) -> Result<(Vec<u8>, String), Box<dyn Error>>;

    /// Determine whether a image exists, and returns None if it does not, or returns an struct Pictures.
    fn exists(&self, partition: &str, id: &str) -> Option<Scheme>;

    /// Delete a image.
    fn delete(&mut self, partition: &str, hash: &str) -> Result<(), String>;

    /// List all schemes in a certain partition.
    /// current >= 1.
    fn list(
        &self,
        current: usize,
        page_size: usize,
        partition: &str,
    ) -> Result<PageList<Scheme>, Box<dyn Error>>;
}

/// Store images in local file system.
pub struct Local {
    root_dir: PathBuf,
    config: &'static Config,

    /// How many images a partition have.
    /// Key: partition string.
    /// Value: count.
    count: HashMap<String, usize>,
}

impl Local {
    pub fn new(mut root_dir: PathBuf, config: &'static Config) -> Self {
        let mut count = HashMap::new();
        // Recounting when the app is restarted.
        for key in config.partitions.keys() {
            root_dir.push(key);
            if !root_dir.exists() {
                create_dir_all(&root_dir).unwrap();
            }
            let reader = read_dir(&root_dir).unwrap();
            count.insert(key.to_string(), reader.count());
            root_dir.pop();
        }
        Local {
            root_dir,
            config,
            count,
        }
    }

    pub fn try_from_str(value: String, config: &'static Config) -> Result<Local, String> {
        let path = PathBuf::from(value);
        if !path.exists() {
            return Err(format!(
                "The path of local 'dir' [{path:?}] does not exist."
            ));
        }
        if !path.is_dir() {
            return Err(format!(
                "The path of local 'dir' [{path:?}] must be a directory."
            ));
        }
        Ok(Local::new(path, config))
    }

    pub fn try_from_self(value: &LocalConfig, config: &'static Config) -> Result<Self, String> {
        let path = PathBuf::from(&value.dir);
        if !path.exists() {
            return Err(format!(
                "The path of local 'dir' [{path:?}] does not exist."
            ));
        }
        if !path.is_dir() {
            return Err(format!(
                "The path of local 'dir' [{path:?}] must be a directory."
            ));
        }
        Ok(Local::new(path, config))
    }
}

pub struct Cos {}

impl Storage for Cos {
    fn store(&mut self, _: Output) -> Result<Scheme, Box<dyn Error>> {
        Err("Not implemented.".into())
    }

    fn get_picture(&self, _: &str, _: &str, _: &str) -> Result<(Vec<u8>, String), Box<dyn Error>> {
        Err("Not implemented.".into())
    }

    fn exists(&self, _: &str, _: &str) -> Option<Scheme> {
        None
    }

    fn delete(&mut self, _: &str, _: &str) -> Result<(), String> {
        Err("Not implemented".into())
    }

    fn list(&self, _: usize, _: usize, _: &str) -> Result<PageList<Scheme>, Box<dyn Error>> {
        Err("Not implemented".into())
    }
}

pub fn generate_url(base_url: &str, partition: &str, name: &str, hash: &str) -> String {
    format!("{}/api/pictures/{}/{}/{}", base_url, partition, name, hash)
}

pub fn parse_picture_name(file_name: &str) -> Option<(&str, &str)> {
    file_name.split_once('.')
}

fn get_thumbnail_name(config: &'static Config, partition_str: &str) -> String {
    if let Some(partition) = config.partitions.get(partition_str) {
        if let Some(t) = &partition.thumbnail {
            t.as_str()
        } else if let Some(first) = &partition.schemes.iter().next() {
            first.0
        } else {
            "origin"
        }
    } else {
        ""
    }
    .to_string()
}

impl Storage for Local {
    fn store(&mut self, output: Output) -> Result<Scheme, Box<dyn Error>> {
        let config = self.config;
        let mut root_dir = self.root_dir.clone();
        let mut pics = Pictures::new();
        root_dir.push(&output.partition);
        root_dir.push(&output.hash);
        create_dir_all(&root_dir)?;
        for target in output.targets {
            info!("WRITING: [{}]", target.name);
            match target.file {
                TargetFile::Original(bytes) => {
                    root_dir.push(&format!("{}.{}", target.name, output.original_format.ext));
                    let file = File::create(&root_dir)?;
                    let mut writer = BufWriter::new(file);
                    writer.write_all(&bytes)?;
                }
                TargetFile::Processed(webp) => {
                    root_dir.push(&format!("{}.webp", target.name));
                    let file = File::create(&root_dir)?;
                    let mut writer = BufWriter::new(file);
                    writer.write_all(&webp)?;
                }
            }
            let mut base_url = config.base_url.clone();
            if base_url.ends_with('/') {
                base_url.remove(base_url.len() - 1);
            }
            pics.insert(
                target.name.clone(),
                generate_url(
                    base_url.as_str(),
                    output.partition.as_str(),
                    target.name.as_str(),
                    output.hash.as_str(),
                ),
            );
            root_dir.pop();
        }
        let old = self.count.get(&output.partition).ok_or("Not found")?;
        let thumbnail = get_thumbnail_name(config, output.partition.as_str());
        self.count.insert(output.partition, old + 1);

        Ok(Scheme {
            id: output.hash.to_string(),
            thumbnail,
            pictures: pics,
        })
    }

    fn get_picture(
        &self,
        partition: &str,
        hash: &str,
        scheme: &str,
    ) -> Result<(Vec<u8>, String), Box<dyn Error>> {
        let mut dir = self.root_dir.clone();
        dir.push(partition);
        dir.push(hash);
        dir.push(&format!("{}.*", scheme));
        let pattern = dir.to_str().unwrap_or("");
        dir = glob::glob(pattern)?.next().ok_or("Not found")??;
        let extension = dir
            .extension()
            .ok_or("")?
            .to_str()
            .ok_or("Unknown extension.")?;
        let format = ImageFormat::try_from(
            image::ImageFormat::from_extension(extension).ok_or("Unknown extension.")?,
        )?;
        let file = File::open(dir)?;
        let mut reader = BufReader::new(file);
        let mut buf: Vec<u8> = Vec::with_capacity(reader.capacity());
        reader.read_to_end(&mut buf)?;
        Ok((buf, format.mime_type))
    }

    fn exists(&self, partition: &str, id: &str) -> Option<Scheme> {
        let mut dir = self.root_dir.clone();
        dir.push(partition);
        dir.push(id);
        let mut result = Pictures::new();
        let dir = read_dir(dir).ok()?;
        for res in dir {
            let entry = res.ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_str()?;
            let (name, _) = parse_picture_name(file_name)?;
            result.insert(
                name.to_string(),
                generate_url(&self.config.base_url, partition, name, id),
            );
        }
        Some(Scheme {
            id: id.to_string(),
            thumbnail: get_thumbnail_name(self.config, partition),
            pictures: result,
        })
    }

    fn delete(&mut self, partition: &str, hash: &str) -> Result<(), String> {
        let mut dir = self.root_dir.clone();
        dir.push(partition);
        dir.push(hash);
        let pattern = dir.to_str().unwrap_or("");
        if let Ok(mut paths) = glob::glob(pattern) {
            if let Some(Ok(path)) = paths.next() {
                if fs::remove_dir_all(&path).is_err() {
                    return Err("Delete failed.".to_string());
                }
                let old = self.count.get(partition).ok_or("Not found")?;
                self.count.insert(partition.to_string(), old - 1);
                return Ok(());
            }
        }
        Err("File not found!".to_string())
    }

    fn list(
        &self,
        current: usize,
        page_size: usize,
        partition: &str,
    ) -> Result<PageList<Scheme>, Box<dyn Error>> {
        let mut dir = self.root_dir.clone();
        dir.push(partition);
        let dir = read_dir(dir)?;
        let n = (current - 1) * page_size;
        let mut skip = dir.skip(n);
        let mut list: Vec<Scheme> = vec![];
        for _ in 0..page_size {
            if let Some(Ok(res)) = skip.next() {
                let file_name = res.file_name();
                let id = file_name.to_str().ok_or("Cannot take the file name.")?;
                let mut pics = Pictures::new();
                for item in read_dir(res.path())? {
                    let item = item?;
                    let file_name = item.file_name();
                    let file_name = file_name.to_str().unwrap_or("");
                    let (name, _) = parse_picture_name(file_name)
                        .ok_or(format!("File name error: {}", file_name))?;

                    pics.insert(
                        name.to_string(),
                        generate_url(&self.config.base_url, partition, name, id),
                    );
                }
                list.push(Scheme {
                    id: id.to_string(),
                    thumbnail: get_thumbnail_name(self.config, partition),
                    pictures: pics,
                });
            } else {
                break;
            }
        }
        Ok(PageList {
            list,
            pagination: Pagination {
                current,
                page_size,
                total: self.count[partition],
            },
        })
    }
}
