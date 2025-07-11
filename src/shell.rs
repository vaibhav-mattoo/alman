use crate::cli::cli_data::InitShell;
use std::path::PathBuf;

pub struct ShellOpts {
    pub app_path: String,
    pub data_dir: String,
    pub alias_file_path: String,
}

impl ShellOpts {
    pub fn new() -> Self {
        let app_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("alman"))
            .to_string_lossy()
            .to_string();
        
        let data_dir = crate::database::persistence::get_data_directory()
            .unwrap_or_else(|_| PathBuf::from(".").join(".alman"))
            .to_string_lossy()
            .to_string();
        
        let alias_file_path = crate::database::persistence::load_config()
            .and_then(|cfg| cfg.alias_file_paths.first().cloned())
            .unwrap_or_else(|| crate::database::persistence::get_default_alias_file_path());
        
        Self {
            app_path,
            data_dir,
            alias_file_path,
        }
    }
}

pub fn render_shell_init(shell: InitShell, opts: &ShellOpts) -> String {
    match shell {
        InitShell::Bash => render_bash(opts),
        InitShell::Zsh => render_zsh(opts),
        InitShell::Fish => render_fish(opts),
        InitShell::Posix => render_posix(opts),
    }
}

fn render_bash(opts: &ShellOpts) -> String {
    let mut script = String::new();
    script.push_str("# Alman shell integration for bash\n");
    script.push_str("# Add this to your ~/.bashrc\n\n");
    script.push_str(&format!("export ALMAN_DATA_DIR=\"{}\"\n", opts.data_dir));
    script.push_str(&format!("export ALMAN_BIN=\"{}\"\n\n", opts.app_path));
    
    script.push_str("alman_preexec() {\n");
    script.push_str("    if [ -n \"$1\" ]; then\n");
    script.push_str(&format!("        {} custom \"$1\" 2>/dev/null\n", opts.app_path));
    script.push_str("    fi\n");
    script.push_str("}\n\n");
    
    script.push_str("alman_source_aliases() {\n");
    script.push_str("    # Source all alias files from alman config\n");
    script.push_str(&format!("    if [ -f \"{}/config.json\" ]; then\n", opts.data_dir));
    script.push_str("        # Extract alias file paths from config and source them\n");
    script.push_str("        local alias_files=$(cat \"$ALMAN_DATA_DIR/config.json\" | grep -o '\"[^\"]*aliases[^\"]*\"' | tr -d '\"')\n");
    script.push_str("        for file in $alias_files; do\n");
    script.push_str("            if [ -f \"$file\" ]; then\n");
    script.push_str("                source \"$file\"\n");
    script.push_str("            fi\n");
    script.push_str("        done\n");
    script.push_str("    else\n");
    script.push_str(&format!("        # Fallback to default alias file\n"));
    script.push_str(&format!("        if [ -f \"{}\" ]; then\n", opts.alias_file_path));
    script.push_str(&format!("            source \"{}\"\n", opts.alias_file_path));
    script.push_str("        fi\n");
    script.push_str("    fi\n");
    script.push_str("}\n\n");
    
    // Use DEBUG trap for pre-execution hook
    script.push_str("if [ -n \"$BASH_VERSION\" ] && [ -n \"$PS1\" ]; then\n");  // Limit to interactive shells
    script.push_str("    trap 'alman_preexec \"$BASH_COMMAND\"' DEBUG\n");
    script.push_str("fi\n\n");
    
    script.push_str("# Source aliases on shell startup\n");
    script.push_str("alman_source_aliases\n");
    
    script
}


fn render_zsh(opts: &ShellOpts) -> String {
    let mut script = String::new();
    script.push_str("# Alman shell integration for zsh\n");
    script.push_str("# Add this to your ~/.zshrc\n\n");
    script.push_str(&format!("export ALMAN_DATA_DIR=\"{}\"\n", opts.data_dir));
    script.push_str(&format!("export ALMAN_BIN=\"{}\"\n\n", opts.app_path));
    
    script.push_str("alman_preexec() {\n");
    script.push_str("    if [ -n \"$1\" ]; then\n");
    script.push_str(&format!("        {} custom \"$1\" 2>/dev/null\n", opts.app_path));
    script.push_str("    fi\n");
    script.push_str("}\n\n");
    
    script.push_str("alman_source_aliases() {\n");
    script.push_str("    # Source all alias files from alman config\n");
    script.push_str(&format!("    if [ -f \"{}/config.json\" ]; then\n", opts.data_dir));
    script.push_str("        # Extract alias file paths from config and source them\n");
    script.push_str("        local alias_files=$(cat \"$ALMAN_DATA_DIR/config.json\" | grep -o '\"[^\"]*aliases[^\"]*\"' | tr -d '\"')\n");
    script.push_str("        for file in $alias_files; do\n");
    script.push_str("            if [ -f \"$file\" ]; then\n");
    script.push_str("                source \"$file\"\n");
    script.push_str("            fi\n");
    script.push_str("        done\n");
    script.push_str("    else\n");
    script.push_str(&format!("        # Fallback to default alias file\n"));
    script.push_str(&format!("        if [ -f \"{}\" ]; then\n", opts.alias_file_path));
    script.push_str(&format!("            source \"{}\"\n", opts.alias_file_path));
    script.push_str("        fi\n");
    script.push_str("    fi\n");
    script.push_str("}\n\n");
    
    script.push_str("autoload -U add-zsh-hook\n");
    script.push_str("add-zsh-hook preexec alman_preexec\n\n");
    
    script.push_str("# Source aliases on shell startup\n");
    script.push_str("alman_source_aliases\n");
    
    script
}

