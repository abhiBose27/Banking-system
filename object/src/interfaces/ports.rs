
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Ports {
    ControllerRoute
}

impl fmt::Display for Ports {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ports::ControllerRoute => write!(f, "5006")
        }
    }
    
}