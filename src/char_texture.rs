pub struct CharTexture {
    pub pixels: Vec<char>,
    pub dimensions: (usize, usize),
    pub dimensions_i: (i32, i32),
    pub changed: bool,
}

impl CharTexture {
    pub const FILL_CHAR: char = '#';
    const OFFSETS_3X3: [(i32, i32); 9] = [
        (-1, 1),
        (0, 1),
        (1, 1),
        (-1, 0),
        (0, 0),
        (1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ];

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![CharTexture::FILL_CHAR; width * height],
            dimensions: (width, height),
            dimensions_i: (width as i32, height as i32),
            changed: false,
        }
    }

    pub fn get_valid_3x3_indices(&self, x: usize, y: usize) -> [Option<(usize, usize)>; 9] {
        let mut result = [None; 9];
        for (index, (offset_x, offset_y)) in CharTexture::OFFSETS_3X3.into_iter().enumerate() {
            let potential_x = x as i32 + offset_x;
            let potential_y = y as i32 + offset_y;
            if !self.out_of_range(potential_x, potential_y) {
                result[index] = Some((potential_x as usize, potential_y as usize))
            }
        }
        result
    }

    pub fn set(&mut self, x: usize, y: usize, letter: char) {
        self.changed = true;
        let index = self.index_from_xy(x, y);
        self.pixels[index] = letter;
    }

    pub fn get(&self, x: usize, y: usize) -> char {
        let index = self.index_from_xy(x, y);
        self.pixels[index]
    }

    pub fn out_of_range(&self, x: i32, y: i32) -> bool {
        x < 0 || y < 0 || x >= self.dimensions_i.0 || y >= self.dimensions_i.1
    }

    pub fn index_from_xy(&self, x: usize, y: usize) -> usize {
        y * self.dimensions.0 + x
    }

    pub fn xy_from_index(&self, index: usize) -> (usize, usize) {
        (index % self.dimensions.0, index / self.dimensions.0)
    }

    pub fn stringify(&self, x: usize, y: usize) -> [char; 9] {
        let mut result = [CharTexture::FILL_CHAR; 9];
        for (index, loc) in self.get_valid_3x3_indices(x, y).into_iter().enumerate() {
            if let Some((abs_x, abs_y)) = loc {
                result[index] = self.get(abs_x, abs_y);
            }
        }
        result
    }

    pub fn full_stringify(&self) -> String {
        let size = self.pixels.len();
        let mut result = String::with_capacity(size * 9);
        for index in 0..size {
            let (x, y) = self.xy_from_index(index);
            result.extend(self.stringify(x, y).iter());
        }
        result
    }

    pub fn count(&self, char: char) -> u32 {
        let mut result = 0;

        for index in 0..self.pixels.len() {
            if self.pixels[index] == char {
                result += 1;
            }
        }

        result
    }

    pub fn get_array(&self) -> &[char] {
        &self.pixels
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.pixels.resize(width * height, CharTexture::FILL_CHAR);
        self.dimensions = (width, height);
        self.dimensions_i = (width.try_into().unwrap(), height.try_into().unwrap());
    }
}