fn render_fish(opts: &ShellOpts) -> String {
    let mut script = String::new();
    script.push_str("# Alman shell integration for fish\n");
    script.push_str("# Add this to your ~/.config/fish/config.fish\n\n");
    script.push_str(&format!("set -gx ALMAN_DATA_DIR \"{}\"\n", opts.data_dir));
    script.push_str(&format!("set -gx ALMAN_BIN \"{}\"\n\n", opts.app_path));
    
    script.push_str("function alman_preexec --on-event fish_preexec\n");
    script.push_str("    if test -n \"$argv[1]\"\n");
    script.push_str(&format!("        {} custom \"$argv[1]\" 2>/dev/null\n", opts.app_path));
    script.push_str("    end\n");
    script.push_str("end\n\n");
    
    script.push_str("function alman_source_aliases\n");
    script.push_str("    # Source all alias files from alman config\n");
    script.push_str(&format!("    if test -f \"{}/config.json\"\n", opts.data_dir));
    script.push_str("        # Extract alias file paths from config and source them\n");
    script.push_str("        for file in (cat \"$ALMAN_DATA_DIR/config.json\" | grep -o '\"[^\"]*aliases[^\"]*\"' | tr -d '\"')\n");
    script.push_str("            if test -f \"$file\"\n");
    script.push_str("                source \"$file\"\n");
    script.push_str("            end\n");
    script.push_str("        end\n");
    script.push_str("    else\n");
    script.push_str(&format!("        # Fallback to default alias file\n"));
    script.push_str(&format!("        if test -f \"{}\"\n", opts.alias_file_path));
    script.push_str(&format!("            source \"{}\"\n", opts.alias_file_path));
    script.push_str("        end\n");
    script.push_str("    end\n");
    script.push_str("end\n\n");
    
    script.push_str("# Source aliases on shell startup\n");
    script.push_str("alman_source_aliases\n");
    
    script
}

fn render_posix(opts: &ShellOpts) -> String {
    let mut script = String::new();
    script.push_str("# Alman shell integration for POSIX shells (ksh, dash, etc.)\n");
    script.push_str("# Add this to your ~/.profile or ~/.kshrc\n\n");
    script.push_str(&format!("export ALMAN_DATA_DIR=\"{}\"\n", opts.data_dir));
    script.push_str(&format!("export ALMAN_BIN=\"{}\"\n\n", opts.app_path));
    
    script.push_str("alman_preexec() {\n");
    script.push_str("    if [ -n \"$1\" ]; then\n");
    script.push_str(&format!("        {} custom \"$1\" 2>/dev/null\n", opts.app_path));
    script.push_str("    fi\n");
    script.push_str("}\n\n");
    
    script.push_str("alman_source_aliases() {\n");
    script.push_str("    # Source all alias files from alman config\n");
    script.push_str(&format!("    if [ -f \"{}/config.json\" ]; then\n", opts.data_dir));
    script.push_str("        # Extract alias file paths from config and source them\n");
    script.push_str("        local alias_files=$(cat \"$ALMAN_DATA_DIR/config.json\" | grep -o '\"[^\"]*aliases[^\"]*\"' | tr -d '\"')\n");
    script.push_str("        for file in $alias_files; do\n");
    script.push_str("            if [ -f \"$file\" ]; then\n");
    script.push_str("                . \"$file\"\n");
    script.push_str("            fi\n");
    script.push_str("        done\n");
    script.push_str("    else\n");
    script.push_str(&format!("        # Fallback to default alias file\n"));
    script.push_str(&format!("        if [ -f \"{}\" ]; then\n", opts.alias_file_path));
    script.push_str(&format!("            . \"{}\"\n", opts.alias_file_path));
    script.push_str("        fi\n");
    script.push_str("    fi\n");
    script.push_str("}\n\n");
    
    script.push_str("# Note: POSIX shells don't have built-in preexec hooks\n");
    script.push_str("# You may need to manually call alman_preexec in your PS1\n");
    script.push_str("# Example: PS1='$(alman_preexec $?) $ '\n\n");
    
    script.push_str("# Source aliases on shell startup\n");
    script.push_str("alman_source_aliases\n");
    
    script
} 
