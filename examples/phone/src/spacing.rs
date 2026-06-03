pub fn n_cols(size: iced::Size, n: usize) -> f32 {
    let w = size.width.min(size.height);
    let n = n as f32;
    0.21 * w * n + spacing(size) * (n - 1.0)
}

pub fn n_rows(size: iced::Size, n: usize) -> f32 {
    let w = size.width.min(size.height);
    let n = n as f32;
    0.21 * w * n + spacing(size) * (n - 1.0)
}

pub fn spacing(size: iced::Size) -> f32 {
    let w = size.width.min(size.height);
    0.16 / 3.0 * w
}
