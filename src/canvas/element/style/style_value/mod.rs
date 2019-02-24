mod f64;
pub(crate) use self::f64::F64;

pub trait StyleValue {
    type ValueType;

    fn is_unset(&self) -> bool;
    fn unset(&mut self);

    fn is_inherit(&self) -> bool;
    fn set_inherit(&mut self);

    fn is_auto(&self) -> bool;
    fn set_auto(&mut self);

    fn get(&self) -> Self::ValueType;
    fn set(&mut self, v: Self::ValueType);
}
