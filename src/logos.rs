use std::collections::HashMap;

pub fn logos() -> Vec<(&'static str, &'static str)> {
    vec![
        ("void", "      ________________\n     /                \\\n    /                  \\\n   /                    \\\n  /       ________       \\\n /_______|        |_______\\\n _________  V0ID   ________\n \\       |________|       /\n  \\                      /\n   \\                    /\n    \\                  /\n     \\________________/"),
        ("arch", "      /\\\n     /  \\\n    /\\   \\\n   /      \\\n  /   ,,   \\\n /   |  |  -\n/_-''    ''-_\\"),
        ("ubuntu", "        .-/+oo+/:-.\n     .:+ssssssssss+:.\n    -+ssssssssssssss+-\n   .osssssssssssssssso.\n   /sssssssssssssssssss/\n   +sssssssssssssssssss+\n   /sssssssssssssssssss/\n   .osssssssssssssssso.\n    -+ssssssssssssss+-\n     .:+ssssssssss+:.\n        .-/+oo+/:-."),
        ("debian", "       ,g$$$$$gg.\n    ,g$$P\"     \"Y$$.\n  ,$$P'          `$$$.\n ',$$P       ,ggs.  `$$b:\n `d$$'     ,$P\"'  .  $$$\n  $$P      d$'    ,  $$P\n  $$:      $$.  -   ,d$$'\n  $$;      Y$b._  _,d$P'\n  Y$$.    `.`\"Y$$$$P\"'\n  `$$b      \"-.__\n   `Y$$\n    `Y$$.\n      `$$b.\n        `Y$$b.\n           `\"Y$b._\n               `\"`"),
        ("fedora", "       .-\"\"\"-.\n      /  .-.  \\\n     |  /   \\  |\n     | |  f  | |\n     |  \\   /  |\n      \\  '-'  /\n       '-...-'"),
        ("opensuse", "      __\n     /  \\___\n    | o      \\\n    |  \\___   \\\n     \\_____\\___\\"),
        ("alpine", "        /\\\n       /  \\\n      / /\\ \\\n     / /  \\ \\\n    /_/____\\_\\"),
        ("nixos", "       /\\\n      /  \\\n     / /\\ \\\n    / /  \\ \\\n    \\ \\  / /\n     \\ \\/ /\n      \\  /\n       \\/"),
        ("gentoo", "      _____\n     /  __ \\\n    |  /  \\/\n    | |  __\n    |  \\_\\ \\\n     \\_____/"),
        ("mint", "       __\n      /  \\\n     /    \\\n    |      |\n     \\    /\n      \\  /\n       \\/"),
        ("linux", "        .--.\n       |o_o |\n       |:_/ |\n      //   \\ \\\n     (|     | )\n    /'\\_   _/`\\\n    \\___)=(___/"),
        ("cat", "       /\\_/\\\n      ( o.o )\n       > ^ <\n      /  |  \\\n     (_  |  _)"),
        ("frog", "       @..@\n      (----)\n     ( >__< )\n     ^^ ~~ ^^"),
        ("cow", "      \\   ^__^\n       \\  (oo)\\_______\n          (__)\\       )\\/\\\n              ||----w |\n              ||     ||"),
        ("cactus", "        __\n       /  \\\n       |  |\n    ___|  |___\n   /   |  |   \\\n  |    |  |    |\n   \\___|  |___/\n       |  |\n      /    \\"),
        ("penguin", "        .---.\n       /     \\\n      | o   o |\n      |   ^   |\n      |  ---  |\n       \\     /\n      /|     |\\\n     / |     | \\"),
        ("skull", "       .-\"\"\"-.\n      /       \\\n     |  O   O  |\n     |    ^    |\n     |   ---   |\n      \\  ___  /\n       '-...-'"),
        ("ghost", "      .-\"\"\"-.\n     /       \\\n    |  o   o  |\n    |    ^    |\n    |   ---   |\n     \\       /\n      `~~~~~'"),
        ("robot", "        [###]\n       /|_|_|\\\n      |  o o  |\n      |   ^   |\n      |  ---  |\n      /|     |\\\n     / |     | \\"),
        ("rocket", "         /\\\n        /  \\\n       /    \\\n      |      |\n      |      |\n      |      |\n     /|      |\\\n    / |      | \\\n   /  |      |  \\\n      |______|\n       /____\\\n      /_/  \\_\\"),
        ("coffee", "       ( (\n        ) )\n     ........\n     |      |]\n     \\      /\n      `----'"),
        ("computer", "    .----------------.\n    |  ___________   |\n    | |           |  |\n    | |   hello   |  |\n    | |           |  |\n    | |___________|  |\n    '----------------'\n         |      |\n      ___|______|___"),
        ("tree", "         /\\\n        /  \\\n       /    \\\n      /______\\\n        /  \\\n       /    \\\n      /______\\\n         ||\n         ||"),
        ("moon", "        _____\n      .'     '.\n     /  .-.    \\\n    |  (   )   |\n     \\  '-'   /\n      '.____.'"),
    ]
}

pub fn aliases() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m
}

pub fn distro_colors() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m
}