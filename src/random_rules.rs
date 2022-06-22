use crate::rule::Rule;
use rand::{
    distributions::WeightedIndex,
    prelude::{Distribution, SliceRandom, ThreadRng},
    Rng,
};
use std::collections::HashSet;

use bevy::render::color::Color;

/*
make all sliders, randomize on the sliders

condition sliders (rule-specific):
- letter uniformity [number of letters 0..3]
- letter closeness [spaces between letters 0..2]
- direction bias [top/down/left/right/center/any]

action sliders (rule-specific):
- number of actions range [1-5]
    - activator / inhibitor tendency [activator/inhibitor/neither]
        - activator, value used in condition
        - inhibitor, value not used in condition
        - neither, random under distribution
    - replacement mode [targeted or random direction]
        - targeted, replace by value
        - random, replace by random index
    - condition direction bias [top/down/left/right/center/any]
    - action chance [.001-.999]

rule sliders (all rules):
- percent adversarial [0-100]
- number of rules range [0-30]

color sliders (all rules):
- number of letters [A-Z]*
- letter distribution [top-[10/20/40]/uniform/split]
- percent accent/primary colors [top-[10/20/40]/uniform/split]
- accent offset [distance from primary 0-100]

*/

pub struct RuleSettings {
    /* condition */
    pub letter_distribution: f32,
    pub letter_distribution_pref: u8,
    pub rules_range: (u8, u8),
    pub condition_cell_fill_range: (u8, u8),
    pub condition_direction_weight: Vec<usize>,
    pub condition_direction_chance: f32,
    /* action */
    pub action_max_quantity: u8,
    pub action_chance_for_chance: f32,
    pub action_direction_weight: Vec<usize>,
    pub action_activ_inhib_chance: f32,
    pub action_activ_inhib_ratio: f32,
    pub action_wildcard_chance: f32,
}

impl RuleSettings {
    pub fn random(rng: &mut ThreadRng) -> RuleSettings {
        RuleSettings {
            letter_distribution: rng.gen_range(0.0..=1.0),
            letter_distribution_pref: rng.gen_range(1..=5),
            rules_range: (5, 8),
            condition_cell_fill_range: (1, rng.gen_range(1..=5)),
            condition_direction_weight: (0..=8).map(|_| rng.gen_range(1..=3)).collect(),
            condition_direction_chance: rng.gen_range(0.3..0.7),
            action_max_quantity: rng.gen_range(1..=3),
            action_chance_for_chance: rng.gen_range(0.1..0.4),
            action_direction_weight: (0..=8).map(|_| rng.gen_range(1..=3)).collect(),
            action_activ_inhib_chance: rng.gen_range(0.1..=0.6),
            action_activ_inhib_ratio: rng.gen_range(0.0..=1.0),
            action_wildcard_chance: rng.gen_range(0.05..=0.3),
        }
    }

    pub fn generate(&self, rng: &mut ThreadRng, letters: &[char]) -> Vec<Rule> {
        let mut rules = vec![];
        for _ in 0..rng.gen_range(self.rules_range.0..self.rules_range.1) {
            // make condition and actions for rule
            rules.push(self.generate_single(rng, letters));
        }
        rules
    }

    pub fn generate_single(&self, rng: &mut ThreadRng, letters: &[char]) -> Rule {
        // generate condition
        let num_letters = rng.gen_range(1..=letters.len() / 2);
        let condition_letters: Vec<char> = weighted_values(
            rng,
            letters,
            self.letter_distribution,
            self.letter_distribution_pref as usize,
            num_letters as usize,
        );

        let condition_uses_indices = rng.gen_range(0.0..1.0) < self.condition_direction_chance;
        let condition_cell_fill =
            rng.gen_range(self.condition_cell_fill_range.0..=self.condition_cell_fill_range.1);
        let condition;

        if condition_uses_indices {
            let condition_indices: Vec<usize> = weighted_index_values(
                rng,
                &(0..=8).collect::<Vec<usize>>(),
                &self.condition_direction_weight,
                condition_cell_fill as usize,
            );

            let mut condition_chars = vec!['.'; 9];
            for index in condition_indices {
                condition_chars[index] = *condition_letters.choose(rng).unwrap();
            }
            condition = condition_chars.into_iter().collect();
        } else {
            //use any location condition [not bound char]
            let combined_letters: String = condition_letters.iter().collect();
            condition = format!(
                //r"(?:[{}][^{}{}]*){{{}}}",
                r"(?:[{}].*){{{}}}",
                combined_letters, condition_cell_fill
            );
        }

        // generate action
        let mut actions = String::new();
        //let max_actions = if num_letters < self.action_max_quantity { num_letters } else { self.action_max_quantity };
        for _ in 0..rng.gen_range(1..=self.action_max_quantity) {
            // for each action
            let mut location = weighted_index_values(
                rng,
                &(1..=9).collect::<Vec<usize>>(),
                &self.action_direction_weight,
                1,
            )[0]
            .to_string()
            .chars()
            .next()
            .unwrap();
            let mut value = weighted_values(
                rng,
                letters,
                self.letter_distribution,
                self.letter_distribution_pref as usize,
                1,
            )[0];
            if rng.gen_range(0.0..1.0) < self.action_activ_inhib_chance {
                if rng.gen_range(0.0..1.0) < self.action_activ_inhib_ratio {
                    // active
                    // location = random weighted index, value = one of the condition letters
                    value = *condition_letters.choose(rng).unwrap();
                } else {
                    // inhib
                    // location = one of the condition letters, value = one of the condition letters
                    location = *condition_letters.choose(rng).unwrap();
                }
            } else {
                // any
                // location = random weighted index or *, value = any letter or any index
                if rng.gen_range(0.0..1.0) < self.action_wildcard_chance {
                    location = '*';
                }
                let mut combined_choices = letters.to_owned();
                combined_choices
                    .append(&mut (1..=9).map(|n| char::from_digit(n, 10).unwrap()).collect());
                value = *combined_choices.choose(rng).unwrap();
            }

            actions.push(location);
            actions.push(value);
            if rng.gen_range(0.0..1.0) < self.action_chance_for_chance {
                actions.push_str(&format!(r"[{:.2}]", rng.gen_range(0.01..0.90)));
            }
        }

        Rule::new(&condition, &actions)
    }
}

