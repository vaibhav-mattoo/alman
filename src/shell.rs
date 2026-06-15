use crate::cli::cli_data::InitShell;
use askama::Template;
use std::path::PathBuf;

pub struct ShellOpts {
    pub app_path: String,
    pub data_dir: String,
    pub config_dir: String,
}

impl ShellOpts {
    pub fn new() -> Self {
        let app_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("alman"))
            .to_string_lossy()
            .to_string();

        let data_dir = crate::database::persistence::get_data_directory()
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".local")
                    .join("share")
                    .join("alman")
            })
            .to_string_lossy()
            .to_string();

        let config_dir = crate::database::persistence::get_config_directory()
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".config")
                    .join("alman")
            })
            .to_string_lossy()
            .to_string();

        Self {
            app_path,
            data_dir,
            config_dir,
        }
    }
}

macro_rules! make_template {
    ($name:ident, $path:expr) => {
        #[derive(Template)]
        #[template(path = $path)]
        pub struct $name<'a> {
            pub app_path: &'a str,
            pub data_dir: &'a str,
            pub config_dir: &'a str,
        }

        impl<'a> From<&'a ShellOpts> for $name<'a> {
            fn from(opts: &'a ShellOpts) -> Self {
                Self {
                    app_path: &opts.app_path,
                    data_dir: &opts.data_dir,
                    config_dir: &opts.config_dir,
                }
            }
        }
    };
}

make_template!(Bash, "bash.txt");
make_template!(Zsh, "zsh.txt");
make_template!(Fish, "fish.txt");
make_template!(Posix, "posix.txt");

pub fn render_shell_init(shell: InitShell, opts: &ShellOpts) -> String {
    match shell {
        InitShell::Bash => Bash::from(opts).render().expect("bash template render failed"),
        InitShell::Zsh => Zsh::from(opts).render().expect("zsh template render failed"),
        InitShell::Fish => Fish::from(opts).render().expect("fish template render failed"),
        InitShell::Posix => Posix::from(opts).render().expect("posix template render failed"),
    }
}
