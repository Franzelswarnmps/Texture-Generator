
use std::collections::HashMap;

use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::{prelude::ThreadRng, Rng};
use regex::Regex;

use crate::char_texture::*;
use crate::noise::*;
use crate::random::{SpriteSettings, ColorSettings};

pub struct SpriteGen {
    pub char_texture: CharTexture,
    pub rules: Vec<Rule>,
    pub dimensions: (usize, usize),
    pub char_color: HashMap<char, [u8; 4]>,
}

impl SpriteGen {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            char_texture: CharTexture::new(width, height),
            rules: vec![],
            dimensions: (width, height),
            char_color: HashMap::new(),
        }
    }

    pub fn apply(&mut self) {
        // apply all rules to all pixels
        let mut rng = rand::thread_rng();
        let input = self.char_texture.full_stringify();
        let mut indices: Vec<usize> = (0..self.rules.len()).collect();
        indices.shuffle(&mut rng);
        for index in indices {
            let rule = &self.rules[index];
            let matches = apply_rule(&mut rng, &mut self.char_texture, rule, &input);
            println!(
                "rule \"{}\" matches: {}",
                rule.condition.to_string(),
                matches
            );
        }
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
        self.char_color = color_settings.generate(&mut rng, &letters).into_iter().collect();
    }

    pub fn update_texture(&self, texture: &mut Vec<u8>) {
        for (index, char) in self.char_texture.get_array().iter().enumerate() {
            let offset_index = index * 4;
            for channel in 0..3 {
                if let Some(color_channels) = self.char_color.get(char) {
                    texture[offset_index + channel] = color_channels[channel];
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Rule {
    pub condition: Regex,
    pub action: Vec<RuleAction>,
}
#[derive(Debug,Clone)]
pub struct RuleAction {
    pub chance: Option<f32>,
    pub location: ActionParam,
    pub value: ActionParam,
}
#[derive(Debug,Clone)]
pub enum ActionParam {
    Char(char),
    Index(usize),
    Wildcard,
}

impl Rule {
    pub fn new(condition: String, action: String) -> Self {
        //let regex_bounds = format!("[{}]", CharTexture::BOUNDS_CHAR);
        Self {
            //condition: Regex::new(&format!(r"{}{}{}", regex_bounds, condition, regex_bounds)).unwrap(),
            condition: Regex::new(&condition).unwrap(),
            action: parse_action(action),
        }
    }
}

fn parse_action(action: String) -> Vec<RuleAction> {
    lazy_static! {
        static ref PARSE_ACTION: Regex =
            Regex::new(r"([A-Z1-9*])([A-Z1-9*])(?:\[([0]?[.][0-9]+)\])?").unwrap();
    }
    let mut results = vec![];
    for caps in PARSE_ACTION.captures_iter(&action) {
        let chance = caps
            .get(3)
            .and_then(|c| Some(c.as_str().parse::<f32>().ok()).flatten());
        let location = caps[1].chars().next().unwrap();
        let value = caps[2].chars().next().unwrap();

        let final_location;
        if location.is_ascii_digit() {
            final_location = ActionParam::Index(location.to_digit(10).unwrap() as usize);
        } else if value == '*' {
            final_location = ActionParam::Wildcard;
        } else {
            final_location = ActionParam::Char(location);
        }

        let final_value;
        if value.is_ascii_digit() {
            final_value = ActionParam::Index(value.to_digit(10).unwrap() as usize);
        } else if value == '*' {
            final_value = ActionParam::Wildcard;
        } else {
            final_value = ActionParam::Char(value);
        }
        results.push(RuleAction {
            chance,
            location: final_location,
            value: final_value,
        })
    }
    results
}

pub fn apply_rule(rng: &mut ThreadRng, texture: &mut CharTexture, rule: &Rule, input: &str) -> u32 {
    let mut total = 0;
    for index in 0..texture.pixels.len() {
        let start_index = index * 9;
        let end_index = start_index + 9;
        let match_slice = &input[start_index..end_index];
        if rule.condition.is_match(match_slice) {
            total += 1;
            let (x, y) = texture.xy_from_index(index);
            apply_action(texture, rule, rng, match_slice, x, y);
        }
    }
    total
}

/* <location><value>[<chance>]
    if <location> is 1-9, use as loc
    if <location> is A-Z, use indices of cells with those values
    if <location> is *, use 1-9 indices
    if <value> is 1-9, use value from that cell
    if <value> is A-Z, use that value
    if <value> is *, use random value
    <chance> is a nonnegative decimal such that 1.0 >= chance >= std::f32::MIN_POS_VALUE
    ([A-Z1-9*])([A-Z1-9*])(?:\[([0]?[.][0-9]+)\])?
*/
fn apply_action(
    texture: &mut CharTexture,
    rule: &Rule,
    rng: &mut ThreadRng,
    input: &str,
    x: usize,
    y: usize,
) {
    for action in rule.action.iter() {
        if let Some(chance) = action.chance {
            if chance < rng.gen_range(0.0..1.0) {
                continue; // rng failed, skipping
            }
        }
        let value;
        match action.value {
            ActionParam::Char(c) => {
                value = c;
            }
            ActionParam::Index(i) => {
                value = input.chars().nth(i - 1).unwrap();
            }
            ActionParam::Wildcard => {
                value = input.chars().nth(rng.gen_range(0..9) as usize).unwrap();
            }
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
        for relative in 0..8 {
            // ^ gross
            if indices[relative] {
                if let Some((abs_x, abs_y)) = valid_indices[relative] {
                    texture.set(abs_x, abs_y, value);
                }
            }
        }
    }
}
