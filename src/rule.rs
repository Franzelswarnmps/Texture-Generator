use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug,Clone)]
pub struct Action {
    pub chance: Option<f32>,
    pub location: ActionParam,
    pub value: ActionParam,
}

#[derive(Debug,Clone)]
pub struct Actions {
    pub actions: Vec<Action>
}

#[derive(Debug,Clone)]
pub enum ActionParam {
    Char(char),
    Index(usize),
    Wildcard,
}

#[derive(Debug,Clone)]
pub struct Rule {
    action: Vec<Action>,
    condition: Regex,
    original_action: String,
    original_condition: String,
}

impl Rule {
    pub fn new(condition: &str, action: &str) -> Self {
        Self { 
            action: Rule::parse_action(action), 
            condition: Rule::parse_condition(condition), 
            original_action: action.to_owned(), 
            original_condition: condition.to_owned()
        }
    }

    /// Get a reference to the rule's action.
    pub fn action(&self) -> &[Action] {
        self.action.as_ref()
    }

    /// Get a reference to the rule's condition.
    pub fn condition(&self) -> &Regex {
        &self.condition
    }

    /// Set the rule's action.
    pub fn set_action(&mut self, action: &str) {
        self.original_action = action.to_owned();
        self.action = Rule::parse_action(action);
    }

    /// Set the rule's condition.
    pub fn set_condition(&mut self, condition: &str) {
        self.condition = Rule::parse_condition(condition);
    }

    fn parse_condition(condition: &str) -> Regex {
        Regex::new(condition).unwrap()
    }

    fn parse_action(action: &str) -> Vec<Action> {
        lazy_static! {
            static ref PARSE_ACTION: Regex =
                Regex::new(r"([A-Z1-9*])([A-Z1-9*])(?:\[([0]?[.][0-9]+)\])?").unwrap();
        }
        let mut results = vec![];
        for caps in PARSE_ACTION.captures_iter(action) {
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
            results.push(Action {
                chance,
                location: final_location,
                value: final_value,
            })
        }
        results
    }

    /// Get a reference to the rule's original action.
    pub fn original_action(&self) -> &str {
        self.original_action.as_ref()
    }

    /// Get a reference to the rule's original condition.
    pub fn original_condition(&self) -> &str {
        self.original_condition.as_ref()
    }
}