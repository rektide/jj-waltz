//! Local argv declarations; the source producer alone executes them.
use super::*;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Settings {
    pub remote: Option<String>,
    pub command: Option<Vec<String>>,
    pub commands: Option<Vec<Vec<String>>>,
}
impl Settings {
    pub(super) fn argv(&self) -> Result<Vec<Vec<String>>> {
        if self.command.is_some() && self.commands.is_some() {
            bail!("freshen.command and freshen.commands cannot both be present, even when empty");
        }
        let argv = match (&self.command, &self.commands) {
            (Some(argv), None) => vec![argv.clone()],
            (None, Some(argv)) => argv.clone(),
            _ => Vec::new(),
        };
        for (index, command) in argv.iter().enumerate() {
            if command.is_empty() || command[0].is_empty() || command.iter().any(|arg| arg.contains('\0')) {
                bail!("freshen command {} must be a nonempty argv with a nonempty program and no NUL bytes", index + 1);
            }
        }
        Ok(argv)
    }
}
