use std::marker::PhantomData;

use num_traits::NumCast;
use serde::de::Visitor;
use tracing::trace;

pub(super) struct NumVisitor<N, R, F> {
    setter: F,
    _marker: PhantomData<(N, R)>,
}

impl<N, R, F> NumVisitor<N, R, F> {
    pub(super) fn new(setter: F) -> Self {
        Self {
            setter,
            _marker: PhantomData,
        }
    }
}

impl<'de, N, R, F> Visitor<'de> for NumVisitor<N, R, F>
where
    N: NumCast,
    F: FnOnce(Option<N>) -> R,
{
    type Value = R;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", std::any::type_name::<N>())
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_u8 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_u16 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_u32 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_u64 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_u128 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_i8 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_i16 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_i32 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_i64 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_i128 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_f32 {v:?}");
        Ok((self.setter)(N::from(v)))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        trace!("NumVisitor::visit_f64 {v:?}");
        Ok((self.setter)(N::from(v)))
    }
}
