//! Help and usage texts captured from the
//! Python at COLUMNS=80, byte for byte (the
//! family's 80-column pin).

pub const TOP_USAGE: &str = r####"usage: tetlock [-h] {confess,grade,ledger,wall,tally} ...
"####;

pub const TOP_HELP: &str = r####"usage: tetlock [-h] {confess,grade,ledger,wall,tally} ...

positional arguments:
  {confess,grade,ledger,wall,tally}
    confess             append a confession
    grade               append an outcome
    ledger              the ledger view
    wall                the wall view
    tally               the tally view

options:
  -h, --help            show this help message and exit
"####;

pub const CONFESS_USAGE: &str = r####"usage: tetlock confess [-h] --action ACTION --sentence SENTENCE --due DUE --at
                       AT --asserter ASSERTER --approval APPROVAL
                       record
"####;

pub const CONFESS_HELP: &str = r####"usage: tetlock confess [-h] --action ACTION --sentence SENTENCE --due DUE --at
                       AT --asserter ASSERTER --approval APPROVAL
                       record

positional arguments:
  record

options:
  -h, --help           show this help message and exit
  --action ACTION
  --sentence SENTENCE
  --due DUE
  --at AT
  --asserter ASSERTER
  --approval APPROVAL
"####;

pub const GRADE_USAGE: &str = r####"usage: tetlock grade [-h] --confession CONFESSION --grade GRADE --happened
                     HAPPENED --at AT --asserter ASSERTER
                     [--evidence EVIDENCE] [--supersedes SUPERSEDES]
                     record
"####;

pub const GRADE_HELP: &str = r####"usage: tetlock grade [-h] --confession CONFESSION --grade GRADE --happened
                     HAPPENED --at AT --asserter ASSERTER
                     [--evidence EVIDENCE] [--supersedes SUPERSEDES]
                     record

positional arguments:
  record

options:
  -h, --help            show this help message and exit
  --confession CONFESSION
  --grade GRADE
  --happened HAPPENED
  --at AT
  --asserter ASSERTER
  --evidence EVIDENCE
  --supersedes SUPERSEDES
"####;

pub const LEDGER_USAGE: &str = r####"usage: tetlock ledger [-h] --now NOW [-o OUTPUT] record
"####;

pub const LEDGER_HELP: &str = r####"usage: tetlock ledger [-h] --now NOW [-o OUTPUT] record

positional arguments:
  record

options:
  -h, --help            show this help message and exit
  --now NOW
  -o OUTPUT, --output OUTPUT
"####;

pub const WALL_USAGE: &str = r####"usage: tetlock wall [-h] --date DATE [-o OUTPUT] record
"####;

pub const WALL_HELP: &str = r####"usage: tetlock wall [-h] --date DATE [-o OUTPUT] record

positional arguments:
  record

options:
  -h, --help            show this help message and exit
  --date DATE
  -o OUTPUT, --output OUTPUT
"####;

pub const TALLY_USAGE: &str = r####"usage: tetlock tally [-h] --now NOW [-o OUTPUT] record
"####;

pub const TALLY_HELP: &str = r####"usage: tetlock tally [-h] --now NOW [-o OUTPUT] record

positional arguments:
  record

options:
  -h, --help            show this help message and exit
  --now NOW
  -o OUTPUT, --output OUTPUT
"####;
