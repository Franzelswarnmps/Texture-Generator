All possible cellular automata are hot trash

* add similar-color grouping to SpriteGen
* use grouping to generate rules
* generate 'seed' ruleset that is only run once during initial state generation
    - 'seed' ruleset are very simple, only matching off 1 center letter with very low probability
    - save seed letters for future rule generation

condition:
- rule that expands a group via consumption of another group (fuzzy)
- rule that can only consume two groups at once (guided fuzzy)
- rule that grows off a seed, consuming any other group (random)

action:
- consumption strategies
        - only consume in certain directions
        - only consume a certain pattern
        - self-propagate a pattern with no cleanup
        - self-propagate a pattern with cleanup
        - ^ with low self-inhibiting chance (plant seed that stops future growth)
            - need to add to condition?