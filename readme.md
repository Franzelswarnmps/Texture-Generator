# What does it do?
This *game/app/tool* generates textures, or runs a more convoluted version of cellular automata
depending on how you want to look at it. The end result is images with interesting patterns. Initially
the tool was made to loosely follow Allan Turing's activator/inhibitor approach of generating
lifelike patterns.

RUN THE WEB DEMO HERE: [https://franzelswarnmps.github.io/Texture-Generator](https://franzelswarnmps.github.io/Texture-Generator/)

# How can I use it?
The easiest way to get started is to open the web version here []
and hold space (run the rules), while occasionally pressing R (randomize all).
You will see the rules being applied continuously. Every time you press R, you
will see a new starting condition and new set of rules applied.

# How is it working?
This tool executes *Rules* on a grid of letters using regular expressions.
Each *Rule*'s regular expression runs on each letter in the grid, including the 8
neighbor letters in the following order:

1 - 2 - 3\
4 - 5 - 6\
7 - 8 - 9\

If letters were assigned to indices in alphabetical order, a rule's regular
expression would run on the string "ABCDEFGHI".

If the regular expression matches, an *Action* is taken. Actions are a set of instructions
describing how the letter and it's 8 neighbors should be updated. Below is the *Action* syntax:

Multiple actions can be concatenated. Below is a single action.
~~~
<location><value>[<chance>]
~~~

Location is the relative index of the letter to be changed.
Value is the new letter to use in those locations.
A chance can be optionally specified. If the rng check fails, the entire action is skipped.

~~~
if <location> is 1-9, use as the relative index
if <location> is A-Z, lookup the indices of cells in range matching the letter
if <location> is *, use all 1-9 as indices
if <value> is 1-9, use the letter from that cell as the value
if <value> is A-Z, use that letter as the value
if <value> is *, use a random letter as the value
<chance> is a nonnegative decimal such that 1.0 >= chance >= std::f32::MIN_POS_VALUE
~~~

# How is the starting image generated?
A layered noise approach is used to generate the starting images.
A random number of generator functions are selected and then fed into
modifier (1 input, 1 output) or combiner (n input, 1 output) functions
until there is only 1 remaining output. This usually results in a slightly modified
version of the standard noise functions like Perlin/Simplex.

# How are colors generated?
Each letter has a chance of being a primary or an accent color.
Letters tend to be grouped by their common occurrences in rules.
The end result is the colors that are more likely to interact with
each other are also more likely to be accent to the same primary color.

# How do I export the image?
In the WASM version, the *Save* button will prompt you to save the png to your
filesystem. On the standalone version, you can paste the string into Chrome and
save the image. The string is contains the image in as a png in base64 with 
content type and content encoding specified for html rendering.

# Build notes
~~~
https://bevy-cheatbook.github.io/platforms/wasm/gh-pages.html
https://dev.to/sbelzile/making-games-in-rust-deploying-a-bevy-app-to-the-web-1ahn
https://rustwasm.github.io/wasm-bindgen/reference/cli.html
~~~
