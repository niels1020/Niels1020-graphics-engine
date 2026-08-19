#[cfg(test)]
mod tests {
    use crate::logic::commands::Commands;

    #[test]
    fn commands_append_new() {
        let mut a = Commands::new();
        let mut b = Commands::new();
        b.exit();
        a.append(&mut b);
        assert_eq!(a.queue.len(), 1);
        assert_eq!(b.queue.len(), 0);
    }
}
