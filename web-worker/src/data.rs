use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}
