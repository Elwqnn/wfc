use crate::Color;

/// Sample image represented as a 2D grid of colors
#[derive(Clone)]
pub struct Sample {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Sample {
    pub fn new(width: usize, height: usize, pixels: Vec<Color>) -> Self {
        assert_eq!(pixels.len(), width * height);
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Color {
        self.pixels[y * self.width + x]
    }

    /// Load a sample from an image file
    pub fn from_image(path: &std::path::Path) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| e.to_string())?;
        let rgb = img.to_rgb8();
        let width = rgb.width() as usize;
        let height = rgb.height() as usize;
        let pixels: Vec<Color> = rgb.pixels().map(|p| [p[0], p[1], p[2]]).collect();
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Save sample to an image file
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        let mut img = image::RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.get(x, y);
                img.put_pixel(x as u32, y as u32, image::Rgb(c));
            }
        }
        img.save(path).map_err(|e| e.to_string())
    }
}

/// Default 8x8 pipe pattern sample
pub fn default_pipe_sample() -> Sample {
    let bg: Color = [32, 32, 48];
    let pipe: Color = [64, 128, 192];
    let junction: Color = [96, 192, 255];

    #[rustfmt::skip]
    let pixels = vec![
        bg, bg, bg, bg, bg, bg, bg, bg,
        bg, junction, pipe, pipe, pipe, junction, bg, bg,
        bg, pipe, bg, bg, bg, pipe, bg, bg,
        bg, pipe, bg, junction, pipe, junction, pipe, bg,
        bg, pipe, bg, pipe, bg, bg, pipe, bg,
        bg, junction, pipe, junction, bg, junction, pipe, bg,
        bg, bg, bg, pipe, bg, pipe, bg, bg,
        bg, bg, bg, junction, pipe, junction, bg, bg,
    ];

    Sample::new(8, 8, pixels)
}
