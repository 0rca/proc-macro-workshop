pub struct CommandBuilder {
    executable: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    current_dir: Option<String>,
}
impl CommandBuilder {
    fn executable(&mut self, executable: String) -> &mut Self {
        self.executable = Some(executable);
        self
    }
    fn args(&mut self, args: Vec<String>) -> &mut Self {
        self.args = Some(args);
        self
    }
    fn env(&mut self, env: Vec<String>) -> &mut Self {
        self.env = Some(env);
        self
    }
    fn current_dir(&mut self, current_dir: String) -> &mut Self {
        self.current_dir = Some(current_dir);
        self
    }
    pub fn arg(&mut self, str: String) -> &mut Self {
        self
    }
    pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
        Ok(Command {
            executable: self.executable.to_owned().unwrap(),
            args: self.args.to_owned().unwrap(),
            env: self.env.to_owned().unwrap(),
            current_dir: self.current_dir.clone(),
        })
    }
}
impl Command {
    pub fn builder() -> CommandBuilder {
        CommandBuilder {
            executable: None,
            args: None,
            env: None,
            current_dir: None,
        }
    }
}
