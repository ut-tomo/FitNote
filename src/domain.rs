/// Shared domain types used across UI and DB layers.

#[derive(Debug, Clone, Copy, Default)]
pub struct Summary {
    pub kcal: f64,
    pub p: f64,
    pub f: f64,
    pub c: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
}

impl Slot {
    pub fn as_str(&self) -> &'static str {
        match self {
            Slot::Breakfast => "breakfast",
            Slot::Lunch => "lunch",
            Slot::Dinner => "dinner",
            Slot::Snack => "snack",
        }
    }
}

pub type Result<T> = anyhow::Result<T>;
