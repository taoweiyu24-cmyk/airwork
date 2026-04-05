/// Trait that every wif module must implement.
///
/// Modules encapsulate an optional initialization and shutdown sequence
/// so that the [`ModuleHost`] can manage their lifecycle in a uniform way.
pub trait WifModule: Send + Sync {
    /// The unique name of this module.
    fn name(&self) -> &str;

    /// Called once during application startup.  Default implementation is a no-op.
    fn initialize(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called once during application shutdown.  Default implementation is a no-op.
    fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Manages the lifecycle of all registered [`WifModule`]s.
pub struct ModuleHost {
    modules: Vec<Box<dyn WifModule>>,
}

impl ModuleHost {
    /// Create an empty host with no modules registered.
    pub fn new() -> Self {
        Self { modules: vec![] }
    }

    /// Register a module with the host.
    pub fn register(&mut self, module: Box<dyn WifModule>) {
        self.modules.push(module);
    }

    /// Initialize all modules in registration order.
    pub fn initialize_all(&self) -> anyhow::Result<()> {
        for m in &self.modules {
            m.initialize()?;
        }
        Ok(())
    }

    /// Shut down all modules in reverse registration order.
    pub fn shutdown_all(&self) -> anyhow::Result<()> {
        for m in self.modules.iter().rev() {
            m.shutdown()?;
        }
        Ok(())
    }

    /// Return the names of all registered modules.
    pub fn list(&self) -> Vec<&str> {
        self.modules.iter().map(|m| m.name()).collect()
    }
}

impl Default for ModuleHost {
    fn default() -> Self {
        Self::new()
    }
}