pub struct ColorSettings {
    pub color_primary_accent_ratio: f32,
    pub color_hue_sat_buffer: f32,
}

impl ColorSettings {
    pub fn random(rng: &mut ThreadRng) -> ColorSettings {
        ColorSettings {
            color_primary_accent_ratio: rng.gen_range(0.05..0.1),
            color_hue_sat_buffer: 0.1,
        }
    }

    pub fn generate(&self, rng: &mut ThreadRng, letters: &[char]) -> Vec<(char, [u8; 4])> {
        let mut primaries: Vec<[f32; 3]> = vec![];
        let mut accents: Vec<[f32; 3]> = vec![];

        //color_primary_accent_ratio: rng.gen_range(0.1..1.0),
        //color_accent_max_offset: rng.gen_range(0..100),
        let primary_hue_sat_range = self.color_hue_sat_buffer..1.0 - self.color_hue_sat_buffer;
        for _ in 0..letters.len() {
            if primaries.is_empty() || rng.gen_range(0.0..1.0) < self.color_primary_accent_ratio {
                // primary
                primaries.push([
                    rng.gen_range(0.0..360.0),
                    rng.gen_range(primary_hue_sat_range.clone()),
                    rng.gen_range(primary_hue_sat_range.clone()),
                ]);
            } else {
                // accent, modify an existing color
                let mut accent = *primaries.choose(rng).unwrap();
                accent[1] += rng
                    .gen_range(-self.color_hue_sat_buffer / 2.0..self.color_hue_sat_buffer / 2.0);
                accent[2] += rng
                    .gen_range(-self.color_hue_sat_buffer / 2.0..self.color_hue_sat_buffer / 2.0);
                accents.push(accent);
            }
        }

        // hsl to rgb
        letters
            .iter()
            .copied()
            .zip(
                primaries
                    .into_iter()
                    .chain(accents.into_iter())
                    .map(|c| Color::hsl(c[0], c[1], c[2]).as_rgba().as_rgba_f32())
                    .map(|r| {
                        [
                            ((r[0] * 255.0).round() as u8),
                            ((r[1] * 255.0).round() as u8),
                            ((r[2] * 255.0).round() as u8),
                            255,
                        ]
                    }),
            )
            .collect()
    }
}

pub struct LetterSettings {
    num_letters: usize,
}

impl LetterSettings {
    pub fn random(rng: &mut ThreadRng) -> Self {
        Self {
            num_letters: rng.gen_range(6..=26),
        }
    }

    pub fn generate(&self) -> Vec<char> {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"[0..self.num_letters]
            .chars()
            .into_iter()
            .collect()
    }
}

pub struct SpriteSettings {
    pub rules: Vec<Rule>,
    pub colors: Vec<(char, [u8; 4])>,
}

impl SpriteSettings {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();

        let letter_settings = LetterSettings::random(&mut rng);
        let letters = letter_settings.generate();

        let rule_settings = RuleSettings::random(&mut rng);
        let rules: Vec<Rule> = rule_settings.generate(&mut rng, &letters);

        let color_settings = ColorSettings::random(&mut rng);
        let colors = color_settings.generate(&mut rng, &letters);

        Self { rules, colors }
    }
}

fn weighted_values<T>(
    rng: &mut ThreadRng,
    values: &[T],
    ratio: f32,
    pref_weight: usize,
    quantity: usize,
) -> Vec<T>
where
    T: Copy,
{
    let num_high = (values.len() as f32 * ratio).round() as usize;
    let num_low = values.len() - num_high;
    let indices: Vec<usize> = (0..num_high)
        .map(|_| pref_weight)
        .chain((0..num_low).map(|_| 1))
        .collect();

    weighted_index_values(rng, values, &indices, quantity)
}

fn weighted_index_values<T>(
    rng: &mut ThreadRng,
    values: &[T],
    indices: &[usize],
    quantity: usize,
) -> Vec<T>
where
    T: Copy,
{
    let distr = WeightedIndex::new(indices).unwrap();
    let mut indices: HashSet<usize> = HashSet::new();
    while indices.len() < quantity {
        indices.insert(distr.sample(rng));
    }

    let mut result = vec![];
    for index in indices.into_iter() {
        result.push(values[index])
    }
    result
}
