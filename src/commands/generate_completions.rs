use clap::CommandFactory;

use crate::commands::cli::Cli;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Shell {
    Bash,
    Fish,
    Zsh,
    PowerShell,
    Elvish,
}

pub fn generate_completions(shell: Shell) {
    let mut app = Cli::command();
    let app_name = &app.get_name().to_owned();

    match shell {
        Shell::Bash => clap_complete::generate(
            clap_complete::shells::Bash,
            &mut app,
            app_name,
            &mut std::io::stdout(),
        ),
        Shell::Fish => clap_complete::generate(
            clap_complete::shells::Fish,
            &mut app,
            app_name,
            &mut std::io::stdout(),
        ),
        Shell::Zsh => clap_complete::generate(
            clap_complete::shells::Zsh,
            &mut app,
            app_name,
            &mut std::io::stdout(),
        ),
        Shell::PowerShell => clap_complete::generate(
            clap_complete::shells::PowerShell,
            &mut app,
            app_name,
            &mut std::io::stdout(),
        ),
        Shell::Elvish => clap_complete::generate(
            clap_complete::shells::Elvish,
            &mut app,
            app_name,
            &mut std::io::stdout(),
        ),
    }
}
