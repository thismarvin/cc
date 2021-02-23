pub fn remap_range(
    current: f64,
    current_min: f64,
    current_max: f64,
    new_min: f64,
    new_max: f64,
) -> f64 {
    new_min + (new_max - new_min) * (current - current_min) / (current_max - current_min)
}

pub fn almost_equal(value: f64, target: f64, precision: f64) -> bool {
    value - precision <= target && target <= value + precision
}
