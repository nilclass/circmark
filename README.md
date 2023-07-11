# circmark

A concise markup language to describe (parts of) linear electronic circuits, in order to draw them.

## Examples

The following circuit consists of two resistors in parallel (`R1` and `R2`) followed by a resistor in series (`R3`).

```circmark
(R1||R2+R3)
```

This input can be rendered with `cm-to-svg`...
```sh
cargo run --bin cm-to-svg -- '(R1||R2+R3)'
```

... to produce this:

![SVG rendering of example1](./assets/example1.svg)

## Status

WIP

## Reference

The grammar looks roughly like this:
```
document       : twoport
               | subcircuit
twoport        : (shunt-link | series-link)+
shunt-link     : '|' subcircuit
series-link    : '-' subcircuit
subcircuit     : element
               | '(' series-group ')'
series-group   : parallel-group '+' parallel-group
               | parallel-group
parallel-group : subcircuit '||' subcircuit
               | subcircuit
element        : 'O'
               | /[RCLVIZ]/ id
id             : /[0-9a-zA-Z]+/
```

### Elements
An `Element` consists of a single component:

- **resistor**, e.g. `R1`
- **capacitor**, e.g. `C1`
- **inductor**, e.g. `L1`
- **voltage source**: e.g. `V1`
- **current source**: e.g. `I1`
- **generic impedance**: e.g. `Z1`
- **open circuit**: `O`

Except for the **open circuit** (`O`) *all* elements consist of a **letter** (`R`, `C`, `L`, `V`, `I`, `Z`) followed by an **identifier**. The identifier can be any alphanumeric string with no spaces.

#### Examples
All of these are **valid**: `R1`, `C27`, `Zth1`, `Lseries`

And all of these are **invalid**: `R*`, `C++`, `Z th 1`

### Groups

Groups represent a series or parallel arrangement of other groups.
A group can also consist of just a single element `(R1)`

Series arrangements are denoted with the `+` operator, parallel arrangements with `||`.
`||` takes precedence over `+`, so `(R1+R2||R3)` means: *R1 is followed by a parallel combination of R2 and R3*. It does *not* mean *R1 together with R2 are in parallel with R3*.

Currently groups **must** be enclosed in **parentheses**. This may change in the future.
Nested groups do not need parentheses, unless they are needed for precedence.
For example `(R1+R2||R3)` and `(R1+(R2||R3))` are the same circuit, while `(R4+R5||R6)` and `((R4+R5)||R6)` are not.

#### Examples

- Series arrangement of `R1` and `R2`: `(R1+R2)`
- Parallel arrangement of `C1` and `L1`: `(C1||L1)`
- Equivalent circuit of a quartz crystal: `(Cp||(Rs+Ls+Cs))`

### Twoport link

There are two types of twoport links:
- *shunt* links which connect the signal path with the common path, denoted by `|`
- *series* links which are within the signal path, denoted by `-`

A twoport link is simply one of those operators (`|` or `-`) followed by either an **Element** or a **Group**.

### Twoport network

A twoport network consists of one or more twoport links.

#### Examples

- A series resistor surrounded by open ports: `|O-R1|O`
- A voltage source, shunted by a capacitor, with an open in between: `|V1-O|C1`

## How to contribute

...
