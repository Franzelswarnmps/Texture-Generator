use std::collections::BTreeMap;

use rand::seq::SliceRandom;
use rand::{prelude::ThreadRng, Rng};

use crate::char_texture::*;
use crate::random_rules::{ColorSettings, RuleSettings, SpriteSettings};
use crate::rule::*;
use crate::texture_noise::*;

pub struct SpriteGen {
    pub char_texture: CharTexture,
    pub rules: Vec<Rule>,
    pub char_color: BTreeMap<char, [u8; 4]>,
}

impl SpriteGen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            char_texture: CharTexture::new(width, height),
            rules: vec![],
            char_color: BTreeMap::new(),
        }
    }

    // apply all rules to all pixels
    pub fn apply(&mut self) {
        let mut rng = rand::thread_rng();
        let input = self.char_texture.full_stringify();
        apply_rules(&mut rng, &mut self.char_texture, &self.rules, &input);
    }

    pub fn randomize(&mut self) {
        let seed = SpriteSettings::random();
        self.char_color = seed.colors.into_iter().collect();
        self.rules = seed.rules;

        noise_fill(self);
    }

    pub fn randomize_color(&mut self) {
        let mut rng = rand::thread_rng();
        let letters: Vec<char> = self.char_color.keys().map(|c| c.to_owned()).collect();
        let color_settings = ColorSettings::random(&mut rng);
        self.char_color = color_settings
            .generate(&mut rng, &letters)
            .into_iter()
            .collect();
    }

    pub fn randomize_rules(&mut self) {
        let mut rng = rand::thread_rng();
        let letters: Vec<char> = self.char_color.keys().map(|c| c.to_owned()).collect();
        let rule_settings = RuleSettings::random(&mut rng);
        self.rules = rule_settings.generate(&mut rng, &letters);
    }

    pub fn update_texture(&self, texture: &mut [u8]) {
        for (index, char) in self.char_texture.get_array().iter().enumerate() {
            let offset_index = index * 4;
            for channel in 0..3 {
                if let Some(color_channels) = self.char_color.get(char) {
                    texture[offset_index + channel] = color_channels[channel];
                } else {
                    // if no color, make transparent
                    texture[offset_index + channel] = 0;
                }
            }
        }
    }

    pub fn set_changed(&mut self) {
        self.char_texture.changed = true;
    }

    pub fn is_changed(&self) -> bool {
        self.char_texture.changed
    }
}

pub fn apply_rules(rng: &mut ThreadRng, texture: &mut CharTexture, rules: &[Rule], input: &str) {
    let mut rule_indices: Vec<usize> = (0..rules.len()).collect();
    rule_indices.shuffle(rng);

    for rule_index in rule_indices {
        let rule = &rules[rule_index];
        if !rule.original_condition().is_empty() {
            for index in 0..texture.pixels.len() {
                let start_index = index * 9;
                let end_index = start_index + 9;
                let match_slice = &input[start_index..end_index];

                if rule.condition().is_match(match_slice) {
                    let (x, y) = texture.xy_from_index(index);
                    apply_actions(texture, rules[rule_index].action(), rng, match_slice, x, y);
                }
            }
        }
    }
}

/* <location><value>[<chance>]
if <location> is 1-9, use as the relative index
if <location> is A-Z, lookup the indices cells in range matching the letter
if <location> is *, use all 1-9 as indices
if <value> is 1-9, use the letter from that cell as the value
if <value> is A-Z, use that letter as the value
if <value> is *, use a random letter as the value
<chance> is a nonnegative decimal such that 1.0 >= chance >= std::f32::MIN_POS_VALUE
    ([A-Z1-9*])([A-Z1-9*])(?:\[([0]?[.][0-9]+)\])?
*/
fn apply_actions(
    texture: &mut CharTexture,
    actions: &[Action],
    rng: &mut ThreadRng,
    input: &str,
    x: usize,
    y: usize,
) {
    for action in actions {
        if let Some(chance) = action.chance {
            if chance < rng.gen_range(0.0..1.0) {
                continue; // rng failed, skipping
            }
        }
        let value = match action.value {
            ActionParam::Char(c) => c,
            ActionParam::Index(i) => input.chars().nth(i - 1).unwrap(),
            ActionParam::Wildcard => input.chars().nth(rng.gen_range(0..9) as usize).unwrap(),
        };
        if value == CharTexture::FILL_CHAR {
            // never propagate FILL_CHAR
            return;
        }

        let mut indices = vec![false; 9];
        match action.location {
            ActionParam::Char(c) => {
                indices = input.chars().map(|char| char == c).collect();
            }
            ActionParam::Index(i) => {
                indices[i - 1] = true;
            }
            ActionParam::Wildcard => {
                indices = vec![true; 9];
            }
        }

        let valid_indices = texture.get_valid_3x3_indices(x, y);
        //println!("value: {}, indices: {:?}, valid indices: {:?}",value, indices, valid_indices);
        for relative in 0..9 {
            if indices[relative] {
                if let Some((abs_x, abs_y)) = valid_indices[relative] {
                    texture.set(abs_x, abs_y, value);
                }
            }
        }
    }
}
