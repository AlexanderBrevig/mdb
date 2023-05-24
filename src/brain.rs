use std::{
    collections::HashSet,
    error::Error,
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

use log::info;
use serde_derive::{Deserialize, Serialize};

use crate::config::Data;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BrainData {
    pub entries: HashSet<PathBuf>,
}
impl BrainData {
    pub(crate) fn add(&mut self, pwd: &PathBuf) {
        self.entries.insert(pwd.to_owned());
    }

    fn list(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|path| path.to_str().unwrap_or_default().to_string())
            .collect()
    }
}

pub struct Brain {}
impl Brain {
    pub(crate) fn add(data: &Data, pwd: PathBuf) -> Result<String, Box<dyn Error>> {
        match pwd.try_exists()? {
            true => {
                info!("Brain::add {:?}", pwd);
                let mut brain = Brain::load(data)?;
                brain.add(&pwd);
                Brain::save(data, &brain)?;
                Ok(pwd.to_str().unwrap_or_default().to_string())
            }
            false => Err(Box::from(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File {:?} not found", pwd),
            ))),
        }
    }

    pub(crate) fn load(data: &Data) -> Result<BrainData, Box<dyn Error>> {
        let data_file = PathBuf::from(shellexpand::tilde(&data.config.data).to_string());
        info!("Brain::load {:?}", data_file);
        fs::create_dir_all(data_file.parent().expect("Data must point to a file"))?;
        let mut file = if data_file.exists() {
            File::open(data_file).unwrap()
        } else {
            File::create(data_file).unwrap()
        };
        // Read config file
        let mut contents = String::new();
        let mut ret = BrainData::default();
        if file.read_to_string(&mut contents).is_ok() {
            ret = toml::from_str(&contents)?;
            info!("Brain::load entries: {:?}", ret.entries);
        }
        file.flush()?;

        Ok(ret)
    }

    pub(crate) fn save(data: &Data, brain: &BrainData) -> Result<(), Box<dyn Error>> {
        let data_file = PathBuf::from(shellexpand::tilde(&data.config.data).to_string());
        info!("Brain::save {:?}", data_file);
        let _ = fs::create_dir_all(data_file.parent().expect("Data must point to a file"));
        let mut file = File::create(&data_file)?;
        info!("Brain::save entries: {:?}", brain.entries);
        let toml = toml::to_string(brain)?;
        file.write_all(toml.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    pub(crate) fn list(data: &Data) -> Result<String, Box<dyn Error>> {
        info!("Brain::list");
        let brain = Brain::load(data)?;
        let list = brain.list().join("\n");
        println!("{}", list);
        Ok(list)
    }

    pub(crate) fn clean(data: &Data) -> Result<String, Box<dyn Error>> {
        info!("Brain::clean");
        let brain = Brain::load(data)?;
        let removals = &brain
            .entries
            .iter()
            .filter(|fil| !fil.exists())
            .collect::<Vec<&PathBuf>>();
        let mut brain = Brain::load(data)?;
        info!("Brain::clean removals: {:?}", removals);
        for removal in removals {
            brain.entries.remove(removal.to_owned());
        }
        Brain::save(data, &brain)?;
        Ok(removals.len().to_string())
    }
}
