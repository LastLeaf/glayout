use std::fmt;
use std::f64;

const UNSET: f64 = f64::NAN;
const INHERIT: f64 = f64::NEG_INFINITY;
const AUTO: f64 = f64::INFINITY;

#[derive(Clone)]
pub(crate) struct F64 {
    value: f64,
}

impl fmt::Debug for F64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            UNSET => write!(f, "unset"),
            INHERIT => write!(f, "inherit"),
            AUTO => write!(f, "auto"),
            _ => write!(f, "{:?}", self.value),
        }
    }
}

impl Default for F64 {
    fn default() -> Self {
        Self {
            value: UNSET
        }
    }
}

impl super::StyleValue for F64 {
    type ValueType = f64;

    fn is_unset(&self) -> bool {
        self.value == UNSET
    }
    fn unset(&mut self) {
        self.value = UNSET;
    }

    fn is_inherit(&self) -> bool {
        self.value == INHERIT
    }
    fn set_inherit(&mut self) {
        self.value = INHERIT;
    }

    fn is_auto(&self) -> bool {
        self.value == AUTO
    }
    fn set_auto(&mut self) {
        self.value = AUTO;
    }

    fn get(&self) -> Self::ValueType {
        match self.value {
            UNSET => 0.,
            INHERIT => panic!(),
            AUTO => 0.,
            _ => self.value,
        }
    }
    fn set(&mut self, v: Self::ValueType) {
        self.value = match v {
            UNSET => 0.,
            INHERIT => 0.,
            AUTO => 0.,
            _ => v,
        }
    }
}
