use std::collections::HashMap;
use std::fs;

// ---- Config ---- //

pub struct Config {
    config: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Config {
        let config = fs::read_to_string("src/config").expect("Unable to read file");
        Config {config:Config::parse_hashmap(&config, "\n", "=")}
    }

    pub fn print(&self) {
        println!("{:?}", self.config);
    }

    pub fn get_env_path(&self) -> String {
        let path = &self.config["minecraft_folder"];
        if path.starts_with("%") {
            let path_splitted: Vec<&str> = path.split("%").collect();
            let var: &str = &format!("{}", &path_splitted[1].to_uppercase());
            let path = match std::env::var(var) {
                Ok(path) => path,
                Err(_) => panic!("Environnement variable '{}' not found", var),
            };
            return path.to_string() + &path_splitted[2].to_string();
        }
        path.to_string()
    }

    fn parse_hashmap(target: &str, entries_separator: &str, key_value_separator: &str) -> HashMap<String, String> {
        let mut result: HashMap<String, String> = HashMap::new();
        let entries = target.split(entries_separator);
        entries.for_each(|e| {
            if let Some((k,v)) = e.split_once(key_value_separator) {
                result.insert(
                    k.trim().to_string(),
                    v.trim().to_string()
                );
            }
        });
        result
    }
}

// ---- File handling ---- //

// #[derive(Debug)]
// pub enum DownloadStatus {
//     Downloading,
//     Finished(Vec<u8>),
// }

// #[derive(Debug)]
// pub struct DownloadInfo {
//     pub status: DownloadStatus,
//     pub downloaded_size: u64,
//     pub total_size: u64,
// }

// pub async fn download_stream(
//     url: String,
//     tx: mpsc::Sender<DownloadInfo>,
// ) -> Result<(), reqwest::Error> {
//     let client = reqwest::Client::builder().build()?;
//     let mut resp = client.get(&url).send().await?;
//     let mut downloaded_size: u64 = 0;
//     let total_size: u64 = resp.content_length().unwrap_or(0);
//     let mut downloaded_data = Vec::with_capacity(total_size as usize);

//     while let Some(chunk) = resp.chunk().await? {
//         downloaded_size += chunk.len() as u64;
//         downloaded_data.extend_from_slice(&chunk);
//         tx.send(DownloadInfo {
//             status: DownloadStatus::Downloading,
//             downloaded_size,
//             total_size,
//         })
//         .unwrap();
//     }
//     tx.send(DownloadInfo {
//         status: DownloadStatus::Finished(downloaded_data),
//         downloaded_size: 0,
//         total_size: 0,
//     })
//     .unwrap();
//     Ok(())
// }

// pub fn extract_archive(data: Vec<u8>, path: &str) -> Result<(), std::io::Error> {
//     let path = std::path::Path::new(path);
//     let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data)).unwrap();

//     for i in 0..archive.len() {
//         let mut file = archive.by_index(i).unwrap();
//         let outpath = path.join(file.name());
//         if (*file.name()).ends_with('/') {
//             std::fs::create_dir_all(&outpath).unwrap();
//         } else {
//             std::fs::create_dir_all(outpath.parent().unwrap()).unwrap();
//             let mut outfile = std::fs::File::create(&outpath).unwrap();
//             std::io::copy(&mut file, &mut outfile).unwrap();
//         }
//     }
//     Ok(())
// }

// pub fn remove_mods(config: Config) -> Result<(), Box<dyn std::error::Error>> {
//     let minecraft_path: &str = &config.minecraft_folder_path;
//     for folder in config.modified_folders {
//         let path = format!("{}\\{}", minecraft_path, folder);
//         std::fs::remove_dir_all(path)?;
//     }
//     Ok(())
// }

// ---- Utils ---- //
