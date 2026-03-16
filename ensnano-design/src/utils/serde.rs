pub(crate) fn isize_is_zero(x: &isize) -> bool {
    *x == 0
}

pub(crate) fn f32_is_zero(x: &f32) -> bool {
    *x == 0.0
}

pub(crate) fn is_false(x: &bool) -> bool {
    !x
}
pub(crate) fn is_true(x: &bool) -> bool {
    *x
}
