pub trait Device: Send {
    fn name(&self) -> &str;
}
