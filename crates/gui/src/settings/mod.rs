pub mod keymap;

#[derive(Debug, Clone, Copy)]
pub struct Grid {
    pub rows: i64,
    pub columns: i64,
    pub zs: i64,
}
