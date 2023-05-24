pub mod config {
    pub static APPLICATION_NAME: &str = "mdb";
    use crate::brain::brain::Brain;
    use chrono::Utc;
    use dirs;
    use log::info;
    use serde_derive::Deserialize;
    use shellexpand;
    use std::env::{self, var};
    use std::error::Error;
    use std::io::{self, prelude::*};
    use std::process::Command;
    use std::{fs::File, path::PathBuf};

    #[derive(Deserialize, Debug)]
    pub struct Data {
        pub config: Config,
        pub templates: Vec<Template>,
    }
    impl Data {
        pub fn get_default_template(&self) -> Option<&Template> {
            self.templates.iter().find(|&x| x.id == "default")
        }

        pub fn get_template(&self, templ: &String) -> Option<&Template> {
            self.templates.iter().find(|&x| x.id == templ.to_string())
        }
        pub fn template_file_exists(tmpl: &String) -> bool {
            Template::get_path(tmpl).exists()
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub data: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct ExecCommand {
        run: String,
        args: Vec<String>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub enum TemplateName {
        Text(String),
        Exec(ExecCommand),
    }

    #[derive(Deserialize, Debug)]
    pub struct Template {
        id: String,
        dir: Option<String>,
        content: Option<String>,
        name: Option<TemplateName>,
    }
    pub type OptStr = Option<String>;
    #[derive(Debug, PartialEq)]
    pub enum Named {
        Default,
        Name(OptStr),
        Template(OptStr),
        TemplateWithName(OptStr, OptStr),
    }

    #[derive(Debug)]
    pub enum Action {
        Default(Named),
        New(Named),
        Add(Named),
        // Scan(PathBuf),
        List,
    }

    impl Named {
        pub fn from_template_and_name(template: OptStr, name: OptStr) -> Named {
            if name.is_some() && template.is_some() {
                Named::TemplateWithName(template, name)
            } else if name.is_some() {
                Named::Name(name)
            } else if template.is_some() {
                Named::Template(template)
            } else {
                Named::Default
            }
        }
    }

    impl Template {
        fn err(msg: String) -> io::Result<String> {
            return Err(io::Error::new(io::ErrorKind::NotFound, msg));
        }
        pub fn config_dir() -> PathBuf {
            let mut path = dirs::config_dir().expect("Must have a config dir set");
            path.push(APPLICATION_NAME);
            path
        }

        pub fn get_path(tmpl: &String) -> PathBuf {
            let mut path = tmpl.clone();
            path.push_str(".md");
            Template::config_dir().join(path)
        }

        pub fn create(&self, path: PathBuf, name: &String, overwrite: bool) -> io::Result<String> {
            let mut contents = match &self.content {
                Some(template) => template.clone(),
                None => {
                    let template = Template::get_path(&self.id);
                    if !template.exists() {
                        return Template::err(format!("Template {} not found", self.id));
                    }
                    let mut template_file = File::open(template).unwrap();
                    let mut contents = String::new();
                    template_file.read_to_string(&mut contents)?;
                    contents
                }
            };

            // Load the template and inject the environment variables
            Template::inject_variables(&mut contents, name, &path);

            // Create the target new file and insert the template text
            let mut file_path = match &self.dir {
                Some(dir) => {
                    let path = PathBuf::from(shellexpand::tilde(dir).to_string());
                    if !path.exists() {
                        return Template::err(format!("Template dir {} does not exist", dir));
                    }
                    path
                }
                None => path.clone(),
            };
            if !file_path.exists() {
                return Template::err(format!(
                    "Target dir {} not found",
                    file_path.to_str().unwrap_or_default()
                ));
            }
            file_path.push(&name);
            file_path.set_extension("md");
            if file_path.exists() && overwrite == false {
                return Ok(file_path.to_str().unwrap_or_default().into());
            }
            let mut new_file = File::create(&file_path)?;
            new_file.write_all(contents.as_bytes())?;
            Ok(file_path.to_str().unwrap_or_default().into())
        }

        fn render_to_default(&self, pwd: PathBuf, overwrite: bool) -> Result<String, io::Error> {
            let name: String = match &self.name {
                Some(name) => match name {
                    TemplateName::Text(text) => text.to_string(),
                    TemplateName::Exec(exec) => {
                        info!("{:?}", &exec);
                        let foo = Command::new(&exec.run).args(&exec.args).output().unwrap();
                        let out = String::from_utf8_lossy(&foo.stdout);
                        info!("{:?}", &out);
                        out.to_string()
                    }
                },
                None => {
                    return Template::err(format!(
                        "Template id {} does not create a name, and therefore a name is needed",
                        &self.id
                    ));
                }
            };
            self.create(pwd, &name.to_string(), overwrite)
        }

        fn render_to_name(
            &self,
            pwd: PathBuf,
            name: String,
            overwrite: bool,
        ) -> Result<String, io::Error> {
            self.create(pwd, &name, overwrite)
        }

        fn inject_variables(contents: &mut String, name: &String, path: &PathBuf) {
            *contents = contents
                .replace("$NAME", &name)
                .replace("$DATE", &Utc::now().format("%Y-%m-%d").to_string())
                .replace(
                    "$PWD",
                    &path
                        .as_path()
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                )
                .replace("$PATH", &path.to_str().unwrap_or_default());
        }
    }

    impl Action {
        pub fn act(data: &Data, action: Action) -> Result<String, Box<dyn Error>> {
            let mut pwd = env::current_dir()?;
            return match action {
                Action::Default(name) => Action::handle_named(name, data, pwd, false),
                Action::New(name) => Action::handle_named(name, data, pwd, true),
                Action::Add(name) => {
                    let error_msg = "Name must be set for `add` command.";
                    match name {
                        Named::Name(name) => {
                            info!("Adding {:?}", name);
                            let name = name.expect(error_msg);
                            pwd.push(name);
                            pwd.set_extension("md");
                            Brain::add(data, pwd)
                        }
                        _ => {
                            return Err(Box::from(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                error_msg,
                            )));
                        }
                    }
                }
                Action::List => Brain::list(data),
            };
        }

        fn handle_named(
            name: Named,
            data: &Data,
            mut pwd: PathBuf,
            overwrite: bool,
        ) -> Result<String, Box<dyn Error>> {
            let result = match name {
                Named::Default => match data.get_default_template() {
                    Some(tmpl) => tmpl.render_to_default(pwd, overwrite),
                    None => Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "No default template found",
                    )),
                },
                Named::Name(name) => {
                    let name = name.expect("Name must be set for Named::Name");
                    match data.get_default_template() {
                        Some(tmpl) => {
                            pwd.push(&name);
                            pwd.push(".md");
                            tmpl.render_to_name(pwd, name, overwrite)
                        }
                        None => Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "No default template found",
                        )),
                    }
                }
                Named::Template(template_name) => {
                    let template_name =
                        template_name.expect("Template must be set for Named::Template");
                    match data.get_template(&template_name) {
                        Some(template) => template.render_to_default(pwd, overwrite),
                        None => Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "No default template found",
                        )),
                    }
                }
                Named::TemplateWithName(templ, name) => {
                    let templ = templ.expect("Template must be set for Named::Template");
                    let name = name.expect("Name must be set for Named::Name");
                    match data.get_template(&templ) {
                        Some(template) => template.render_to_name(pwd, name, overwrite),
                        None => Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "No default template found",
                        )),
                    }
                }
            };

            match result {
                Ok(result) => {
                    let editor = var("EDITOR").unwrap();
                    std::process::Command::new(editor)
                        .arg(&result)
                        .status()
                        .expect(&format!("$EDITOR must be set and able to open {}", &result));
                    Ok(result)
                }
                Err(e) => Err(Box::from(e)),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::fs::{self, File};

        use crate::config::config::Named;

        use super::{Data, Template};

        #[test]
        fn test_named_from_template_and_name_default() {
            assert_eq!(Named::from_template_and_name(None, None), Named::Default);
        }
        #[test]
        fn test_named_from_template_and_name_template() {
            assert_eq!(
                Named::from_template_and_name(Some("test".into()), None),
                Named::Template(Some("test".into()))
            );
        }
        #[test]
        fn test_named_from_template_and_name_name() {
            assert_eq!(
                Named::from_template_and_name(None, Some("test".into())),
                Named::Name(Some("test".into()))
            );
        }
        #[test]
        fn test_named_from_template_and_name_both() {
            assert_eq!(
                Named::from_template_and_name(Some("testt".into()), Some("testn".into())),
                Named::TemplateWithName(Some("testt".into()), Some("testn".into()))
            );
        }

        #[test]
        fn test_template_exists_false() {
            assert_eq!(Data::template_file_exists(&String::from("nonexist")), false)
        }

        #[test]
        fn test_template_exists_true() {
            let config_dir = Template::config_dir();
            let config_file_path = config_dir.join("test_file_exists.toml");
            fs::create_dir_all(config_dir).expect("Must have access to create config folder");
            File::create(&config_file_path).expect("Must be able to create dummy test file");
            assert_eq!(Data::template_file_exists(&String::from("nonexist")), false);
            fs::remove_file(&config_file_path).expect("Must be able to delete dummy test file");
            assert_eq!(config_file_path.exists(), false);
        }
    }
}
